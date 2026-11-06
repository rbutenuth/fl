use std::sync::Arc;

use crate::interpreter::scope::MapScope;
use crate::interpreter::function::Sample;
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
            vec![Sample::new("(quote (1 2 3))", "(1 2 3)")]
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
                Ok(v) => results.push(self.evaluate(v)),
                e => results.push(e),
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use crate::Value;
    use crate::interpreter::Engine;

    #[test]
    fn evaluate_string_with_two_expressions() {
        let engine = Engine::new();
        let results = engine.evaluate_string("'(1 2 3) (first '(4 5 6))");
        assert_eq!(2, results.len());
        assert_eq!("(1 2 3)", results[0].as_ref().unwrap().to_string());
        assert_eq!("4", results[1].as_ref().unwrap().to_string());
    }

    /// Get all entries from the global scope. For all entries containing a functions,
    /// iterate over the samples. Evaluate the expression, check it returns one result
    /// which is not Err, convert the value to string and compare it with yields in
    /// the sample entry.
    #[test]
    fn run_and_check_samples_from_all_functions() {
        let engine = Engine::new();
        let mut failures = Vec::new();
        let mut sample_count = 0;

        for entry in engine.scope.entries() {
            let Value::Function(function) = &entry.value else {
                continue;
            };
            let name = format!("{}::{}", function.get_package(), function.get_name());
            for sample in function.get_samples() {
                sample_count += 1;
                let mut results = engine.evaluate_string(sample.expression.as_str());
                if results.len() != 1 {
                    failures.push(format!(
                        "{}: \"{}\" returned {} results instead of one",
                        name, sample.expression, results.len()
                    ));
                    continue;
                }
                match results.remove(0) {
                    Ok(value) if value.to_string() == sample.yields => {}
                    Ok(value) => failures.push(format!(
                        "{}: \"{}\" yields \"{}\", expected \"{}\"",
                        name, sample.expression, value, sample.yields
                    )),
                    Err(e) => failures.push(format!(
                        "{}: \"{}\" failed: {:?}",
                        name, sample.expression, e
                    )),
                }
            }
        }

        assert!(sample_count > 0, "no samples found in any function");
        assert!(failures.is_empty(), "{} of {} samples failed:\n{}",
            failures.len(), sample_count, failures.join("\n"));
    }
}