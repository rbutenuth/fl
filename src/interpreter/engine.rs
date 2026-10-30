use std::sync::Arc;

use crate::{FlError, Value};
use crate::list::scope_filler;
use crate::interpreter::{FlFunction, Scope, Parameter};
use crate::interpreter::scope::MapScope;

pub struct Engine {
    pub scope: Arc<dyn Scope>,
}

impl Engine {
    pub fn new() -> Engine {
        let global = Arc::new(MapScope::new("global", None));
        
        FlFunction::put_in_scope(&*global, 
        "fl",
        "quote",
        "Return the parameter without evaluation.",
        vec![
            Parameter::new_from_spec("value", "Any value", "").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            match &p[0] {
                // TODO: Add specific code if p[0] is a "Paramter", then returned unevaluted wrapped value.
                x => Ok(x.clone()),
            }
        }));

        scope_filler::add_list_functions(global.as_ref());
        
        Engine {
            scope: global
        }
    }

    pub fn evaluate(&self, expression: Value) -> Result<Value, FlError> {
        expression.evaluate(&self.scope)
    }
}

