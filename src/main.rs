use std::{env, fs, process};

use fl::parser::Parser;

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    if files.is_empty() {
        eprintln!("usage: fl <file>...");
        process::exit(1);
    }
    for file in &files {
        let source = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error reading {}: {}", file, e);
                process::exit(1);
            }
        };
        for value in Parser::parser_from_str(&source) {
            println!("{:?}", value);
        }
    }
}
