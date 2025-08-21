use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{File, metadata};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use pars::{FnInfo, find_roots, print_tree}; 

#[derive(Copy, Clone, Debug, ValueEnum)]
enum InfoLevel {
    L1,
    L2,
    L3,
}

#[derive(Parser, Debug)]
struct Cli {
    file_path: PathBuf,

    #[clap(value_enum, default_value_t = InfoLevel::L1)]
    info_level: InfoLevel,
    
    /// Number of threads for parallel processing if enabled 
    #[clap(long, default_value_t = 8)]
    threads: usize,
    
    /// Block size in KB for parallel reading if enabled
    #[clap(long, default_value_t = 16)]
    block_size_kb: usize,
    
    #[clap(long)]
    no_cache: bool,
    
    #[clap(long)]
    parallel_read: bool,
}


#[derive(Debug)]
struct Config {
    threads: usize,
    block_size: usize,
    enable_cache: bool,
    parallel_read: bool,
}

impl From<&Cli> for Config {
    fn from(cli: &Cli) -> Self {
        Self {
            threads: cli.threads,
            block_size: cli.block_size_kb * 1024,
            enable_cache: !cli.no_cache,
            parallel_read: cli.parallel_read,
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

/// Creates chunks for parallel file reading
fn chunk_file(length: u64, block_size: usize) -> VecDeque<(u64, u64)> {
    let mut chunks = VecDeque::new();
    let mut offset = 0;

    while offset < length {
        let size = std::cmp::min(block_size as u64, length - offset);
        chunks.push_back((offset, size));
        offset += size;
    }

    chunks
}

fn read_file_parallel_fixed(
    path: Arc<PathBuf>,
    config: &Config,
) -> Result<Vec<u8>, ParseError> {
    let length = metadata(&*path)?.len();
    let chunks = Arc::new(Mutex::new(chunk_file(length, config.block_size)));
    
    // Pre-allocate space and use HashMap to maintain order
    let chunk_data = Arc::new(Mutex::new(HashMap::<u64, Vec<u8>>::new()));
    
    let result = thread::scope(|scope| -> Result<Vec<u8>, ParseError> {
        let mut handles = Vec::new();
        
        for thread_id in 0..config.threads {
            let path = Arc::clone(&path);
            let chunks = Arc::clone(&chunks);
            let chunk_data = Arc::clone(&chunk_data);

            let handle = scope.spawn(move || -> Result<(), ParseError> {
                let mut file = File::open(&*path)?;

                loop {
                    let (offset, size) = {
                        let mut q = chunks.lock().map_err(|e| {
                            ParseError::ParseFailure(format!("Mutex poison error: {}", e))
                        })?;
                        match q.pop_front() {
                            Some(chunk) => chunk,
                            None => break,
                        }
                    };

                    let mut buffer = vec![0_u8; size as usize];
                    file.seek(SeekFrom::Start(offset))?;
                    file.read_exact(&mut buffer)?;

                    {
                        let mut data = chunk_data.lock().map_err(|e| {
                            ParseError::ParseFailure(format!("Mutex poison error: {}", e))
                        })?;
                        data.insert(offset, buffer);
                    }
                    
                    println!("[Thread {}] Read offset {}, size {}", thread_id, offset, size);
                }
                Ok(())
            });
            handles.push(handle);
        }
        
        // Wait for all threads and collect any errors
        for handle in handles {
            handle.join().map_err(|e| {
                ParseError::ParseFailure(format!("Thread panicked: {:?}", e))
            })??;
        }
        
        // Reconstruct file in correct order
        let chunk_map = chunk_data.lock().map_err(|e| {
            ParseError::ParseFailure(format!("Mutex poison error: {}", e))
        })?;
        
        let mut sorted_offsets: Vec<u64> = chunk_map.keys().cloned().collect();
        sorted_offsets.sort();
        
        let mut result = Vec::with_capacity(length as usize);
        for offset in sorted_offsets {
            result.extend_from_slice(&chunk_map[&offset]);
        }
        
        Ok(result)
    });
    
    result
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
fn parse_functions(content: &str) -> Result<HashMap<String, FnInfo>, ParseError> {
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
        if trimmed.starts_with("def ") {
            match extract_function_name(trimmed) {
                Some(fn_name) => {
                    // Handle multi-line function definitions
                    let mut complete_def = line.to_string();
                    let mut line_idx = i;
                    
                    // Continue reading until we find the colon
                    while !complete_def.trim_end().ends_with(':') && line_idx + 1 < lines.len() {
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

/// extract function name from a line with 'def'
fn extract_function_name(def_line: &str) -> Option<String> {
    let after_def = def_line.trim_start_matches("def ").trim();
    
    if let Some(paren_pos) = after_def.find('(') {
        let name = after_def[..paren_pos].trim();
        if !name.is_empty() && is_valid_python_identifier(name) {
            return Some(name.to_string());
        }
    }
    
    None
}

fn is_valid_python_identifier(name: &str) -> bool {
    name.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') &&
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
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

fn parse_file(path: &PathBuf, config: &Config) -> Result<HashMap<String, FnInfo>, ParseError> {
    // Read file content
    let file_content = read_file(path)?;
    
    if file_content.is_empty() {
        return Err(ParseError::ParseFailure("File is empty".to_string()));
    }
    
    // Try to load from cache if enabled
    if config.enable_cache {
        match load_cache(path, &file_content) {
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
        if let Err(e) = save_cache(path, &file_content, &functions) {
            eprintln!("Failed to save cache (continuing): {}", e);
        }
    }
    
    Ok(functions)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let config = Config::from(&args);
    let path = &args.file_path;

    println!("Analyzing file: {}", path.display());
    println!("Configuration: threads={}, block_size={}KB, cache={}, parallel_read={}", 
             config.threads, config.block_size / 1024, config.enable_cache, config.parallel_read);

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
    let functions = match parse_file(path, &config) {
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

    println!("\nSummary:");
    println!("  Total functions: {}", functions.len());
    println!("  Root functions: {}", roots.len());
    println!("  Orphan functions: {}", functions.len() - visited.len());
    
    let total_calls: usize = functions.values().map(|f| f.callees.len()).sum();
    println!("  Total function calls: {}", total_calls);

    Ok(())
}

