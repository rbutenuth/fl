use crate::interpreter::function::{Parameter, Sample};
use crate::{
    Value,
    interpreter::{Engine, FlFunction, Scope},
};
use crate::interpreter::scope::MapScope;
use std::sync::Arc;

pub fn add_control_structures_functions(scope: &dyn Scope) {
    FlFunction::put_in_scope(
        &*scope,
        "control_structures",
        "if",
        "Evaluate the first parameter, on true evaluate second parameter and \
         return the result, otherwise nil.",
        vec![
            Parameter::new_from_spec("condition", "A condition", "E").unwrap(),
            Parameter::new_from_spec("evaluated-on-true", "Evaluated when condition is true", "")
                .unwrap(),
        ],
        false,
        Box::new(|scope: &Arc<dyn Scope>, p: &Vec<Value>| {
            if Engine::evaluate_to_boolean(&p[0]) {
                p[1].evaluate(scope)
            } else {
                Ok(Value::Nil)
            }
        }),
        vec![
            Sample::new("(if 1 2)", "2"),
            Sample::new("(if 0 2)", "nil"),
            Sample::new("(if 1 (/ 1 0))", "Error: division by 0"),
            Sample::new("(if 0 (/ 1 0))", "nil"),
        ],
    );

    FlFunction::put_in_scope(
        &*scope,
        "control_structures",
        "if-else",
        "Evaluate the first parameter, on true evaluate second parameter and return the \
         result, otherwise evaluate third parameter and return the result.",
        vec![
            Parameter::new_from_spec("condition", "A condition", "E").unwrap(),
            Parameter::new_from_spec("evaluated-on-true", "Evaluated when condition is true", "")
                .unwrap(),
            Parameter::new_from_spec(
                "evaluated-on-false",
                "Evaluated when condition is false",
                "",
            )
            .unwrap(),
        ],
        false,
        Box::new(|scope: &Arc<dyn Scope>, p: &Vec<Value>| {
            if Engine::evaluate_to_boolean(&p[0]) {
                p[1].evaluate(scope)
            } else {
                p[2].evaluate(scope)
            }
        }),
        vec![
            Sample::new("(if-else 1 2 3)", "2"),
            Sample::new("(if-else 0 2 3)", "3"),
            Sample::new("(if-else 1 (/ 1 0) 42)", "Error: division by 0"),
            Sample::new("(if-else 0 (/ 1 0) 42)", "42"),
        ],
    );

    FlFunction::put_in_scope(
        &*scope,
        "control_structures",
        "cond",
        "Handle condition expression pairs. When the first condition is true, \
         the first expression is executed and the evaluation result is returned. When \
         not, continue with next condition. When the number of parameters is not even, \
         the last one is evaluated when all conditions are false.",
        vec![
            Parameter::new_from_spec(
                "pair-part",
                "A part of the pairs (or the last, not part of a pair)",
                "",
            )
            .unwrap(),
        ],
        true,
        Box::new(|scope: &Arc<dyn Scope>, p: &Vec<Value>| {
            let mut i = 0;
            while i < p.len() -1 {
                if Engine::evaluate_to_boolean(&p[i].evaluate(scope)?) {
                    return p[i + 1].evaluate(scope);
                }
                i += 2;
            }
            match p.get(i) {
                Some(default) => default.evaluate(scope),
                None => Ok(Value::Nil),
            }
        }),
        vec![
            Sample::new("(cond 1 2 3)", "2"),
            Sample::new("(cond 0 2 3)", "3"),
            Sample::new("(cond 0 2)", "nil"),
            Sample::new("(cond 0 1 1 2 3)", "2"),
            Sample::new("(cond 1 2 (/ 1 0))", "2"),
            Sample::new("(cond 0 (/ 1 0) 42)", "42"),
            Sample::new("(cond 1 (/ 1 0) 0 2)", "Error: division by 0"),
        ],
    );

    FlFunction::put_in_scope(
        &*scope,
        "control_structures",
        "sequential",
        "Evaluate the parameters, return value of last parameter.",
        vec![Parameter::new_from_spec("element", "Expression to evaluate", "").unwrap()],
        true,
        Box::new(|scope: &Arc<dyn Scope>, p: &Vec<Value>| {
            let mut value = Value::Nil;
            for element in p {
                value = element.evaluate(scope)?;
            }
            Ok(value)
        }),
        vec![
            Sample::new("(sequential 1)", "1"),
            Sample::new("(sequential 1 2 3)", "3"),
            Sample::new("(sequential 1 (/ 1 0) 3)", "Error: division by 0"),
            Sample::new("(sequential (/ 1 0) 3)", "Error: division by 0"),
        ],
    );

    FlFunction::put_in_scope(
        &*scope,
        "control_structures",
        "scope",
        "Evaluate the expressions within a new scope, return value of last expression.",
        vec![Parameter::new_from_spec("expression", "Expression to evaluate", "").unwrap()],
        true,
        Box::new(|scope: &Arc<dyn Scope>, p: &Vec<Value>| {
            let local_scope: Arc<dyn Scope> =
                Arc::new(MapScope::new("scope", Some(Arc::clone(scope))));
            let mut value = Value::Nil;
            for expression in p {
                value = expression.evaluate(&local_scope)?;
            }
            Ok(value)
        }),
        vec![
            Sample::new("(scope 1)", "1"),
            Sample::new("(scope 1 2 3)", "3"),
            Sample::new("(scope 1 (/ 1 0) 3)", "Error: division by 0"),
        ],
    );

    // TODO: pipeline
    // TODO: synchronized
    // TODO: throw
    // TODO: throw-with-id
    // TODO: try-catch
    // TODO: try-with
}
