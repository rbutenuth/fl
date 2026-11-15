use std::sync::Arc;
use crate::interpreter::function::{Callable, Parameter, Sample};
use crate::interpreter::scope::MapScope;
use crate::interpreter::{FlFunction, Scope};
use crate::{FlError, Position, Value, ValueTypes};

/// Body + defining scope of a function created at runtime.
pub struct LambdaCallable {
    param_names: Vec<String>,
    code: Vec<Value>,
    defining_scope: Arc<dyn Scope>,
}

impl Callable for LambdaCallable {
    fn call(&self, _caller_scope: &Arc<dyn Scope>, args: &Vec<Value>) -> Result<Value, FlError> {
        let local: Arc<dyn Scope> =
            Arc::new(MapScope::new("lambda", Some(Arc::clone(&self.defining_scope))));
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

pub fn add_lambda_functions(scope: &dyn Scope) {
    FlFunction::put_in_scope(
        scope, "functions", "lambda",
        "Create an anonymous function. The result of the last expression is returned.",
        vec![
            Parameter::new_from_spec("parameter-list", "A list of symbols", "L").unwrap(),
            Parameter::new_from_spec("code", "The body of the function, expressions.", "").unwrap(),
        ],
        true, // body = any number of expressions (stub had `false`)
        Box::new(|scope: &Arc<dyn Scope>, p: &Vec<Value>| {
            let list = p[0].as_list()?;
            let mut names = Vec::new();
            let mut position = Position::anonymous_start();
            for (i, v) in list.into_iter().enumerate() {
                let symbol = v.as_symbol()?;
                if i == 0 { if let Some(pos) = symbol.position() { position = pos.clone(); } }
                if names.iter().any(|n: &String| n == symbol.name()) {
                    return Err(FlError::new(format!("Duplicate parameter {}", symbol.name())));
                }
                names.push(symbol.name().to_string());
            }
            let parameters = names.iter()
                .map(|n| Parameter::new(n.clone(), "", ValueTypes::ALL, true))
                .collect();
            let callable = LambdaCallable {
                param_names: names,
                code: p[1..].to_vec(),
                defining_scope: Arc::clone(scope),
            };
            Ok(Value::Function(Arc::new(FlFunction::new(
                position, "functions", "lambda", "", parameters, false, Box::new(callable), vec![],
            ))))
        }),
        vec![
            Sample::new("((lambda (x) (* x x)) 7)", "49"),
            Sample::new("((lambda (x y) (+ x y)) 1 2)", "3"),
            Sample::new("((lambda () 42))", "42"),
            Sample::new("(pipeline y 10 ((lambda (x) (+ x y)) 1))", "11"), // captures defining scope
        ],
    );
}//		scope.define(new AbstractFunction("lambda", "Create an anonymous function.", "parameter-list", "code...") {
//
//			@Override
//			public FplValue callInternal(Scope scope, FplValue... parameters) throws EvaluationException {
//				FplList params = evaluateToListIfNotAlreadyList(scope, parameters[0]);
//				Position position = FplValue.position(parameters[0]);
//				FplValue[] code = new FplValue[parameters.length - 1];
//				System.arraycopy(parameters, 1, code, 0, code.length);
//				return lambda("lambda", position, FplValue.comments(parameters[0]), params, code, scope);
//			}
//		});
