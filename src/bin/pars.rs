use std::fs;
use clap::Parser;
use std::collections::HashMap;

#[derive(Parser)]
struct Cli {
    file_path: std::path::PathBuf,
}

fn main() {
    // get the file path & open file as a string; Need to look at alternatives! 
    let args = Cli::parse();
    let file_path = args.file_path;
    let file_contents = fs::read_to_string(file_path).expect("Unable to read file");
    
    // store each line in a vector
    // I was thinking to somehow parallelize this,
    // each thread looks at subset. I need to check what rust offers though first. 
    let lines: Vec<&str> = file_contents.lines().collect();
    
    // I keep a hashMap with each unique function name(parent_fn) (doesn't include args)
    // as key and a vector as values which stores other functions(child_fns) called 
    // in the parent_fn's scope
    let mut hm: HashMap<String, Vec<String>> = HashMap::new();
    let mut fn_names: Vec<String> = Vec::new();

    // The following approach is pretty lanky! 

    // First pass of the file: 
    // If line starts with 'def'; I can't think of a case where this doesn't happen when defining a
    // fn. We store the fn_name in the hashMap if not already in it, and init an empty vector. 
    //
    // 1 of 2 things can happen then:
    //
    // 1. function definition ends on the same line => def add(a:int, b:int)-> c:int:
    //
    // 2. function definition ends on any line other than the same=>
    // def add(
    // a:int,
    // b:int, 
    // )->
    // c:int:
    //
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
            let mut fn_end: bool = false;

            while !local_line.trim_end().ends_with(':') {
                i += 1;
                if i < lines.len() {
                    local_line.push_str(" ");
                    local_line.push_str(lines[i].trim());
                } else {
                    break;
                }
            }

            // println!("[FUNC]: {}", local_line);
        }
        i += 1;
    }

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

    println!("{:#?}", hm);
}
