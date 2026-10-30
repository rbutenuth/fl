use std::sync::Arc;

use crate::{FlError, Value};
use crate::list::scope_filler;
use crate::interpreter::Scope;
use crate::interpreter::scope::MapScope;

pub struct Engine {
    pub scope: Arc<dyn Scope>,
}

impl Engine {
    pub fn new() -> Engine {
        let global = Arc::new(MapScope::new("global", None));
        
        scope_filler::add_list_functions(global.as_ref());
        
        Engine {
            scope: global
        }
    }

    pub fn evaluate(&self, expression: Value) -> Result<Value, FlError> {
        expression.evaluate(&self.scope)
    }
}

