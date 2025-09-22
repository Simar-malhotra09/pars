use std::collections::HashMap;
use std::path::PathBuf;
use crate::{FnInfo, lang, cache};
use crate::lang::LangSpec; // Add this import
use crate::file_info::{FileInfo, Language};
use crate::config::Config;
use crate::error::ParseError;

pub fn read_file(path: &PathBuf) -> Result<String, ParseError> {
    std::fs::read_to_string(path).map_err(ParseError::from)
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

pub fn parse_functions(file_info: &FileInfo, content: &str) -> Result<HashMap<String, FnInfo>, ParseError> {
    use crate::lang::{py::Python, rs::Rust};
    
    let (func_def, params_open, _params_close, _end_def) = match file_info.file_type {
        Language::Py => (
            Python::FUNC_DEF,
            Python::PARAMS_OPEN,
            Python::PARAMS_CLOSE,
            Python::END_DEF,
        ),
        Language::Rs => (
            Rust::FUNC_DEF,
            Rust::PARAMS_OPEN,
            Rust::PARAMS_CLOSE,
            Rust::END_DEF,
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
        
        if trimmed.starts_with(func_def) {
            let fn_name = match file_info.file_type {
                Language::Py => extract_function_name::<Python>(trimmed),
                Language::Rs => extract_function_name::<Rust>(trimmed),
                Language::Unknown => None,
            };
            
            if let Some(name) = fn_name {
                let mut complete_def = line.to_string();
                let mut line_idx = i;
                
                while !complete_def.trim_end().ends_with(params_open) && line_idx + 1 < lines.len() {
                    line_idx += 1;
                    complete_def.push(' ');
                    complete_def.push_str(lines[line_idx].trim());
                }
                
                functions.insert(
                    name.clone(),
                    FnInfo {
                        line_at_call: i,
                        callees: Vec::new(),
                    }
                );
                fn_names.push(name.clone());
                current_fn = Some(name);
                i = line_idx;
            } else {
                eprintln!("Warning: Could not parse function name from line {}: {}", i + 1, trimmed);
            }
        } else if let Some(ref current_func) = current_fn {
            if !line.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                current_fn = None;
            } else {
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

pub fn parse_file(file_info: &FileInfo, config: &Config) -> Result<HashMap<String, FnInfo>, ParseError> {
    let file_content = read_file(&file_info.file_path)?;
    
    if file_content.is_empty() {
        return Err(ParseError::ParseFailure("File is empty".to_string()));
    }
    
    if config.enable_cache {
        match cache::load_cache(&file_info.file_path, &file_content) {
            Ok(Some(cached_functions)) => return Ok(cached_functions),
            Ok(None) => {},
            Err(e) => {
                eprintln!("Cache error (continuing without cache): {}", e);
            }
        }
    }
    
    let functions = parse_functions(file_info, &file_content)?;
    
    if config.enable_cache {
        if let Err(e) = cache::save_cache(&file_info.file_path, &file_content, &functions) {
            eprintln!("Failed to save cache (continuing): {}", e);
        }
    }
    
    Ok(functions)
}
