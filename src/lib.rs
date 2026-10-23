use std::{error::Error, fmt, sync::Arc};

use interpreter::Scope;
use list::FlList;

pub mod interpreter;
pub mod list;
pub mod parser;

#[derive(Debug)]
pub enum Value {
    Nil,
    Integer(i64),
    Float(f64),
    List(FlList),
    Symbol(Arc<str>, Option<Arc<str>>), // name, comment
    Text(Arc<str>),
    Map(),      // TODO: implement
    Object(),   // TODO: implement
    Function(), // TODO: implement
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Nil => Self::Nil,
            Self::Integer(i) => Self::Integer(*i),
            Self::Float(f) => Self::Float(*f),
            Self::Symbol(n, c) => Self::Symbol(n.clone(), c.clone()),
            Self::Text(s) => Self::Text(s.clone()),
            Self::List(list) => Self::List(list.clone()),
            Self::Map() => Self::Map(),
            Self::Object() => Self::Object(),
            Self::Function() => Self::Function(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Integer(s), Value::Integer(o)) => s == o,
            (Value::Float(s), Value::Float(o)) => s == o,
            (Value::Text(s), Value::Text(o)) => s == o,
            (Value::Symbol(s, _), Value::Symbol(o, _)) => s == o,
            _ => false,
        }
    }
}

/// The variants of [`Value`] without their payload. Use it to name a variant
/// (e.g. as a function argument) without having a value of that variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
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

impl Value {
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Nil => ValueKind::Nil,
            Value::Integer(_) => ValueKind::Integer,
            Value::Float(_) => ValueKind::Float,
            Value::List(_) => ValueKind::List,
            Value::Symbol(..) => ValueKind::Symbol,
            Value::Text(_) => ValueKind::Text,
            Value::Map() => ValueKind::Map,
            Value::Object() => ValueKind::Object,
            Value::Function() => ValueKind::Function,
        }
    }

    pub fn evaluate(&self, scope: &Arc<dyn Scope>) -> Result<Value, FlError> {
        println!("evaluating {:?}", self);
        match self {
            Value::List(_fl_list) => todo!(),
            Value::Symbol(name, _) => Ok(scope.get(name)),
            Value::Map() => todo!(),
            Value::Object() => todo!(),
            // Nil, Integer, Float, Text, Function: just clone
            _ => Ok(self.clone())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlError {
    message: String,
    // TODO: Add information about source position and source (RUST for cause in Java)
}

impl fmt::Display for FlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TODO {}", self.message)
    }
}

impl Error for FlError {
    // TODO: implement chaining
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl FlError {
    /// Accepts `&str` (one allocation) or `String` (moved in, no copy).
    pub fn new(message: impl Into<String>) -> FlError {
        FlError {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use interpreter::scope::MapScope;

    fn symbol(name: &str) -> Value {
        Value::Symbol(Arc::from(name), None)
    }

    #[test]
    fn kind_names_the_variant() {
        assert_eq!(ValueKind::Nil, Value::Nil.kind());
        assert_eq!(ValueKind::Integer, Value::Integer(1).kind());
        assert_eq!(ValueKind::Float, Value::Float(1.0).kind());
        assert_eq!(ValueKind::List, Value::List(FlList::empty()).kind());
        assert_eq!(ValueKind::Symbol, symbol("a").kind());
        assert_eq!(ValueKind::Text, Value::Text(Arc::from("t")).kind());
        assert_eq!(ValueKind::Map, Value::Map().kind());
        assert_eq!(ValueKind::Object, Value::Object().kind());
        assert_eq!(ValueKind::Function, Value::Function().kind());
    }

    #[test]
    fn evaluate_symbol_looks_up_value() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new("global", None));
        scope.put("a", Value::Integer(42));

        assert_eq!(Ok(Value::Integer(42)), symbol("a").evaluate(&scope));
    }

    #[test]
    fn evaluate_unbound_symbol_is_nil() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new("global", None));

        assert_eq!(Ok(Value::Nil), symbol("unknown").evaluate(&scope));
    }

    #[test]
    fn evaluate_symbol_walks_scope_chain() {
        let global: Arc<dyn Scope> = Arc::new(MapScope::new("global", None));
        global.put("a", Value::Integer(1));
        global.put("b", Value::Integer(2));
        let local: Arc<dyn Scope> = Arc::new(MapScope::new("local", Some(Arc::clone(&global))));
        local.put("b", Value::Integer(3));

        assert_eq!(Ok(Value::Integer(1)), symbol("a").evaluate(&local));
        assert_eq!(Ok(Value::Integer(3)), symbol("b").evaluate(&local));
    }

    #[test]
    fn evaluate_symbol_ignores_comment() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new("global", None));
        scope.put("a", Value::Text(Arc::from("hello")));
        let commented = Value::Symbol(Arc::from("a"), Some(Arc::from("a comment")));

        assert_eq!(
            Ok(Value::Text(Arc::from("hello"))),
            commented.evaluate(&scope)
        );
    }

    #[test]
    fn value_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Value>();
        assert_send_sync::<FlError>();
    }
}
