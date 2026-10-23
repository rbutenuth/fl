//use crate::{FlError, Value};

use std::fmt;

use crate::{FlError, Scope, Value};

pub trait Callable: Send + Sync {
    fn call(&self, scope: &dyn Scope, parameters: &Vec<Value>) -> Result<Value, FlError>;
}

pub struct Parameter {
    name: String,
    comment: String,
}

impl Parameter {
    pub fn new(name: String, comment: String) -> Self {
        Parameter { name, comment }
    }
}

pub struct FlFunction {
    name: String,
    comment: String,
    parameters: Vec<Parameter>,
    is_vararg: bool,
    callable: Box<dyn Callable>,
}

impl fmt::Display for FlFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "name: {}", self.name)?;
        writeln!(f, "comment: {}", self.comment)?;
        writeln!(f, "is_vararg: {}", self.is_vararg)?;
        writeln!(f, "parameters:")?;
        for p in &self.parameters {
            writeln!(f, " {}", p.name)?;
            writeln!(f, "  {}", p.comment)?;
        }
        Ok(())
    }
}

impl FlFunction {
    pub fn call(&self, scope: &dyn Scope, parameters: Vec<Value>) -> Result<Value, FlError> {
        let expected = self.parameters.len();
        let given = parameters.len();
        if given == expected {
            self.callable.call(scope, &parameters)
        } else if given < expected {
            Err(FlError::new("todo: Currying"))
        } else {
            if self.is_vararg {
                self.callable.call(scope, &parameters)
            } else {
                Err(FlError::new(format!(
                    "Function {} expects {} paramaters, but is called with {}",
                    self.name, expected, given
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flfunction_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FlFunction>();
    }
}
