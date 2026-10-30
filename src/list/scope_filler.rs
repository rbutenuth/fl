use crate::{Value, ValueType, interpreter::{FlFunction, Scope}};
use crate::interpreter::function::Parameter;

pub fn add_list_functions(scope: &dyn Scope) {

        FlFunction::put_in_scope(&*scope, 
        "some package",
        "add",
        "adds two integers",
        vec![
            Parameter::new("a", "", ValueType::Integer, true),
            Parameter::new_from_spec("b", "", "EI").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>| 
            Ok(Value::Integer(p[0].as_integer()? + p[1].as_integer()?))
        ));


}
