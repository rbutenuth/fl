use std::{env, fs, process};

use fl::interpreter::Engine;
use fl::parser::Parser;

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    if files.is_empty() {
        eprintln!("usage: fl <file>...");
        process::exit(1);
    }

    let engine = Engine::new();

    for file in &files {
        let source = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error reading {}: {}", file, e);
                process::exit(1);
            }
        };
        for value_or_err in Parser::parser_from_str(&source) {
            match value_or_err {
                Ok(value) => match engine.evaluate(value) {
                    Ok(result) => println!("{:?}", result),
                    Err(e) => { println!("{}", e); break }
                },
                Err(e) => { println!("{}", e); break }
            }
        }
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;
    // replace fl:: with crate:: when used in library and not im main.
    use fl::interpreter::function::x;

    #[test]
    fn create() {
        let _engine = Engine::new();
        let _x = x();
    }

}

