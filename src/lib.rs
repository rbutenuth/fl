use std::{error::Error, fmt, sync::Arc };
//use std::{error::Error, fmt, sync::Arc};

use list::FlList;

mod list;
mod parser;


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
    Error(Arc<str>) // TODO: nested Error, stack trace
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
            Self::Error(m) => Self::Error(m.clone())
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
            (Value::Error(s), Value::Error(o)) => s == o,
            _ => false,
        }
    }
}


#[derive(Debug)]
pub struct FplError{
    message: String,
    // TODO: Add information about source position and source (RUST for cause in Java)
}

impl fmt::Display for FplError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TODO {}", self.message)
    }
}

impl Error for FplError {
    // TODO: implement chaining
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl FplError {
    pub fn new(message: String) -> FplError {
        FplError{ message: message }
    }
}
