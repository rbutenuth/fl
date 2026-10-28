use std::sync::Arc;

use crate::{FlError, Value};
use crate::interpreter::Scope;
use crate::interpreter::scope::MapScope;

pub struct Engine {
    pub scope: Arc<dyn Scope>,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            scope: Arc::new(MapScope::new("global", None))
        }
    }

    pub fn evaluate(&self, expression: Value) -> Result<Value, FlError> {
        expression.evaluate(&self.scope)
    }
}

