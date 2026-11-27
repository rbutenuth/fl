use std::format;
use std::sync::Mutex;
use std::{fmt, sync::Arc};

use crate::FlError;
use crate::interpreter::lambda::ParameterPlaceholder;
use crate::interpreter::{FlFunction, Scope, Symbol};
use crate::list::FlList;

pub struct LazyContent {
    pub scope: Arc<dyn Scope>,
    lazy: Value,
    evaluated: Option<Result<Value, FlError>>,
}

pub struct FlLazy {
    pub content: Mutex<LazyContent>,
}

impl fmt::Debug for FlLazy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlLazy").finish_non_exhaustive()
    }
}

impl FlLazy {
    pub fn new(lazy: Value, scope: Arc<dyn Scope>) -> Arc<FlLazy> {
        match lazy {
            Value::Lazy(l) => l.clone(),
            _ => Arc::new(FlLazy {
                content: Mutex::new(LazyContent {
                    scope,
                    lazy,
                    evaluated: Option::None,
                }),
            }),
        }
    }

    pub fn evaluate(&self) -> Result<Value, FlError> {
        let mut content = self.content.lock().expect("mutex is poisoned");
        match &content.evaluated {
            Some(e) => e.clone(),
            None => {
                let result = content.lazy.evaluate(&content.scope.clone());
                content.evaluated = Some(result.clone());
                result
            }
        }
    }

    pub fn original_expression(&self) -> Value {
        let content = self.content.lock().expect("mutex is poisoned");
        content.lazy.clone()
    }

}

#[derive(Debug)]
pub enum Value {
    Nil,
    Integer(i64),
    Float(f64),
    List(FlList),
    Symbol(Symbol),
    Text(Arc<str>),
    Map(),    // TODO: implement
    Object(), // TODO: implement
    Function(Arc<FlFunction>),
    Lazy(Arc<FlLazy>),
    Parameter(Arc<ParameterPlaceholder>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Nil => Self::Nil,
            Self::Integer(i) => Self::Integer(*i),
            Self::Float(f) => Self::Float(*f),
            Self::Symbol(s) => Self::Symbol(s.clone()),
            Self::Text(s) => Self::Text(s.clone()),
            Self::List(list) => Self::List(list.clone()),
            Self::Map() => Self::Map(),
            Self::Object() => Self::Object(),
            Self::Function(f) => Self::Function(f.clone()),
            Self::Lazy(l) => Self::Lazy(l.clone()),
            Self::Parameter(p) => Self::Parameter(p.clone()),
        }
    }
}

/// The variants of [`Value`] without their payload. Use it to name a variant
/// (e.g. as a function argument) without having a value of that variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Nil,
    Integer,
    Float,
    List,
    Symbol,
    Text,
    Map,
    Object,
    Function,
}

impl ValueType {
    /// All types, in declaration order.
    pub const ALL: [ValueType; 9] = [
        ValueType::Nil,
        ValueType::Integer,
        ValueType::Float,
        ValueType::List,
        ValueType::Symbol,
        ValueType::Text,
        ValueType::Map,
        ValueType::Object,
        ValueType::Function,
    ];
}

// ValueTypes stores one bit per type in a u16.
const _: () = assert!(ValueType::Function as u16 <= 15);

/// A set of [`ValueType`]s, stored as a bit set in one integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueTypes(u16);

impl ValueTypes {
    pub const NONE: ValueTypes = ValueTypes(0);
    pub const ALL: ValueTypes = ValueTypes((1 << (ValueType::Function as u16 + 1)) - 1);
    /// Integer or Float.
    pub const NUMBER: ValueTypes =
        ValueTypes::single(ValueType::Integer).union(ValueTypes::single(ValueType::Float));

    pub const fn single(vt: ValueType) -> ValueTypes {
        ValueTypes(1 << vt as u16)
    }

    pub const fn union(self, other: ValueTypes) -> ValueTypes {
        ValueTypes(self.0 | other.0)
    }

    pub const fn contains(self, vt: ValueType) -> bool {
        self.0 & (1 << vt as u16) != 0
    }
}

impl From<ValueType> for ValueTypes {
    fn from(vt: ValueType) -> ValueTypes {
        ValueTypes::single(vt)
    }
}

/// `ValueType::Integer | ValueType::Float`
impl std::ops::BitOr for ValueType {
    type Output = ValueTypes;

    fn bitor(self, other: ValueType) -> ValueTypes {
        ValueTypes::single(self).union(ValueTypes::single(other))
    }
}

/// `ValueTypes::NUMBER | ValueType::Text`
impl std::ops::BitOr<ValueType> for ValueTypes {
    type Output = ValueTypes;

    fn bitor(self, other: ValueType) -> ValueTypes {
        self.union(ValueTypes::single(other))
    }
}

impl std::ops::BitOr for ValueTypes {
    type Output = ValueTypes;

    fn bitor(self, other: ValueTypes) -> ValueTypes {
        self.union(other)
    }
}

/// Lists the types, e.g. "Integer or Float".
impl fmt::Display for ValueTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for vt in ValueType::ALL.iter().filter(|k| self.contains(**k)) {
            if !first {
                write!(f, " or ")?;
            }
            write!(f, "{:?}", vt)?;
            first = false;
        }
        if first {
            write!(f, "nothing")?;
        }
        Ok(())
    }
}

/// Does only work for "simple" types: Nil, Integer, Float, Text, Symbol, Function.
/// All others are always considered as not equal.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Integer(s), Value::Integer(o)) => s == o,
            (Value::Float(s), Value::Float(o)) => s == o,
            (Value::Text(s), Value::Text(o)) => s == o,
            (Value::Symbol(s), Value::Symbol(o)) => s == o,
            (Value::Function(s), Value::Function(o)) => Arc::ptr_eq(s, o),
            _ => false,
        }
    }
}

impl Value {
    pub fn evaluates_to_self(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Integer(_) => true,
            Value::Float(_) => true,
            Value::List(_) => false,
            Value::Symbol(_) => false,
            Value::Text(_) => true,
            Value::Map() => todo!(),
            Value::Object() => todo!(),
            Value::Function(_) => true,
            Value::Lazy(_) => false,
            Value::Parameter(_) => false,
        }
    }
    pub fn fl_equals(&self, other: &Self) -> Result<Value, FlError> {
        match (self, other) {
            (Value::Nil, Value::Nil) => Ok(Value::Integer(1)),
            (Value::Integer(left), Value::Integer(right)) => {
                Ok(Value::Integer(if left == right { 1 } else { 0 }))
            }
            (Value::Float(left), Value::Float(right)) => Ok(Value::Integer(if left == right { 1 } else { 0 })),
            (Value::Text(left), Value::Text(right)) => Ok(Value::Integer(if left == right { 1 } else { 0 })),
            (Value::Symbol(left), Value::Symbol(right)) => Ok(Value::Integer(if left == right { 1 } else { 0 })),
            (Value::Function(left), Value::Function(right)) => {
                Ok(Value::Integer(if Arc::ptr_eq(left, right) { 1 } else { 0 }))
            }
            (Value::Lazy(left), right) => left.evaluate()?.fl_equals(right),
            (left, Value::Lazy(right)) => left.fl_equals(&right.evaluate()?),
// TODO            (Value::Parameter(left), right) => left.evaluate()?.fl_equals(right),
// TODO            (left, Value::Parameter(right)) => left.fl_equals(&right.evaluate()?),
            // TODO: List, etc.
            _ => Ok(Value::Integer(0)),
        }
    }

    pub fn as_value_type(&self) -> Result<ValueType, FlError> {
        match self {
            Value::Nil => Ok(ValueType::Nil),
            Value::Integer(_) => Ok(ValueType::Integer),
            Value::Float(_) => Ok(ValueType::Float),
            Value::List(_) => Ok(ValueType::List),
            Value::Symbol(..) => Ok(ValueType::Symbol),
            Value::Text(_) => Ok(ValueType::Text),
            Value::Map() => Ok(ValueType::Map),
            Value::Object() => Ok(ValueType::Object),
            Value::Function(_) => Ok(ValueType::Function),
            Value::Lazy(l) => l.evaluate()?.as_value_type(),
            Value::Parameter(_p) => todo!(), // no scope available... p.evaluate()?.as_value_type(),
        }
    }

    pub fn as_integer(&self) -> Result<i64, FlError> {
        match self {
            Value::Integer(i) => Ok(*i),
            _ => Err(FlError::new(format!("Not an integer: {:?}", self))),
        }
    }

    pub fn as_float(&self) -> Result<f64, FlError> {
        match self {
            Value::Float(f) => Ok(*f),
            _ => Err(FlError::new(format!("Not a float: {:?}", self))),
        }
    }

    pub fn as_list(&self) -> Result<FlList, FlError> {
        match self {
            Value::List(l) => Ok(l.clone()),
            _ => Err(FlError::new(format!("Not a list: {:?}", self))),
        }
    }

    pub fn as_symbol(&self) -> Result<Symbol, FlError> {
        match self {
            Value::Symbol(s) => Ok(s.clone()),
            _ => Err(FlError::new(format!("Not a symbol: {:?}", self))),
        }
    }

    pub fn as_text(&self) -> Result<Arc<str>, FlError> {
        match self {
            Value::Text(t) => Ok(t.clone()),
            _ => Err(FlError::new(format!("Not a string: {:?}", self))),
        }
    }

    // TODO: as_map, as_object

    pub fn as_function(&self) -> Result<Arc<FlFunction>, FlError> {
        match self {
            Value::Function(f) => Ok(f.clone()),
            _ => Err(FlError::new(format!("Not a function: {:?}", self))),
        }
    }

    pub fn evaluate(&self, scope: &Arc<dyn Scope>) -> Result<Value, FlError> {
        match self {
            Value::List(list) => Self::evaluate_list(scope, list),
            Value::Symbol(s) => Ok(scope.get(s.name())),
            Value::Map() => todo!(),
            Value::Object() => todo!(),
            Value::Parameter(p) => p.evaluate(scope),
            // Nil, Integer, Float, Text, Function: just clone
            _ => Ok(self.clone()),
        }
    }

    /// If list is empty: FlError, empty list can't be evaluated.
    /// Otherwise: Evaluate first element. If it is not a Function: FlError.
    /// Call the function with the remaining list elements as parameters.
    /// But only if the number of parameters matches. It can be larger then
    /// the specified number in case the function is variable parameter list capable.
    fn evaluate_list(scope: &Arc<dyn Scope>, list: &FlList) -> Result<Value, FlError> {
        let mut elements = list.into_iter();
        let first = elements
            .next()
            .ok_or_else(|| FlError::new("Can't evaluate an empty list"))?;

        let position = match first.clone() {
            Value::Symbol(s) => s.position().cloned(),
            _ => None,
        };

        match first.evaluate(scope)? {
            Value::Function(function) => function
                .evaluate_arguments_and_call(scope, elements.collect())
                .map_err(|error| match position {
                    Some(position) if error.position().is_none() => error.with_position(position),
                    // The error already has a position (from a deeper call): add this call
                    // as a new error and keep the inner one as source.
                    Some(position) => {
                        FlError::with_source(error.message.clone(), error).with_position(position)
                    }
                    None => error,
                }),
            other => {
                let error = FlError::new(format!(
                    "First element of a list must evaluate to a Function, but got {:?}",
                    other.as_value_type()
                ));
                Err(match position {
                    Some(position) => error.with_position(position),
                    None => error,
                })
            }
        }
    }

    fn fmt_list(f: &mut fmt::Formatter<'_>, list: &FlList) -> fmt::Result {
        write!(f, "(")?;
        for (i, element) in list.into_iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", element)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil")?,
            Value::Integer(i) => write!(f, "{}", i)?,
            Value::Float(d) => write!(f, "{}", d)?,
            Value::List(list) => Value::fmt_list(f, list)?,
            Value::Symbol(symbol) => write!(f, "{}", symbol.name())?,
            Value::Text(text) => write!(f, "\"{}\"", text)?,
            Value::Map() => todo!(),
            Value::Object() => todo!(),
            Value::Function(_function) => todo!(),
            Value::Lazy(_l) => todo!(),
            Value::Parameter(_p) => todo!(),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;
    use crate::interpreter::function::Parameter;
    use crate::interpreter::scope::MapScope;

    fn symbol(name: &str) -> Value {
        Value::Symbol(Symbol::new(name, None, None))
    }

    #[test]
    fn value_types_contains() {
        assert!(ValueTypes::NUMBER.contains(ValueType::Integer));
        assert!(ValueTypes::NUMBER.contains(ValueType::Float));
        assert!(!ValueTypes::NUMBER.contains(ValueType::Text));
        assert!(!ValueTypes::NONE.contains(ValueType::Nil));
        for vt in ValueType::ALL {
            assert!(ValueTypes::ALL.contains(vt));
            assert!(ValueTypes::single(vt).contains(vt));
        }
    }

    #[test]
    fn value_types_combine() {
        let text_or_symbol = ValueType::Text | ValueType::Symbol;
        assert!(text_or_symbol.contains(ValueType::Text));
        assert!(text_or_symbol.contains(ValueType::Symbol));
        assert!(!text_or_symbol.contains(ValueType::Integer));
        assert_eq!(ValueTypes::NUMBER, ValueType::Integer | ValueType::Float);
        assert!((ValueTypes::NUMBER | ValueType::Text).contains(ValueType::Text));
        assert_eq!(ValueTypes::ALL, ValueTypes::NUMBER | ValueTypes::ALL);
    }

    #[test]
    fn value_typess_display() {
        assert_eq!("Integer or Float", ValueTypes::NUMBER.to_string());
        assert_eq!("Text", ValueTypes::single(ValueType::Text).to_string());
        assert_eq!("nothing", ValueTypes::NONE.to_string());
    }

    #[test]
    fn type_names_the_variant() {
        assert_eq!(Ok(ValueType::Nil), Value::Nil.as_value_type());
        assert_eq!(Ok(ValueType::Integer), Value::Integer(1).as_value_type());
        assert_eq!(Ok(ValueType::Float), Value::Float(1.0).as_value_type());
        assert_eq!(
            Ok(ValueType::List),
            Value::List(FlList::empty()).as_value_type()
        );
        assert_eq!(Ok(ValueType::Symbol), symbol("a").as_value_type());
        assert_eq!(
            Ok(ValueType::Text),
            Value::Text(Arc::from("t")).as_value_type()
        );
        assert_eq!(Ok(ValueType::Map), Value::Map().as_value_type());
        assert_eq!(Ok(ValueType::Object), Value::Object().as_value_type());
        let function = FlFunction::new(
            Position::anonymous_start(),
            "p",
            "f",
            "",
            vec![],
            false,
            Box::new(|_scope: &Arc<dyn Scope>, _parameters: &Vec<Value>| Ok(Value::Nil)),
            Vec::new(),
        );
        assert_eq!(
            Ok(ValueType::Function),
            Value::Function(Arc::new(function)).as_value_type()
        );
    }

    #[test]
    fn evaluate_symbol_looks_up_value() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new(None));
        let _ = scope.put("a", Value::Integer(42));

        assert_eq!(Ok(Value::Integer(42)), symbol("a").evaluate(&scope));
    }

    #[test]
    fn evaluate_unbound_symbol_is_nil() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new(None));

        assert_eq!(Ok(Value::Nil), symbol("unknown").evaluate(&scope));
    }

    #[test]
    fn evaluate_symbol_walks_scope_chain() {
        let global: Arc<dyn Scope> = Arc::new(MapScope::new(None));
        let _ = global.put("a", Value::Integer(1));
        let _ = global.put("b", Value::Integer(2));
        let local: Arc<dyn Scope> = Arc::new(MapScope::new(Some(Arc::clone(&global))));
        let _ = local.put("b", Value::Integer(3));

        assert_eq!(Ok(Value::Integer(1)), symbol("a").evaluate(&local));
        assert_eq!(Ok(Value::Integer(3)), symbol("b").evaluate(&local));
    }

    #[test]
    fn evaluate_symbol_ignores_comment() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new(None));
        let _ = scope.put("a", Value::Text(Arc::from("hello")));
        let commented = Value::Symbol(Symbol::new("a", Some("a comment"), None));

        assert_eq!(
            Ok(Value::Text(Arc::from("hello"))),
            commented.evaluate(&scope)
        );
    }

    fn global_with_functions() -> Arc<dyn Scope> {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new(None));
        // Eager, adds two integers.
        let add = FlFunction::new(
            Position::anonymous_start(),
            "package",
            "add",
            "adds two integers",
            vec![
                Parameter::new("a", "", ValueType::Integer, true),
                Parameter::new("b", "", ValueType::Integer, true),
            ],
            false,
            Box::new(
                |_scope: &Arc<dyn Scope>, p: &Vec<Value>| match (&p[0], &p[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                    _ => Err(FlError::new("not integers")),
                },
            ),
            Vec::new(),
        );
        // Not eager, returns its argument unevaluated.
        let quote = FlFunction::new(
            Position::anonymous_start(),
            "package",
            "quote",
            "returns its argument unevaluated",
            vec![Parameter::new("a", "", ValueTypes::ALL, false)],
            false,
            Box::new(|_scope: &Arc<dyn Scope>, p: &Vec<Value>| Ok(p[0].clone())),
            Vec::new(),
        );
        let _ = scope.put("add", Value::Function(Arc::new(add)));
        let _ = scope.put("quote", Value::Function(Arc::new(quote)));
        scope
    }

    fn list(values: Vec<Value>) -> Value {
        Value::List(FlList::from_vec_values(values))
    }

    #[test]
    fn evaluate_list_calls_function_with_evaluated_parameters() {
        let scope = global_with_functions();
        let _ = scope.put("x", Value::Integer(40));

        let call = list(vec![symbol("add"), symbol("x"), Value::Integer(2)]);

        assert_eq!(Ok(Value::Integer(42)), call.evaluate(&scope));
    }

    #[test]
    fn evaluate_list_evaluates_nested_lists() {
        let scope = global_with_functions();
        let inner = list(vec![symbol("add"), Value::Integer(1), Value::Integer(2)]);

        let call = list(vec![symbol("add"), inner, Value::Integer(4)]);

        assert_eq!(Ok(Value::Integer(7)), call.evaluate(&scope));
    }

    #[test]
    fn evaluate_list_does_not_evaluate_non_eager_parameters() {
        let scope = global_with_functions();
        let _ = scope.put("x", Value::Integer(40));

        let call = list(vec![symbol("quote"), symbol("x")]);

        assert_eq!(Ok(symbol("x")), call.evaluate(&scope));
    }

    #[test]
    fn evaluate_empty_list_is_error() {
        let scope = global_with_functions();

        assert!(Value::List(FlList::empty()).evaluate(&scope).is_err());
    }

    #[test]
    fn evaluate_list_with_non_function_first_is_error() {
        let scope = global_with_functions();

        assert!(
            list(vec![Value::Integer(1), Value::Integer(2)])
                .evaluate(&scope)
                .is_err()
        );
        assert!(list(vec![symbol("unbound")]).evaluate(&scope).is_err());
    }

    #[test]
    fn evaluate_list_with_wrong_parameter_type_is_error() {
        let scope = global_with_functions();

        let call = list(vec![
            symbol("add"),
            Value::Text(Arc::from("a")),
            Value::Integer(2),
        ]);

        assert!(call.evaluate(&scope).is_err());
    }

    #[test]
    fn value_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Value>();
        assert_send_sync::<ValueTypes>();
    }
}
