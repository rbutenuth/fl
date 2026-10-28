use std::{env, fs, process, sync::Arc};

use fl::interpreter::{Engine, Scope, function::{FlFunction, Parameter}};
use fl::parser::Parser;
use fl::{FlError, Value, ValueType};

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    if files.is_empty() {
        eprintln!("usage: fl <file>...");
        process::exit(1);
    }

    let engine = Engine::new();
    let scope = Arc::clone(&engine.scope);

    let add = FlFunction::new(
        "add",
        "adds two integers",
        vec![
            Parameter::new("a", "", ValueType::Integer, true),
            Parameter::new("b", "", ValueType::Integer, true),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>| match (&p[0], &p[1]) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            _ => Err(FlError::new("not integers")),
        }),
    );
    scope.put("add", Value::Function(Arc::new(add)));


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

