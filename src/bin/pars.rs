use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use clap::{Parser};
use serde::{Deserialize, Serialize};
use pars::{FnInfo, find_roots, print_tree}; 

use pars::cli::Cli;
use pars::lang;

#[derive(Debug)]
pub enum Language {
    Py,
    Rs,
    Unknown,
}


#[derive(Debug)]
struct FileInfo<'a> {
    file_type: Language,
    file_path: &'a PathBuf,
    file_size: usize,

}

impl<'a> FileInfo<'a> {
    fn from_path(path: &'a PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(path)?;

        let file_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("py") => Language::Py,
            Some("rs") => Language::Rs,
            _ => Language::Unknown,
        };

        Ok(FileInfo {
            file_type,
            file_path: path,
            file_size: metadata.len() as usize,
        })
    }
}

#[derive(Debug)]
struct Config {
    enable_cache: bool,

}

impl From<&Cli> for Config {
    fn from(cli: &Cli) -> Self {
        Self {
            enable_cache: !cli.no_cache,
        }
    }
}

#[derive(Debug)]
enum ParseError {
    IoError(std::io::Error),
    CacheError(String),
    ParseFailure(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
            ParseError::CacheError(e) => write!(f, "Cache error: {}", e),
            ParseError::ParseFailure(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for ParseError {}



/// Cache entry for parsed function data
#[derive(Serialize, Deserialize, Debug)]
struct CacheEntry {
    file_hash: u64,
    last_modified: u64,
    functions: HashMap<String, FnInfo>,
}

fn read_file(path: &PathBuf) -> Result<String, ParseError> {
    std::fs::read_to_string(path).map_err(ParseError::from)
}

/// create hash based on file content  
fn hash_string(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// get cache file path for a given source file
fn get_cache_path(source_path: &PathBuf) -> PathBuf {
    let mut cache_path = source_path.clone();
    cache_path.set_extension("funcparse_cache");
    cache_path
}

/// load cache results else 
fn load_cache(source_path: &PathBuf, content: &str) -> Result<Option<HashMap<String, FnInfo>>, ParseError> {
    let cache_path = get_cache_path(source_path);
    
    if !cache_path.exists() {
        return Ok(None);
    }
    
    let cache_content = std::fs::read_to_string(&cache_path)
        .map_err(|e| ParseError::CacheError(format!("Failed to read cache: {}", e)))?;
    
    let cache_entry: CacheEntry = serde_json::from_str(&cache_content)
        .map_err(|e| ParseError::CacheError(format!("Failed to parse cache: {}", e)))?;
    
    // compare cached hash to current file content hash 
    let current_hash = hash_string(content);
    let metadata = std::fs::metadata(source_path)?;
    let current_modified = metadata.modified()
        .map_err(|e| ParseError::CacheError(format!("Failed to get file modified time: {}", e)))?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| ParseError::CacheError(format!("Invalid modified time: {}", e)))?
        .as_secs();
    
    if cache_entry.file_hash == current_hash && cache_entry.last_modified == current_modified {
        println!("Using cached parse results");
        Ok(Some(cache_entry.functions))
    } else {
        println!("Cache is stale, will re-parse");
        Ok(None)
    }
}

fn save_cache(source_path: &PathBuf, content: &str, functions: &HashMap<String, FnInfo>) -> Result<(), ParseError> {
    let cache_path = get_cache_path(source_path);
    
    let file_hash = hash_string(content);
    let metadata = std::fs::metadata(source_path)?;
    let last_modified = metadata.modified()
        .map_err(|e| ParseError::CacheError(format!("Failed to get file modified time: {}", e)))?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| ParseError::CacheError(format!("Invalid modified time: {}", e)))?
        .as_secs();
    
    let cache_entry = CacheEntry {
        file_hash,
        last_modified,
        functions: functions.clone(),
    };
    
    let cache_json = serde_json::to_string_pretty(&cache_entry)
        .map_err(|e| ParseError::CacheError(format!("Failed to serialize cache: {}", e)))?;
    
    std::fs::write(&cache_path, cache_json)
        .map_err(|e| ParseError::CacheError(format!("Failed to write cache: {}", e)))?;
    
    println!("Cached parse results to: {}", cache_path.display());
    Ok(())
}

/// parse the file contents 
fn parse_functions(file_info: &FileInfo, content: &str) -> Result<HashMap<String, FnInfo>, ParseError> {
    let (func_def, params_open, param_close, end_def) = match file_info.file_type {
        Language::Py => (
            lang::py::FUNC_DEF,
            lang::py::PARAMS_OPEN,
            lang::py::PARAMS_CLOSE,
            lang::py::END_DEF,
        ),
        Language::Rs => (
            lang::rs::FUNC_DEF,
            lang::rs::PARAMS_OPEN,
            lang::rs::PARAMS_CLOSE,
            lang::rs::END_DEF,
        ),
        Language::Unknown => {
            return Err(ParseError::UnsupportedLanguage("unknown".into()));
        }
    };
    
    let mut functions = HashMap::new();
    let mut fn_names = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() {
        return Err(ParseError::ParseFailure("File is empty".to_string()));
    }
    
    let mut current_fn: Option<String> = None;
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        
        // Check for function definition
        if trimmed.starts_with(func_def) {
            let fn_name= match file_info.file_type {
                    Language::Py => extract_function_name::<lang::py::Python>(trimmed),
                    Language::Rs => extract_function_name::<lang::rs::Rust>(trimmed),
                    Language::Unknown => None,
            };
            if let Some(name)= fn_name=> {
                    // Handle multi-line function definitions
                    let mut complete_def = line.to_string();
                    let mut line_idx = i;
                    
                    // Continue reading until we find the params open (: for py, { for rs etc)
                    while !complete_def.trim_end().ends_with(params_open) && line_idx + 1 < lines.len() {
                        line_idx += 1;
                        complete_def.push(' ');
                        complete_def.push_str(lines[line_idx].trim());
                    }
                    
                    // Store function info
                    functions.insert(
                        fn_name.clone(),
                        FnInfo {
                            line_at_call: i,
                            callees: Vec::new(),
                        }
                    );
                    fn_names.push(fn_name.clone());
                    current_fn = Some(fn_name);
                    i = line_idx;
                }
                None => {
                    eprintln!("Warning: Could not parse function name from line {}: {}", i + 1, trimmed);
                }
        } else if let Some(ref current_func) = current_fn {
            // Simple indentation-based scope detection
            if !line.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                current_fn = None;
            } else {
                // Look for function calls within current scope
                for func_name in &fn_names {
                    if func_name != current_func && line_contains_function_call(line, func_name) {
                        if let Some(info) = functions.get_mut(current_func) {
                            if !info.callees.iter().any(|(name, _)| name == func_name) {
                                info.callees.push((func_name.clone(), i));
                            }
                        }
                    }
                }
            }
        }
        
        i += 1;
    }
    
    Ok(functions)
}

fn extract_function_name<L: lang::LangSpec>(def_line: &str) -> Option<String> {
    let after_def = def_line.trim_start_matches(L::FUNC_DEF).trim();

    if let Some(paren_pos) = after_def.find(L::PARAMS_OPEN) {
        let name = after_def[..paren_pos].trim();
        if !name.is_empty() && L::is_valid_identifier(name) {
            return Some(name.to_string());
        }
    }
    None
}


fn line_contains_function_call(line: &str, func_name: &str) -> bool {
    if !line.contains(func_name) {
        return false;
    }
    
    let pattern = format!("{}(", func_name);
    if line.contains(&pattern) {
        return true;
    }
    
    let method_pattern = format!(".{}(", func_name);
    line.contains(&method_pattern)
}

fn parse_file(file_info: &FileInfo, config: &Config) -> Result<HashMap<String, FnInfo>, ParseError> {
    // Read file content
    // let file_content = read_file(path)?;
    let file_content= read_file(&file_info.file_path)?;
    
    if file_content.is_empty() {
        return Err(ParseError::ParseFailure("File is empty".to_string()));
    }
    
    // Try to load from cache if enabled
    if config.enable_cache {
        match load_cache(&file_info.file_path, &file_content) {
            Ok(Some(cached_functions)) => return Ok(cached_functions),
            Ok(None) => {}, // Cache miss or invalid, continue parsing
            Err(e) => {
                eprintln!("Cache error (continuing without cache): {}", e);
            }
        }
    }
    
    // Parse functions
    let functions = parse_functions(&file_content)?;
    
    // Save to cache if enabled
    if config.enable_cache {
        if let Err(e) = save_cache(&file_info.file_path, &file_content, &functions) {
            eprintln!("Failed to save cache (continuing): {}", e);
        }
    }
    
    Ok(functions)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let config = Config::from(&args);
    let path = &args.file_path;
    let file_info = FileInfo::from_path(&path)?;

    // println!("{:?}", file_info);
    // println!("Lang module works! {}", lang::python::FUNC_DEF);
    //
    println!("Analyzing file: {}", path.display());
    println!("cache?={}", config.enable_cache);

    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()).into());
    }
    
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()).into());
    }
    
    let metadata = std::fs::metadata(path)?;
    let file_size_kb= metadata.len() as f64/ 1024.0;

    if file_size_kb < 1.0{
        println!("File size: {} bytes", metadata.len());

    }else{
        println!("File size: {:.2} KB", file_size_kb);
    }
    


    // Parse the file
    let start = std::time::Instant::now();
    let functions = match parse_file(&file_info, &config) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to parse file: {}", e);
            return Err(e.into());
        }
    };
    
    let parse_duration = start.elapsed();
    println!("Parsing completed in {:?}", parse_duration);
    println!("Found {} functions", functions.len());

    if functions.is_empty() {
        println!("No Python functions found in the file.");
        return Ok(());
    }

    println!("\nFunction Call Hierarchy:\n{}", "=".repeat(40));

    let roots = find_roots(&functions);
    let mut visited = HashSet::new();

    if roots.is_empty() {
        println!("No root functions found (all functions are called by others or part of cycles)");
    } else {
        for (i, root) in roots.iter().enumerate() {
            let is_last = i == roots.len() - 1;
            print_tree(root, &functions, "".to_string(), is_last, &mut visited);
        }
    }

    // Print orphan/unreachable functions
    let mut remaining: Vec<_> = functions
        .keys()
        .filter(|k| !visited.contains(*k))
        .cloned()
        .collect();

    if !remaining.is_empty() {
        println!("\nUnreachable / Orphan Functions:");
        remaining.sort();
        for func_name in remaining {
            let line_num = functions[&func_name].line_at_call + 1; // Human-readable line numbers
            println!("  {} (line {})", func_name, line_num);
        }
    }

    // println!("\nSummary:");
    // println!("  Total functions: {}", functions.len());
    // println!("  Root functions: {}", roots.len());
    // println!("  Orphan functions: {}", functions.len() - visited.len());
    //
    // let total_calls: usize = functions.values().map(|f| f.callees.len()).sum();
    // println!("  Total function calls: {}", total_calls);

    Ok(())
}

