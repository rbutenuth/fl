use std::sync::Arc;

use crate::{
    FlError, Value::self, list::FlList, parser::{
        scanner::Scanner,
        token::{Token, Type::*},
    },
};

pub struct Parser {
    scanner: Scanner,
    next_token: Option<Token>,
}

impl Iterator for Parser {
    type Item = Result<Value, FlError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token.take() {
            None => None,
            Some(token) => Some(self.value(token)),
        }
    }
}

impl Parser {
    pub fn from_scanner(mut scanner: Scanner) -> Parser {
        let t = scanner.next();
        Parser {
            scanner: scanner,
            next_token: t,
        }
    }

    pub fn value(&mut self, token: Token) -> Result<Value, FlError> {
        match token.t_type {
            LeftParen => self.list(),
            Quote => todo!(),
            Integer { value } => Ok(self.integer(value)),
            Float { value } => Ok(self.float(value)),
            Symbol { value, comment } => Ok(self.symbol(value, comment)),
            Text { value } => Ok(self.text(value)),
            NonScanable { message } => Err(FlError::new_arc(message)), // TODO: Position
            _ => Err(FlError::new("unexpected )")), // TODO: Position
        }
    }

    pub fn parser_from_str(str: &str) -> Parser {
        Parser::from_scanner(Scanner::from_str(str))
    }

    fn integer(&mut self, i: i64) -> Value {
        self.fetch_next_token();
        Value::Integer(i)
    }

    fn float(&mut self, f: f64) -> Value {
        self.fetch_next_token();
        Value::Float(f)
    }

    fn symbol(&mut self, symbol: Arc<str>, comment: Option<Arc<str>>) -> Value {
        self.fetch_next_token();
        Value::Symbol(symbol, comment)
    }

    fn text(&mut self, text: Arc<str>) -> Value {
        self.fetch_next_token();
        Value::Text(Arc::from(text))
    }

    fn list(&mut self) -> Result<Value, FlError> {
        self.fetch_next_token(); // skip (

        let mut elements: Vec<Value> = Vec::new();

        loop {
            match &self.next_token {
                None => return Err(FlError::new("Unexpected end of source in list")), // TODO: position
                Some(token) if matches!(token.t_type, RightParen) => break,
                Some(_) => {
                    let token = self.next_token.take().unwrap();
                    elements.push(self.value(token)?);
                }
            }
        }

        self.fetch_next_token(); // skip )
        Ok(Value::List(FlList::from_values(elements)))
    }

    fn fetch_next_token(&mut self) {
        self.next_token = self.scanner.next();
    }
}

#[cfg(test)]
mod tests {
    use crate::{Value, FlError};
    use crate::parser::parser::Parser;
    use std::sync::Arc;

    #[test]
    fn test_empty_source() {
        let mut p = Parser::parser_from_str("");
        let next = p.next();
        assert!(next.is_none());
    }

    #[test]
    fn test_integer_constant() {
        let mut p = Parser::parser_from_str("42");
        assert_eq!(Value::Integer(42), p.next().unwrap().unwrap());
    }

    #[test]
    fn test_float_constant() {
        let mut p = Parser::parser_from_str("3.14");
        assert_eq!(Value::Float(3.14), p.next().unwrap().unwrap());
    }

    #[test]
    fn test_text_constant() {
        let mut p = Parser::parser_from_str("\"a text\"");
        assert_eq!(Value::Text(Arc::from("a text")), p.next().unwrap().unwrap());
    }

    #[test]
    fn test_symbol_with_comment() {
        let mut p = Parser::parser_from_str("; bla fasel\nfoo");
        let symbol = p.next().unwrap().unwrap();
        // Comment is not considered for equality of symbols
        assert_eq!(Value::Symbol(Arc::from("foo"), None), symbol);
        match symbol {
            Value::Symbol(name, comment) => {
                assert_eq!("foo", &*name);
                assert_eq!(Some("bla fasel"), comment.as_deref());
            }
            _ => panic!("symbol expected"),
        }
    }

    #[test]
    fn test_unexpected_right_paren() {
        let mut p = Parser::parser_from_str(")");
        assert_eq!(
            Err(FlError::new("unexpected )")),
            p.next().unwrap()
        );
    }

    #[test]
    fn test_empty_list() {
        let mut p = Parser::parser_from_str("()");
        let value = p.next().unwrap();
        match value {
            Ok(Value::List(list)) => assert_eq!(0, list.len()),
            _ => panic!("list expected"),
        }
    }
}

/*
    @Test
    public void quote() throws Exception {
        Parser p = parser("quote", "'('symbol 42 3.1415 \"a string\")");
        assertTrue(p.hasNext());
        FplList l = (FplList) p.next();
        assertEquals(2, l.size());
        assertEquals("(quote ((quote symbol) 42 3.1415 \"a string\"))", l.toString());
    }

*/
