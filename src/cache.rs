use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::error::ParseError;
use crate::FnInfo;

#[derive(Serialize, Deserialize, Debug)]
struct CacheEntry {
    file_hash: u64,
    last_modified: u64,
    functions: HashMap<String, FnInfo>,
}

pub fn hash_string(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn get_cache_path(source_path: &PathBuf) -> PathBuf {
    let mut cache_path = source_path.clone();
    cache_path.set_extension("funcparse_cache");
    cache_path
}

pub fn load_cache(source_path: &PathBuf, content: &str) -> Result<Option<HashMap<String, FnInfo>>, ParseError> {
    let cache_path = get_cache_path(source_path);
    
    if !cache_path.exists() {
        return Ok(None);
    }
    
    let cache_content = std::fs::read_to_string(&cache_path)
        .map_err(|e| ParseError::CacheError(format!("Failed to read cache: {}", e)))?;
    
    let cache_entry: CacheEntry = serde_json::from_str(&cache_content)
        .map_err(|e| ParseError::CacheError(format!("Failed to parse cache: {}", e)))?;
    
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

pub fn save_cache(source_path: &PathBuf, content: &str, functions: &HashMap<String, FnInfo>) -> Result<(), ParseError> {
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
