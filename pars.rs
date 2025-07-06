use std::fs;
use clap::Parser;
use std::collections::HashMap;

#[derive(Parser)]
struct Cli {
    file_path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();

    let file_path = args.file_path;
    let file_contents = fs::read_to_string(file_path).expect("Unable to read file");

    let lines: Vec<&str> = file_contents.lines().collect();

    let mut hm: HashMap<String, Vec<String>> = HashMap::new();
    let mut fn_names: Vec<String> = Vec::new();

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

            println!("[FUNC]: {}", local_line);
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
