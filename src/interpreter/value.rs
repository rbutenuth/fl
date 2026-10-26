use std::{fmt, sync::Arc};

use crate::FlError;
use crate::interpreter::Scope;
use crate::list::FlList;

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
            // TODO: Function, List, etc.
            _ => false,
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

impl Value {
    pub fn as_value_type(&self) -> ValueType {
        match self {
            Value::Nil => ValueType::Nil,
            Value::Integer(_) => ValueType::Integer,
            Value::Float(_) => ValueType::Float,
            Value::List(_) => ValueType::List,
            Value::Symbol(..) => ValueType::Symbol,
            Value::Text(_) => ValueType::Text,
            Value::Map() => ValueType::Map,
            Value::Object() => ValueType::Object,
            Value::Function() => ValueType::Function,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::scope::MapScope;

    fn symbol(name: &str) -> Value {
        Value::Symbol(Arc::from(name), None)
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
        assert_eq!(ValueType::Nil, Value::Nil.as_value_type());
        assert_eq!(ValueType::Integer, Value::Integer(1).as_value_type());
        assert_eq!(ValueType::Float, Value::Float(1.0).as_value_type());
        assert_eq!(ValueType::List, Value::List(FlList::empty()).as_value_type());
        assert_eq!(ValueType::Symbol, symbol("a").as_value_type());
        assert_eq!(ValueType::Text, Value::Text(Arc::from("t")).as_value_type());
        assert_eq!(ValueType::Map, Value::Map().as_value_type());
        assert_eq!(ValueType::Object, Value::Object().as_value_type());
        assert_eq!(ValueType::Function, Value::Function().as_value_type());
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
        assert_send_sync::<ValueTypes>();
    }
}
