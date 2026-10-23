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

impl ValueKind {
    /// All kinds, in declaration order.
    pub const ALL: [ValueKind; 9] = [
        ValueKind::Nil,
        ValueKind::Integer,
        ValueKind::Float,
        ValueKind::List,
        ValueKind::Symbol,
        ValueKind::Text,
        ValueKind::Map,
        ValueKind::Object,
        ValueKind::Function,
    ];
}

// ValueKinds stores one bit per kind in a u16.
const _: () = assert!(ValueKind::Function as u16 <= 15);

/// A set of [`ValueKind`]s, stored as a bit set in one integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueKinds(u16);

impl ValueKinds {
    pub const NONE: ValueKinds = ValueKinds(0);
    pub const ALL: ValueKinds = ValueKinds((1 << (ValueKind::Function as u16 + 1)) - 1);
    /// Integer or Float.
    pub const NUMBER: ValueKinds =
        ValueKinds::single(ValueKind::Integer).union(ValueKinds::single(ValueKind::Float));

    pub const fn single(kind: ValueKind) -> ValueKinds {
        ValueKinds(1 << kind as u16)
    }

    pub const fn union(self, other: ValueKinds) -> ValueKinds {
        ValueKinds(self.0 | other.0)
    }

    pub const fn contains(self, kind: ValueKind) -> bool {
        self.0 & (1 << kind as u16) != 0
    }
}

impl From<ValueKind> for ValueKinds {
    fn from(kind: ValueKind) -> ValueKinds {
        ValueKinds::single(kind)
    }
}

/// `ValueKind::Integer | ValueKind::Float`
impl std::ops::BitOr for ValueKind {
    type Output = ValueKinds;

    fn bitor(self, other: ValueKind) -> ValueKinds {
        ValueKinds::single(self).union(ValueKinds::single(other))
    }
}

/// `ValueKinds::NUMBER | ValueKind::Text`
impl std::ops::BitOr<ValueKind> for ValueKinds {
    type Output = ValueKinds;

    fn bitor(self, other: ValueKind) -> ValueKinds {
        self.union(ValueKinds::single(other))
    }
}

impl std::ops::BitOr for ValueKinds {
    type Output = ValueKinds;

    fn bitor(self, other: ValueKinds) -> ValueKinds {
        self.union(other)
    }
}

/// Lists the kinds, e.g. "Integer or Float".
impl fmt::Display for ValueKinds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for kind in ValueKind::ALL.iter().filter(|k| self.contains(**k)) {
            if !first {
                write!(f, " or ")?;
            }
            write!(f, "{:?}", kind)?;
            first = false;
        }
        if first {
            write!(f, "nothing")?;
        }
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::scope::MapScope;

    fn symbol(name: &str) -> Value {
        Value::Symbol(Arc::from(name), None)
    }

    #[test]
    fn value_kinds_contains() {
        assert!(ValueKinds::NUMBER.contains(ValueKind::Integer));
        assert!(ValueKinds::NUMBER.contains(ValueKind::Float));
        assert!(!ValueKinds::NUMBER.contains(ValueKind::Text));
        assert!(!ValueKinds::NONE.contains(ValueKind::Nil));
        for kind in ValueKind::ALL {
            assert!(ValueKinds::ALL.contains(kind));
            assert!(ValueKinds::single(kind).contains(kind));
        }
    }

    #[test]
    fn value_kinds_combine() {
        let text_or_symbol = ValueKind::Text | ValueKind::Symbol;
        assert!(text_or_symbol.contains(ValueKind::Text));
        assert!(text_or_symbol.contains(ValueKind::Symbol));
        assert!(!text_or_symbol.contains(ValueKind::Integer));
        assert_eq!(ValueKinds::NUMBER, ValueKind::Integer | ValueKind::Float);
        assert!((ValueKinds::NUMBER | ValueKind::Text).contains(ValueKind::Text));
        assert_eq!(ValueKinds::ALL, ValueKinds::NUMBER | ValueKinds::ALL);
    }

    #[test]
    fn value_kinds_display() {
        assert_eq!("Integer or Float", ValueKinds::NUMBER.to_string());
        assert_eq!("Text", ValueKinds::single(ValueKind::Text).to_string());
        assert_eq!("nothing", ValueKinds::NONE.to_string());
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
        assert_send_sync::<ValueKinds>();
    }
}
