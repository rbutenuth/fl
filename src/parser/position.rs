use std::{fmt::{Display, Formatter, Result}, sync::Arc};

#[derive(Clone, PartialEq, Debug)]
pub struct Position {
    pub name: Arc<String>,
    pub line: u32,
    pub column: u32,
}

impl Position {
    pub fn new(name: Arc<String>, line: u32, column: u32) -> Self {
        Position { name: name, line, column }
    }

    pub fn anonymous_start() -> Self {
        Position { name: Arc::new(String::from("<anonymous>")), line: 1, column: 1 }
    }

    pub fn anonymous(line: u32, column: u32) -> Self {
        Position { name: Arc::new(String::from("<anonymous>")), line, column }
    }

}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}:{}", self.name, self.line, self.column)
    }
}

