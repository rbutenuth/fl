use crate::interpreter::function::{Callable, Parameter, Sample};
use crate::interpreter::scope::MapScope;
use crate::interpreter::{FlFunction, Scope};
use crate::{FlError, Position, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Body + defining scope of a function created at runtime.
pub struct LambdaCallable {
    param_names: Vec<String>,
    code: Vec<Value>,
    defining_scope: Arc<dyn Scope>,
}

impl Callable for LambdaCallable {
    fn call(&self, _caller_scope: &Arc<dyn Scope>, args: &Vec<Value>) -> Result<Value, FlError> {
        //        let parameters: Vec<Value>;
        //        let parameter_scope = ParameterScope::new("name", caller_scope, name_to_index, parameters);

        let local: Arc<dyn Scope> = Arc::new(MapScope::new(Some(Arc::clone(&self.defining_scope))));
        for (name, value) in self.param_names.iter().zip(args) {
            local.define(name, value.clone())?;
        }
        let mut result = Value::Nil;
        for expression in &self.code {
            result = expression.evaluate(&local)?;
        }
        Ok(result)
    }
}

fn lambda(scope: &Arc<dyn Scope>, p: &Vec<Value>) -> Result<Value, FlError> {
    let list = p[0].as_list()?;
    let mut names: Vec<String> = Vec::new();
    let mut name_to_index: HashMap<String, usize> = HashMap::new();
    let mut parameters = Vec::new();
    let mut is_vararg = false;
    let mut position = Position::anonymous_start();
    for (i, v) in list.into_iter().enumerate() {
        let symbol = v.as_symbol()?;
        if i == 0 {
            if let Some(pos) = symbol.position() {
                position = pos.clone();
            }
        }
        // "name:spec" splits into name and type spec; without a spec the parameter is lazy.
        let (name, spec) = symbol.name().split_once(':').unwrap_or((symbol.name(), ""));
        // "..." at the end of the last parameter's name marks the function as vararg.
        let name = match name.strip_suffix("...") {
            Some(stripped) if i + 1 == list.len() => {
                is_vararg = true;
                stripped
            }
            _ => name,
        }
        .to_string();
        if name_to_index.contains_key(&name) {
            return Err(FlError::new(format!("Duplicate parameter {}", name)));
        }
        name_to_index.insert(name.clone(), i);
        parameters.push(Parameter::new_from_spec(
            name.clone(),
            symbol.comment().unwrap_or(""),
            spec,
        )?);
        names.push(name);
    }
    let callable = LambdaCallable {
        param_names: names,
        code: p[1..].to_vec(),
        defining_scope: Arc::clone(scope),
    };
    Ok(Value::Function(Arc::new(FlFunction::new(
        position,
        "functions",
        "lambda",
        "",
        parameters,
        is_vararg,
        Box::new(callable),
        vec![],
    ))))
}

pub fn add_lambda_functions(scope: &dyn Scope) {
    FlFunction::put_in_scope(
        scope,
        "functions",
        "lambda",
        "Create an anonymous function. The result of the last expression is returned.",
        vec![
            Parameter::new_from_spec("parameter-list", "A list of symbols", "L").unwrap(),
            Parameter::new_from_spec("code", "The body of the function, expressions.", "").unwrap(),
        ],
        true, // body = any number of expressions
        Box::new(lambda),
        vec![
            Sample::new("((lambda (x) (* x x)) 7)", "49"),
            Sample::new("((lambda (x y) (+ x y)) 1 2)", "3"),
            Sample::new("((lambda () 42))", "42"),
            Sample::new("(pipeline y 10 ((lambda (x) (+ x y)) 1))", "11"), // captures defining scope
        ],
    );
}
