use std::{error::Error, fmt};

pub mod interpreter;
pub mod list;
pub mod parser;

pub use interpreter::value::{Value, ValueType, ValueTypes};

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

    #[test]
    fn error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FlError>();
    }
}
