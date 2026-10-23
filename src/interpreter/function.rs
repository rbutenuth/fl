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
    pub fn new(name: impl Into<String>, comment: impl Into<String>) -> Self {
        Parameter {
            name: name.into(),
            comment: comment.into(),
        }
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
    pub fn new(
        name: impl Into<String>,
        comment: impl Into<String>,
        parameters: Vec<Parameter>,
        is_vararg: bool,
        callable: Box<dyn Callable>,
    ) -> Self {
        FlFunction {
            name: name.into(),
            comment: comment.into(),
            parameters,
            is_vararg,
            callable,
        }
    }

    pub fn call(&self, scope: &dyn Scope, parameters: Vec<Value>) -> Result<Value, FlError> {
        let expected = self.parameters.len();
        let given = parameters.len();
        if given == expected {
            self.callable.call(scope, &parameters)
        } else if given < expected {
            Err(FlError::new("TODO: Currying"))
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
    use crate::interpreter::scope::MapScope;

    struct First;

    impl Callable for First {
        fn call(&self, _scope: &dyn Scope, parameters: &Vec<Value>) -> Result<Value, FlError> {
            Ok(parameters[0].clone())
        }
    }

    fn function(is_vararg: bool) -> FlFunction {
        FlFunction::new(
            "first",
            "returns first parameter",
            vec![Parameter::new("a", "the value")],
            is_vararg,
            Box::new(First),
        )
    }

    #[test]
    fn new_and_call() {
        let scope = MapScope::new("global", None);

        let result = function(false).call(&scope, vec![Value::Integer(7)]);

        assert_eq!(Ok(Value::Integer(7)), result);
    }

    #[test]
    fn too_many_parameters_without_vararg_is_error() {
        let scope = MapScope::new("global", None);

        let result = function(false).call(&scope, vec![Value::Integer(1), Value::Integer(2)]);

        assert!(result.is_err());
    }

    #[test]
    fn too_many_parameters_with_vararg_is_ok() {
        let scope = MapScope::new("global", None);

        let result = function(true).call(&scope, vec![Value::Integer(1), Value::Integer(2)]);

        assert_eq!(Ok(Value::Integer(1)), result);
    }

    #[test]
    fn flfunction_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FlFunction>();
    }
}
