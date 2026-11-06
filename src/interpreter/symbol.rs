use std::sync::Arc;

use crate::parser::Position;

/// The data of a symbol. Lives behind one `Arc`, so cloning a `Symbol` costs
/// a single reference count increment, no matter how many fields are added.
#[derive(Debug)]
struct SymbolData {
    name: Box<str>,
    comment: Option<Box<str>>,
    position: Option<Position>,
}

#[derive(Debug, Clone)]
pub struct Symbol(Arc<SymbolData>);

/// Equality check only considers name, not comment and position.
impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        // Same allocation: skip the string comparison.
        Arc::ptr_eq(&self.0, &other.0) || self.0.name == other.0.name
    }
}

impl Symbol {
    pub fn new(name: &str, comment: Option<&str>, position: Option<Position>) -> Symbol {
        Symbol(Arc::new(SymbolData {
            name: Box::from(name),
            comment: comment.map(Box::from),
            position,
        }))
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn comment(&self) -> Option<&str> {
        self.0.comment.as_deref()
    }

    /// Where the symbol was found in the source, if known.
    pub fn position(&self) -> Option<&Position> {
        self.0.position.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors() {
        let s = Symbol::new("a", Some("comment"), Some(Position::anonymous(2, 3)));
        assert_eq!("a", s.name());
        assert_eq!(Some("comment"), s.comment());
        assert_eq!(Some(&Position::anonymous(2, 3)), s.position());

        let s = Symbol::new("b", None, None);
        assert_eq!(None, s.comment());
        assert_eq!(None, s.position());
    }

    #[test]
    fn equality_only_looks_at_name() {
        let a = Symbol::new("a", Some("comment"), Some(Position::anonymous(1, 1)));
        let b = Symbol::new("a", None, None);
        assert_eq!(a, b);
        assert_ne!(a, Symbol::new("c", None, None));
    }

    #[test]
    fn clone_shares_data() {
        let a = Symbol::new("a", None, None);
        let b = a.clone();
        assert!(Arc::ptr_eq(&a.0, &b.0));
        assert_eq!(a, b);
    }

    #[test]
    fn symbol_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Symbol>();
    }
}
