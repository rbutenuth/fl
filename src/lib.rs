use std::{error::Error, fmt, sync::Arc };

use list::FlList;
use interpreter::Scope;

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
    Map(), // TODO: implement
    Object(), // TODO: implement
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

impl Value {
    pub fn evaluate(&self, scope: &Arc<dyn Scope>) -> Value {
        println!("evaluating {:?}", self);
        match self {
            Value::Nil => Value::Nil,
            Value::Integer(_) => self.clone(),
            Value::Float(_) => self.clone(),
            Value::List(fl_list) => todo!(),
            Value::Symbol(name, _) => scope.get(name),
            Value::Text(_) => self.clone(),
            Value::Map() => todo!(),
            Value::Object() => todo!(),
            Value::Function() => todo!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FlError{
    message: Arc<str>,
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

    pub fn new(message: &str) -> FlError {
        Self::new_arc(Arc::from(message))
    }

    pub fn new_arc(message: Arc<str>) -> FlError {
        FlError{ message: Arc::clone(&message) }
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
    fn evaluate_symbol_looks_up_value() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new("global", None));
        scope.put("a", Value::Integer(42));

        assert_eq!(Value::Integer(42), symbol("a").evaluate(&scope));
    }

    #[test]
    fn evaluate_unbound_symbol_is_nil() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new("global", None));

        assert_eq!(Value::Nil, symbol("unknown").evaluate(&scope));
    }

    #[test]
    fn evaluate_symbol_walks_scope_chain() {
        let global: Arc<dyn Scope> = Arc::new(MapScope::new("global", None));
        global.put("a", Value::Integer(1));
        global.put("b", Value::Integer(2));
        let local: Arc<dyn Scope> = Arc::new(MapScope::new("local", Some(Arc::clone(&global))));
        local.put("b", Value::Integer(3));

        assert_eq!(Value::Integer(1), symbol("a").evaluate(&local));
        assert_eq!(Value::Integer(3), symbol("b").evaluate(&local));
    }

    #[test]
    fn evaluate_symbol_ignores_comment() {
        let scope: Arc<dyn Scope> = Arc::new(MapScope::new("global", None));
        scope.put("a", Value::Text(Arc::from("hello")));
        let commented = Value::Symbol(Arc::from("a"), Some(Arc::from("a comment")));

        assert_eq!(Value::Text(Arc::from("hello")), commented.evaluate(&scope));
    }
}
