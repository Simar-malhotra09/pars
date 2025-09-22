use std::collections::HashSet;
use clap::Parser;
use pars::{FnInfo, find_roots, print_tree};
use pars::cli::Cli;
use pars::file_info::FileInfo;
use pars::config::Config;
use pars::parser::parse_file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let config = Config::from(&args);
    let path = &args.file_path;
    let file_info = FileInfo::from_path(&path)?;

    println!("Analyzing file: {}", path.display());
    println!("cache?={}", config.enable_cache);

    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()).into());
    }
    
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()).into());
    }
    
    let metadata = std::fs::metadata(path)?;
    let file_size_kb = metadata.len() as f64 / 1024.0;

    if file_size_kb < 1.0 {
        println!("File size: {} bytes", metadata.len());
    } else {
        println!("File size: {:.2} KB", file_size_kb);
    }

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
        println!("No functions found in the file.");
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

    let mut remaining: Vec<_> = functions
        .keys()
        .filter(|k| !visited.contains(*k))
        .cloned()
        .collect();

    if !remaining.is_empty() {
        println!("\nUnreachable / Orphan Functions:");
        remaining.sort();
        for func_name in remaining {
            let line_num = functions[&func_name].line_at_call + 1;
            println!("  {} (line {})", func_name, line_num);
        }
    }

    Ok(())
}
