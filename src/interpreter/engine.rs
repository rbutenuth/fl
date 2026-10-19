use crate::Value;
use crate::interpreter::Scope;

pub struct Engine {
    scope: Scope,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            scope: Scope::new()
        }
    }

    pub fn evaluate(&self, expression: Value) -> Value {
        expression.evaluate(&self.scope)
    }
}

