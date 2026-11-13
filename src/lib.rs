use std::{error::Error, fmt, sync::Arc};

pub mod interpreter;
pub mod list;
pub mod parser;
pub mod stdlib;

pub use interpreter::value::{Value, ValueType, ValueTypes};
pub use parser::Position;

#[derive(Debug, Clone)]
pub struct FlError {
    message: String,
    /// Application defined error id, 0 when not set.
    id: i64,
    position: Option<Arc<Position>>,
    /// The underlying cause. `Arc` keeps `FlError` cloneable.
    source: Option<Arc<dyn Error + Send + Sync + 'static>>,
}

/// Two errors are equal when their messages and ids are equal. Cause and position are
/// ignored (`dyn Error` has no `PartialEq`).
impl PartialEq for FlError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message && self.id == other.id
    }
}

impl fmt::Display for FlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(position) = &self.position {
            write!(f, "{}: ", position)?;
        }
        write!(f, "{}", self.message)
    }
}

impl Error for FlError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|e| e as &(dyn Error + 'static))
    }
}

impl FlError {
    /// Accepts `&str` (one allocation) or `String` (moved in, no copy).
    pub fn new(message: impl Into<String>) -> FlError {
        Self::new_with_id(message, 0)
    }

    /// Creates an error with an application defined id.
    pub fn new_with_id(message: impl Into<String>, id: i64) -> FlError {
        FlError {
            message: message.into(),
            id,
            source: None,
            position: None,
        }
    }

    /// Creates an error with an underlying cause, available via `Error::source()`.
    pub fn with_source(
        message: impl Into<String>,
        source: impl Error + Send + Sync + 'static,
    ) -> FlError {
        Self::with_source_and_id(message, source, 0)
    }

    /// Creates an error with an underlying cause and an application defined id.
    pub fn with_source_and_id(
        message: impl Into<String>,
        source: impl Error + Send + Sync + 'static,
        id: i64,
    ) -> FlError {
        FlError {
            message: message.into(),
            id,
            source: Some(Arc::new(source)),
            position: None,
        }
    }

    /// Sets the source position (builder style). `None` leaves the error without position.
    pub fn with_position(mut self, position: Position) -> FlError {
        self.position = Some(Arc::new(position));
        self
    }

    /// The error message, without position.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The error id, 0 if none was given.
    pub fn id(&self) -> i64 {
        self.id
    }

    /// The source position where the error happened, if known.
    pub fn position(&self) -> Option<&Position> {
        self.position.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FlError>();
    }

    #[test]
    fn id_defaults_to_zero_and_is_kept_by_clone_and_position() {
        assert_eq!(0, FlError::new("x").id());
        let err = FlError::new_with_id("x", 42).with_position(Position::anonymous(1, 1));
        assert_eq!(42, err.id());
        assert_eq!(42, err.clone().id());
        assert_eq!(0, FlError::with_source("x", FlError::new("c")).id());
        let with_cause = FlError::with_source_and_id("x", FlError::new("c"), 7);
        assert_eq!(7, with_cause.id());
        assert!(with_cause.source().is_some());
        // The id takes part in equality, position does not.
        assert_ne!(err, FlError::new("x"));
        assert_eq!(err, FlError::new_with_id("x", 42));
    }

    #[test]
    fn position_is_optional() {
        let err = FlError::new("boom");
        assert!(err.position().is_none());

        let err = err.with_position(Position::anonymous(3, 7));
        assert_eq!(Some(&Position::anonymous(3, 7)), err.position());
        assert_eq!(Some(&Position::anonymous(3, 7)), err.clone().position());
        // Position does not take part in equality.
        assert_eq!(err, FlError::new("boom"));
    }

    #[test]
    fn display_includes_position_as_prefix() {
        let err = FlError::new("boom");
        assert_eq!("boom", err.to_string());
        let err = err.with_position(Position::anonymous(3, 7));
        assert_eq!("<anonymous>:3:7: boom", err.to_string());
    }

    #[test]
    fn source_is_none_by_default() {
        assert!(FlError::new("x").source().is_none());
    }

    #[test]
    fn source_is_chained() {
        let cause = FlError::new("cause");
        let err = FlError::with_source("outer", cause.clone());
        let source = err.source().expect("source expected");
        assert_eq!(source.to_string(), cause.to_string());
        assert!(source.source().is_none());
        // Clone keeps the cause, equality only looks at the message.
        assert!(err.clone().source().is_some());
        assert_eq!(err, FlError::new("outer"));
    }
}
