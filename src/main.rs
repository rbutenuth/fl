use std::{env, fs, process, sync::Arc};

use fl::interpreter::{Engine, Scope, function::{FlFunction, Parameter}};
use fl::parser::Parser;
use fl::{Value, ValueType};

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    if files.is_empty() {
        eprintln!("usage: fl <file>...");
        process::exit(1);
    }

    let engine = Engine::new();
    let scope = Arc::clone(&engine.scope);

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
    //use fl::interpreter::function::x;

    #[test]
    fn create() {
        let _engine = Engine::new();
        
    }

}

