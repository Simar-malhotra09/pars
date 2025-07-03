use std::fs;
use std::env;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    file_path: std::path::PathBuf,
}

fn main() {
    let args= Cli::parse();

    let file_path = args.file_path;
    let file_contents = fs::read_to_string(file_path).expect("Unable to read file");

    let mut lines = file_contents.lines().peekable();

    while let Some(line) = lines.next() {
        if line.trim_start().starts_with("def") {
            let mut local_line = line.to_string();

            while !local_line.trim_end().ends_with(':') {
                if let Some(next_line) = lines.next() {
                    local_line.push_str(" ");
                    local_line.push_str(next_line.trim());
                } else {
                    break;
                }
            }

            println!("[FUNC]: {}", local_line);
        }
    }
}
