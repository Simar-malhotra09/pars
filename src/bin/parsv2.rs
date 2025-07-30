#[allow(unused_imports)]
use std::fs;

use std::io::{Read, Seek, SeekFrom};
use std::thread;
use clap::Parser;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::collections::VecDeque;
use std::collections::HashMap;
/// CLI definition
#[derive(Parser)]
struct Cli {
    file_path: PathBuf,
}


const KB: usize = 1024;
const BLOCK_SIZE: usize = 16 * KB;
const THREADS: usize = 8;

fn main() {
    let args = Cli::parse();
    let path = Arc::new(args.file_path);

    let file_path_str = path.to_str().unwrap_or("<non-utf8>").to_string();
    println!("Path as string: {}", file_path_str);

    let length: u64 = std::fs::metadata(&*path)
        .expect("Unable to query file details")
        .len();
    println!("Total file length: {} bytes", length);

    let mut chunks = VecDeque::new();
    let mut offset = 0;
    while offset < length {
        let size = std::cmp::min(BLOCK_SIZE as u64, length - offset);
        chunks.push_back((offset, size));
        offset += size;
    }

    let chunks = Arc::new(Mutex::new(chunks));
    let output_data = Arc::new(Mutex::new(Vec::with_capacity(length as usize)));

    // Thread pool
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
    let final_data = output_data.lock().unwrap();
    let file_str = String::from_utf8_lossy(&final_data);
    let lines: Vec<&str> = file_str.lines().collect();

    let mut hm: HashMap<String, Vec<String>> = HashMap::new();
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
                hm.insert(fn_name.clone(), Vec::new());
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
    let mut i = 0;
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
                    if let Some(callees) = hm.get_mut(curr) {
                        if !callees.contains(fname) {
                            callees.push(fname.to_string());
                        }
                    }
                }
            }
        }

        i += 1;
    }

    // println!("{:#?}", hm);

    for(key, value) in hm.into_iter(){
        for v in value{

        }
    }
}
