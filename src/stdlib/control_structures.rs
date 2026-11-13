use std::sync::Arc;
use crate::{Value, interpreter::{Engine, FlFunction, Scope}};
use crate::interpreter::function::{Sample, Parameter};

pub fn add_control_structures_functions(scope: &dyn Scope) {

    FlFunction::put_in_scope(&*scope, 
    "control_structures",
    "if",
    "Evaluate the first parameter, on true return value of second, otherwise value of the optional third (nil if missing).",
    vec![
        Parameter::new_from_spec("condition", "A condition", "E").unwrap(),
        Parameter::new_from_spec("evaluated-on-true", "Evaluated when condition is true", "").unwrap(),
    ],
    false,
    Box::new(|scope: &Arc<dyn Scope>, p: &Vec<Value>|{
        if Engine::evaluate_to_boolean(&p[0]) {
            p[1].evaluate(scope)
        } else {
            Ok(Value::Nil)
        }
    }),
    vec![
        Sample::new("(if 1 2)", "2"),
        Sample::new("(if 0 2)", "nil"),
        Sample::new("(if 1 (/ 1 0))", "Error: error in call of 'if'"),
        Sample::new("(if 0 (/ 1 0))", "nil"),
        ]
    );

}