use crate::interpreter::function::{Callable, Parameter, Sample};
use crate::interpreter::scope::MapScope;
use crate::interpreter::{FlFunction, Scope, Symbol};
use crate::list::FlList;
use crate::{FlError, Position, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct ParameterPlaceholder {
    pub symbol: Symbol,
    pub index: usize,
}

impl ParameterPlaceholder {
    pub fn new(symbol: Symbol, index: usize) -> Self {
        ParameterPlaceholder { symbol, index }
    }

    pub fn evaluate(&self) -> Result<Value, FlError> {
        Err(FlError::new("todo"))
    }
}

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

fn compile_all(code: &[Value], name_to_index: &HashMap<String, usize>) -> Vec<Value> {
    code.iter().map(|v| compile(v, name_to_index)).collect()
}

/// Replace every symbol that names a parameter by a `Value::Parameter` with the
/// parameter's index. Lists are compiled recursively. A `Parameter` of an outer
/// function is re-bound: if the name is a parameter here, it gets the new index,
/// otherwise it is turned back into its symbol.
fn compile(code: &Value, name_to_index: &HashMap<String, usize>) -> Value {
    match code {
        Value::List(list) => {
            let elements: Vec<Value> = list.into_iter().map(|v| compile(&v, name_to_index)).collect();
            Value::List(FlList::from_vec_values(elements))
        }
        Value::Symbol(s) => match name_to_index.get(s.name()) {
            Some(&index) => Value::Parameter(Arc::new(ParameterPlaceholder::new(s.clone(), index))),
            None => code.clone(),
        },
        Value::Parameter(p) => match name_to_index.get(p.symbol.name()) {
            Some(&index) => {
                Value::Parameter(Arc::new(ParameterPlaceholder::new(p.symbol.clone(), index)))
            }
            None => Value::Symbol(p.symbol.clone()),
        },
        _ => code.clone(),
    }
}

fn lambda(scope: &Arc<dyn Scope>, p: &Vec<Value>) -> Result<Value, FlError> {
    let list = p[0].as_list()?;
    let mut names: Vec<String> = Vec::new();
    let mut name_to_index: HashMap<String, usize> = HashMap::new();
    let mut parameters = Vec::new();
    let mut is_vararg = false;
    let mut position = Position::anonymous_start();
    // TODO: When we call lambda within a function and a parameter name of the outer
    // function clashes with a parameter name of the lambda, then we have a Parameter
    // instead of a symbol. In this case, extract the String from Parameter.
    for (i, v) in list.into_iter().enumerate() {
        let symbol = v.as_symbol()?; // <- here
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
        code: compile_all(&p[1..], &name_to_index),
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
