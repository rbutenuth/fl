use std::sync::Arc;

use crate::interpreter::scope::MapScope;
use crate::interpreter::{FlFunction, Parameter, Scope};
use crate::list::scope_filler;
use crate::parser::Parser;
use crate::{FlError, Value};

pub struct Engine {
    pub scope: Arc<dyn Scope>,
}

impl Engine {
    pub fn new() -> Engine {
        let global = Arc::new(MapScope::new("global", None));

        FlFunction::put_in_scope(
            &*global,
            "fl",
            "quote",
            "Return the parameter without evaluation.",
            vec![Parameter::new_from_spec("value", "Any value", "").unwrap()],
            false,
            Box::new(|_scope: &dyn Scope, p: &Vec<Value>| {
                match &p[0] {
                    // TODO: Add specific code if p[0] is a "Paramter", then returned unevaluted wrapped value.
                    x => Ok(x.clone()),
                }
            }),
        );

        scope_filler::add_list_functions(global.as_ref());

        Engine { scope: global }
    }

    pub fn evaluate(&self, expression: Value) -> Result<Value, FlError> {
        expression.evaluate(&self.scope)
    }

    pub fn evaluate_string(&self, code: impl Into<String>) -> Vec<Result<Value, FlError>> {
        let mut results = Vec::new();
        for value_or_error in Parser::parser_from_str(&code.into()) {
            match value_or_error {
                Err(e) => results.push(Err(e)),
                Ok(v) => results.push(self.evaluate(v)),
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

use crate::interpreter::Engine;

    #[test]
    fn evaluate_string_with_two_expressions() {
        let engine = Engine::new();
        let results = engine.evaluate_string("'(1 2 3) (first '(4 5 6))");
        assert_eq!(2, results.len());
        assert_eq!("(1 2 3)", results[0].as_ref().unwrap().to_string());
        assert_eq!("4", results[1].as_ref().unwrap().to_string());
    }
}