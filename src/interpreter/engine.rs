use std::sync::Arc;

use crate::Value;
use crate::interpreter::Scope;
use crate::interpreter::scope::MapScope;

pub struct Engine {
    scope: Arc<dyn Scope>,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            scope: Arc::new(MapScope::new("global", None))
        }
    }

    pub fn evaluate(&self, expression: Value) -> Value {
        expression.evaluate(self.scope.as_ref())
    }
}

