use std::collections::HashSet;
use std::collections::{HashMap, VecDeque};
use std::fs::{File};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, ValueEnum)]
enum InfoLevel{
    l1: String,
    l2: String,
    l3: String
}

#[derive(Parser, Debug)]
struct Cli {
    file_path: PathBuf,

    #[clap(value_enum, default_value_t=InfoLevel::l1)]
    info_level: InfoLevel,
}

#[derive(Debug)]
struct FnInfo {
    line_at_call: usize,
    callees: Vec<(String, usize)>, // (callee_name, line_number)
}

const KB: usize = 1024;
const BLOCK_SIZE: usize = 16 * KB;
const THREADS: usize = 8;

/// Find root functions (i.e., not called by anyone)
fn find_roots(hm: &HashMap<String, FnInfo>) -> Vec<String> {
    let all_fns: HashSet<&String> = hm.keys().collect();
    let mut called_fns = HashSet::new();

    for info in hm.values() {
        for (callee, _) in &info.callees {
            called_fns.insert(callee);
        }
    }

    all_fns
        .difference(&called_fns)
        .map(|s| (*s).clone())
        .collect()
}

/// Recursively print the tree
fn print_tree(
    name: &str,
    hm: &HashMap<String, FnInfo>,
    prefix: String,
    is_last: bool,
    visited: &mut HashSet<String>,
) {
    if !visited.insert(name.to_string()) {
        return;
    }

    let connector = if is_last { "└── " } else { "├── " };
    let fn_info = &hm[name];

    println!("{}{}{} (line {})", prefix, connector, name, fn_info.line_at_call);

    let new_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    let callees = &fn_info.callees;
    let len = callees.len();
    for (i, (callee, _)) in callees.iter().enumerate() {
        let is_last_callee = i == len - 1;
        print_tree(callee, hm, new_prefix.clone(), is_last_callee, visited);
    }
}

fn main() {
    let args = Cli::parse();
    let path = Arc::new(args.file_path);

    let file_path_str = path.to_str().unwrap_or("<non-utf8>").to_string();
    println!("Path as string: {}", file_path_str);

    let length: u64 = std::fs::metadata(&*path)
        .expect("Unable to query file details")
        .len();
    println!("Total file length: {} bytes", length);

    // Chunk the file into parts
    let mut chunks = VecDeque::new();
    let mut offset = 0;
    while offset < length {
        let size = std::cmp::min(BLOCK_SIZE as u64, length - offset);
        chunks.push_back((offset, size));
        offset += size;
    }

    let chunks = Arc::new(Mutex::new(chunks));
    let output_data = Arc::new(Mutex::new(Vec::with_capacity(length as usize)));

    // Parallel reading
    thread::scope(|scope| {
        for _ in 0..THREADS {
            let path = Arc::clone(&path);
            let chunks = Arc::clone(&chunks);
            let output_data = Arc::clone(&output_data);

            scope.spawn(move || {
                let mut file = File::open(&*path).expect("Unable to open file");

                loop {
                    let (offset, size) = {
                        let mut q = chunks.lock().unwrap();
                        match q.pop_front() {
                            Some(chunk) => chunk,
                            None => break,
                        }
                    };

                    let mut buffer = vec![0_u8; size as usize];
                    file.seek(SeekFrom::Start(offset)).expect("Seek failed");
                    file.read_exact(&mut buffer).expect("Read failed");

                    let mut data = output_data.lock().unwrap();
                    data.extend_from_slice(&buffer);
                    println!("[Thread] Read offset {offset}, size {size}");
                }
            });
        }
    });

    // PHASE 2: Parse
    // This code is really bad and extremly inefficient. 
    // But it works.
    //
    // Simply speaking we do two passes over the file 
    // 1st pass: Get fucn definitions
    // 2nd pass: Get func calls and scope called in 
    //
   


    let final_data = output_data.lock().unwrap();
    let file_str = String::from_utf8_lossy(&final_data);
    let lines: Vec<&str> = file_str.lines().collect();

    let mut hm: HashMap<String, FnInfo> = HashMap::new();
    let mut fn_names: Vec<String> = Vec::new();



    // First pass: collect function definitions
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim_start().starts_with("def") {
            let mut local_line = line.to_string();
            let fn_name = line
                .trim_start()
                .trim_start_matches("def")
                .trim()
                .split('(')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();

            if !hm.contains_key(&fn_name) {
                hm.insert(
                    fn_name.clone(),
                    FnInfo {
                        line_at_call: i,
                        callees: Vec::new(),
                    },
                );
                fn_names.push(fn_name.clone());
            }

            while !local_line.trim_end().ends_with(':') {
                i += 1;
                if i < lines.len() {
                    local_line.push_str(" ");
                    local_line.push_str(lines[i].trim());
                } else {
                    break;
                }
            }
        }
        i += 1;
    }

    // Second pass: detect function calls
    let mut i: usize = 0;
    let mut current_fn: Option<String> = None;

    while i < lines.len() {
        let line = lines[i];

        if line.trim_start().starts_with("def") {
            let fn_name = line
                .trim_start()
                .trim_start_matches("def")
                .trim()
                .split('(')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            current_fn = Some(fn_name.clone());
            i += 1;
            continue;
        }

        if let Some(curr) = &current_fn {
            for fname in &fn_names {
                if line.contains(fname) && fname != curr {
                    if let Some(info) = hm.get_mut(curr) {
                        if !info.callees.iter().any(|(name, _)| name == fname) {
                            info.callees.push((fname.to_string(), i));
                        }
                    }
                }
            }
        }

        i += 1;
    }

    println!("\nFunction Call Hierarchy:\n---------------------------");

    let roots = find_roots(&hm);
    let mut visited = HashSet::new();

    for (i, root) in roots.iter().enumerate() {
        let is_last = i == roots.len() - 1;
        print_tree(root, &hm, "".to_string(), is_last, &mut visited);
    }

    // Optionally, print any leftover unvisited (disconnected) functions
    let mut remaining: Vec<_> = hm
        .keys()
        .filter(|k| !visited.contains(*k))
        .cloned()
        .collect();

    if !remaining.is_empty() {
        println!("\nUnreachable / Orphan Functions:");
        remaining.sort();
        for r in remaining {
            println!("  {} (line {})", r, hm[&r].line_at_call);
        }
    }

}




// [#cfg(test)]
// mod tests{
//     use super::*;
//
//     #[test]
//     fn test_reas  


