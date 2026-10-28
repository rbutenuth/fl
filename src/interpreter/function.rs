//use crate::{FlError, Value};

use std::{fmt, sync::Arc};

use crate::interpreter::Scope;
use crate::{FlError, Value, ValueType, ValueTypes};

pub trait Callable: Send + Sync {
    fn call(&self, scope: &dyn Scope, parameters: &Vec<Value>) -> Result<Value, FlError>;
}

/// Closures (and function pointers) with the right signature are callables.
impl<F> Callable for F
where
    F: Fn(&dyn Scope, &Vec<Value>) -> Result<Value, FlError> + Send + Sync,
{
    fn call(&self, scope: &dyn Scope, parameters: &Vec<Value>) -> Result<Value, FlError> {
        self(scope, parameters)
    }
}

pub struct Parameter {
    name: String,
    comment: String,
    types: ValueTypes,
    eager: bool,
}

impl Parameter {
    /// `types` is the set of value types this parameter accepts, a single
    /// [`ValueType`] or a [`ValueTypes`] (use `ValueTypes::ALL` for "anything").
    pub fn new(
        name: impl Into<String>,
        comment: impl Into<String>,
        types: impl Into<ValueTypes>,
        eager: bool
    ) -> Self {
        Parameter {
            name: name.into(),
            comment: comment.into(),
            types: types.into(),
            eager: eager,
        }
    }

    /// `type_spec` is a String describing the parameter. One letter per type, with a special meaning for `E`
    /// * N: Nil
    /// * I: Integer
    /// * D: Float
    /// * L: List
    /// * S: Symbol
    /// * T: Text
    /// * M: Map
    /// * O: Object
    /// * F: Function
    /// * E: Eager evaluation
    /// In case of no letter provided (except E), all types are allowed.
    /// Letters are case insensitive. An unknown letter is an error.
    pub fn new_from_spec(
        name: impl Into<String>,
        comment: impl Into<String>,
        type_spec: impl AsRef<str>,
    ) -> Result<Self, FlError> {
        let mut types = ValueTypes::NONE;
        let mut eager = false;
        for letter in type_spec.as_ref().chars() {
            let value_type = match letter.to_ascii_uppercase() {
                'N' => ValueType::Nil,
                'I' => ValueType::Integer,
                'D' => ValueType::Float,
                'L' => ValueType::List,
                'S' => ValueType::Symbol,
                'T' => ValueType::Text,
                'M' => ValueType::Map,
                'O' => ValueType::Object,
                'F' => ValueType::Function,
                'E' => {
                    eager = true;
                    continue;
                }
                _ => {
                    return Err(FlError::new(format!(
                        "Unknown letter '{}' in parameter type spec",
                        letter
                    )));
                }
            };
            types = types | value_type;
        }
        if types == ValueTypes::NONE {
            types = ValueTypes::ALL;
        }
        Ok(Parameter::new(name, comment, types, eager))
    }

    fn check(&self, value: &Value) -> Result<(), FlError> {
        if self.types.contains(value.as_value_type()) {
            Ok(())
        } else {
            Err(FlError::new(format!(
                "Parameter {} expects {}, but got {:?}",
                self.name,
                self.types,
                value.as_value_type()
            )))
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
            writeln!(f, "  accepts: {}", p.types)?;
        }
        Ok(())
    }
}

impl fmt::Debug for FlFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FlFunction({})", self.name)
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

    /// Evaluate the eager arguments in `scope`, then call the function.
    /// Arguments of non-eager parameters are passed as they are. Arguments beyond
    /// the declared parameters (vararg) are evaluated when the last parameter 
    /// is eager.
    pub fn evaluate_and_call(
        &self,
        scope: &Arc<dyn Scope>,
        arguments: Vec<Value>,
    ) -> Result<Value, FlError> {
        let mut parameters = Vec::with_capacity(arguments.len());
        for (i, argument) in arguments.into_iter().enumerate() {
            let eager = self
                .parameters
                .get(i)
                .or(self.parameters.last())
                .map_or(false, |p| p.eager);
            parameters.push(if eager { argument.evaluate(scope)? } else { argument });
        }
        self.call(&**scope, parameters)
    }

    pub fn call(&self, scope: &dyn Scope, parameters: Vec<Value>) -> Result<Value, FlError> {
        let expected = self.parameters.len();
        let given = parameters.len();
        // Extra (vararg) parameters beyond the declared ones are not checked.
        for (parameter, value) in self.parameters.iter().zip(&parameters) {
            parameter.check(value)?;
        }
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

/// Evaluate `value` and return the integer it evaluates to. Any other result is an error.
pub fn evaluate_to_integer(scope: &Arc<dyn Scope>, value: Value) -> Result<i64, FlError> {
    match value.evaluate(scope)? {
        Value::Integer(i) => Ok(i),
        other => Err(FlError::new(format!("expected integer, got {:?}", other))),
    }
}

/// Evaluate `value` and return the float it evaluates to. Any other result is an error.
pub fn evaluate_to_float(scope: &Arc<dyn Scope>, value: Value) -> Result<f64, FlError> {
    match value.evaluate(scope)? {
        Value::Float(f) => Ok(f),
        other => Err(FlError::new(format!("expected integer, got {:?}", other))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ValueType;
    use crate::interpreter::scope::MapScope;
    use std::sync::Arc;

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
            vec![Parameter::new("a", "the value", ValueTypes::ALL, true)],
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
    fn closure_as_callable_doubles_numbers() {
        let scope: MapScope = MapScope::new("global", None);
        let multiply_by_2 = FlFunction::new(
            "multiply_by_2",
            "multiplies a number by 2",
            vec![Parameter::new("n", "an integer or a float", ValueTypes::NUMBER, true)],
            false,
            Box::new(|_scope: &dyn Scope, parameters: &Vec<Value>| match &parameters[0] {
                Value::Integer(i) => Ok(Value::Integer(i * 2)),
                Value::Float(f) => Ok(Value::Float(f * 2.0)),
                other => Err(FlError::new(format!(
                    "expected integer or float, got {:?}",
                    other
                ))),
            }),
        );

        assert_eq!(Ok(Value::Integer(42)), multiply_by_2.call(&scope, vec![Value::Integer(21)]));
        assert_eq!(Ok(Value::Float(3.0)), multiply_by_2.call(&scope, vec![Value::Float(1.5)]));
        assert!(multiply_by_2
            .call(&scope, vec![Value::Text(Arc::from("x"))])
            .is_err());
        assert!(multiply_by_2.call(&scope, vec![Value::Nil]).is_err());
    }

    #[test]
    fn evaluate_to_integer_accepts_integers_and_symbols_bound_to_integers() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new("global", None));
        scope.put("a", Value::Integer(5));

        assert_eq!(Ok(7), evaluate_to_integer(&scope, Value::Integer(7)));
        assert_eq!(
            Ok(5),
            evaluate_to_integer(&scope, Value::Symbol(Arc::from("a"), None))
        );
    }

    #[test]
    fn evaluate_to_integer_rejects_other_values() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new("global", None));

        assert!(evaluate_to_integer(&scope, Value::Float(1.0)).is_err());
        assert!(evaluate_to_integer(&scope, Value::Nil).is_err());
        assert!(evaluate_to_integer(&scope, Value::Symbol(Arc::from("unbound"), None)).is_err());
    }

    #[test]
    fn parameter_types_are_checked_before_the_callable_runs() {
        let scope = MapScope::new("global", None);
        let text_only = FlFunction::new(
            "length",
            "never called with wrong types",
            vec![Parameter::new("s", "a text", ValueType::Text, true)],
            false,
            Box::new(|_scope: &dyn Scope, _parameters: &Vec<Value>| -> Result<Value, FlError> {
                Ok(Value::Nil)
            }),
        );

        assert_eq!(Ok(Value::Nil), text_only.call(&scope, vec![Value::Text(Arc::from("x"))]));
        let error = text_only.call(&scope, vec![Value::Integer(1)]).unwrap_err();
        assert_eq!(
            FlError::new("Parameter s expects Text, but got Integer"),
            error
        );
    }

    #[test]
    fn vararg_parameters_beyond_declared_ones_are_not_checked() {
        let scope = MapScope::new("global", None);
        let function = FlFunction::new(
            "f",
            "",
            vec![Parameter::new("n", "a number", ValueTypes::NUMBER, true)],
            true,
            Box::new(|_scope: &dyn Scope, parameters: &Vec<Value>| Ok(parameters[0].clone())),
        );

        let result = function.call(&scope, vec![Value::Integer(1), Value::Nil]);

        assert_eq!(Ok(Value::Integer(1)), result);
    }

    #[test]
    fn new_from_spec_parses_types_and_eager() {
        let p = Parameter::new_from_spec("p", "", "IDE").unwrap();
        assert_eq!(ValueTypes::NUMBER, p.types);
        assert!(p.eager);

        let p = Parameter::new_from_spec("p", "", "NLSTMOF").unwrap();
        assert_eq!(
            ValueType::Nil
                | ValueType::List
                | ValueType::Symbol
                | ValueType::Text
                | ValueType::Map
                | ValueType::Object
                | ValueType::Function,
            p.types
        );
        assert!(!p.eager);
    }

    #[test]
    fn new_from_spec_ignores_case() {
        let p = Parameter::new_from_spec("p", "", "idE").unwrap();
        assert_eq!(ValueTypes::NUMBER, p.types);
        assert!(p.eager);
    }

    #[test]
    fn new_from_spec_without_type_letters_accepts_everything() {
        assert_eq!(ValueTypes::ALL, Parameter::new_from_spec("p", "", "").unwrap().types);

        let p = Parameter::new_from_spec("p", "", "e").unwrap();
        assert_eq!(ValueTypes::ALL, p.types);
        assert!(p.eager);
    }

    #[test]
    fn new_from_spec_rejects_unknown_letters() {
        let error = Parameter::new_from_spec("p", "", "IX").err().unwrap();
        assert_eq!(
            FlError::new("Unknown letter 'X' in parameter type spec"),
            error
        );
    }

    #[test]
    fn flfunction_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FlFunction>();
    }
}
