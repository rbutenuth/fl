use std::sync::Arc;

use super::position::Position;
use super::token::{Token, Type};

struct ScanError {
    message: Arc<str>
}

#[derive(Clone, PartialEq, Debug)]
struct CharWithPosition {
    position: Position,
    value: char,
}

const NON_SYMBOL_CHARS: &str = "'\"()[] {}:;";
const ILLEGAL_CHARS: &str = "[]{}:"; // reservered for future use

pub struct Scanner {
    position: Position, // The position of the next character we read.
    lookahead: Vec<CharWithPosition>,
    comment: String,
    chars: Box<dyn Iterator<Item = char>>,
}

impl Scanner {
    pub fn from_named_source(
        initial_position: Position,
        chars: Box<dyn Iterator<Item = char>>,
    ) -> Self {
        let mut scanner = Scanner {
            position: initial_position,
            lookahead: Vec::with_capacity(2),
            comment: String::new(),
            chars: chars,
        };
        for _ in 1..=2 {
            match scanner.read_char() {
                Some(ch) => {
                    scanner.lookahead.push(ch);
                }
                None => {}
            }
        }
        // In first line of code, # is a comment character, allowing "#!/usr/bin/fpl"
        // as first line for starting an interpreter on Unix like operating systems.
        if !scanner.eof() && scanner.lookahead[0].value == '#' {
            scanner.skip_rest_of_line();
        }
        scanner
    }

    pub fn from_anonymous_source(chars: Box<dyn Iterator<Item = char>>) -> Self {
        Self::from_named_source(Position::anonymous_start(), chars)
    }

    pub fn from_str(source: &str) -> Self {
        let chars: Vec<char> = source.chars().collect();
        Scanner::from_anonymous_source(Box::new(chars.into_iter()))
    }

    fn eof(&self) -> bool {
        self.lookahead.is_empty()
    }

    fn move_one_char(&mut self) {
        if !self.eof() {
            self.lookahead.remove(0);
            match self.read_char() {
                Some(ch) => {
                    self.lookahead.push(ch);
                }
                None => {}
            };
        }
    }

    fn read_char(&mut self) -> Option<CharWithPosition> {
        match self.chars.next() {
            Some(ch) => {
                let result = Some(CharWithPosition {
                    position: self.position.clone(),
                    value: ch,
                });
                self.position = match ch {
                    '\n' => Position {
                        name: self.position.name.clone(),
                        line: self.position.line + 1,
                        column: 1,
                    },
                    '\r' => Position {
                        name: self.position.name.clone(),
                        line: self.position.line,
                        column: 1,
                    },
                    _ => Position {
                        name: self.position.name.clone(),
                        line: self.position.line,
                        column: self.position.column + 1,
                    },
                };
                result
            }
            None => None,
        }
    }

    // only call this when you have checked !self.eof()
    fn current_char(&self) -> char {
        self.lookahead[0].value
    }

    fn current_position(&mut self) -> Position {
        self.lookahead[0].position.clone()
    }

    fn current_char_is(&mut self, ch: char) -> bool {
        !self.eof() && self.current_char() == ch
    }

    fn current_char_is_whitespace(&mut self) -> bool {
        !self.eof() && self.current_char().is_whitespace()
    }

    fn char_is_digit(&mut self) -> bool {
        !self.eof() && self.lookahead[0].value >= '0' && self.lookahead[0].value <= '9'
    }

    fn next_char_is_digit(&mut self) -> bool {
        self.lookahead.len() >= 2 && self.lookahead[1].value >= '0' && self.lookahead[1].value <= '9'
    }

    fn current_char_is_symbol_char(&mut self) -> bool {
        !self.eof() && !NON_SYMBOL_CHARS.contains(self.current_char())
    }

    fn skip_rest_of_line(&mut self) -> String {
        let mut content = String::new();
        while !self.eof() && !self.current_char_is('\n') && !self.current_char_is('\r') {
            content.push(self.current_char());
            self.move_one_char();
        }
        content
    }

    fn skip_comment(&mut self) {
        while self.current_char_is(';') || self.current_char_is_whitespace() {
            if self.current_char_is(';') {
                if self.comment.len() > 0 {
                    self.comment.push('\n');
                }
                self.move_one_char(); // skip ';'
                let comment_line = String::from(self.skip_rest_of_line().trim());
                self.comment += comment_line.as_str();
            } else {
                while !self.eof() && self.current_char_is_whitespace() {
                    self.move_one_char();
                }
            }
        }
    }

    fn symbol(&mut self) -> Token {
        let mut symbol_text = String::new();
        let symbol_pos = self.current_position();
        while self.current_char_is_symbol_char() {
            symbol_text.push(self.current_char());
            self.move_one_char();
        }
        let opt_comment = if self.comment.len() > 0 {
            Some(Arc::from(self.comment.as_str()))
        } else {
            None
        };
        Token::new_with_pos(
            Type::TSymbol {
                value: Arc::from(symbol_text.as_str()),
                comment: opt_comment,
            },
            symbol_pos,
        )
    }

    fn text(&mut self) -> Token {
        let start = self.current_position();
        self.move_one_char(); // skip leading "

        let mut text = String::new();
        while !self.eof() && self.current_char() != '"' {
            let ch = self.current_char();
            if ch == '\\' {
                self.move_one_char();
                if self.eof() {
                    return Token::new_error(
                        &Arc::from("\\ at end of input"),
                        self.position.clone(),
                    );
                }
                let decoded = self.decode_escape_sequence();
                match decoded {
                    Ok(ch) => text.push(ch),
                    Err(scan_error) => {
                        return Token::new_error(
                            &scan_error.message,
                            self.position.clone(),
                        );
                    }
                }
            } else {
                text.push(self.current_char());
                self.move_one_char();
            }
        }
        if self.eof() {
            return Token::new_error("Unterminated string at end of input", start);
        }
        self.move_one_char(); // skip trailing "
        Token::new_with_pos(Type::Text { value: Arc::from(text.as_str()) }, start)
    }

    fn number(&mut self) -> Token {
        let start = self.current_position();
        let negative = if self.current_char_is('-') {
            self.move_one_char();
            true
        } else {
            false
        };
        let mut value: i64 = 0;
        while self.char_is_digit() {
            value = 10 * value + (self.current_char() as i64) - ('0' as i64);
            self.move_one_char()
        }
        if self.current_char_is('.') || self.current_char_is('e') || self.current_char_is('E') {
            let mut d_value = value as f64;
            if self.current_char_is('.') {
               self.move_one_char();
                let mut base = 0.1;
                while self.char_is_digit() {
                    d_value += base * (self.current_char() as i64 - '0' as i64) as f64;
                    base /= 10.0;
                    self.move_one_char();
                }
            }
            let mut negative_exponent = false;
            if self.current_char_is('e') || self.current_char_is('E') {
                self.move_one_char();
                if self.current_char_is('+') {
                    self.move_one_char();
                } else if self.current_char_is('-') {
                    negative_exponent = true;
                    self.move_one_char();
                }
                let mut exp_value: i32 = 0;
                while self.char_is_digit() {
                    exp_value = 10 * exp_value + (self.current_char() as i32) - ('0' as i32);
                    self.move_one_char()
                }
                d_value *= (10 as f64).powi(if negative_exponent { -exp_value } else { exp_value });
            }
            Token::new_with_pos(Type::Float { value: if negative {-d_value} else {d_value} }, start)
        } else {
            Token::new_with_pos(Type::Integer { value: if negative {-value} else {value} }, start)
        }
    }

    // Current position is on the \ character. Try to decode the character/sequence following.
    fn decode_escape_sequence(&mut self) -> Result<char, ScanError> {
        let esc = self.current_char();
        self.move_one_char();
        match esc {
            '"' => Ok('"'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            'u' => self.read_hex_sequence(4),
            'v' => self.read_hex_sequence(8),
            _ => Ok(esc),
        }
    }

    fn read_hex_sequence(&mut self, number_of_hex_digits: usize) -> Result<char, ScanError> {
       let mut result: u32 = 0;
        for _ in 1..=number_of_hex_digits {
            result <<= 4;
            if self.eof() {
                return Err(ScanError {
                    message: Arc::from("Unterminated string at end of input"),
                });
            }
            let ch = self.lookahead[0].value.to_ascii_lowercase();
            match self.parse_hex_digit(ch) {
                Ok(value) => result += value,
                Err(e) => return Err(e),
            }
            self.move_one_char();
        }
        match char::from_u32(result) {
            Some(ch) => Ok(ch),
            None => Err(ScanError {
                message: Arc::from("Illegal unicode value sequence"),
            }),
        }
    }

    fn parse_hex_digit(&self, ch: char) -> Result<u32, ScanError> {
        Ok((match ch {
            '0'..='9' => ch as u8 - '0' as u8,
            'a'..='f' => ch as u8 - 'a' as u8 + 10,
            'A'..='F' => ch as u8 - 'A' as u8 + 10,
            _ => {
                return Err(ScanError {
                    message: format!("Illegal character '{}' in unicode hex sequence", ch).into(),
                });
            }
        }) as u32)
    }
}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_comment();
        if self.eof() {
            None
        } else {
            let position = self.current_position();
            Some(match self.current_char() {
                '(' => { self.move_one_char(); Token::new_with_pos(Type::LeftParen, position) },
                ')' => { self.move_one_char(); Token::new_with_pos(Type::RightParen, position) },
                '\'' => { self.move_one_char(); Token::new_with_pos(Type::Quote, position) },
                '0'..='9' => { self.number() },
                '-' if self.next_char_is_digit() => { self.number() },
                ch if !NON_SYMBOL_CHARS.contains(ch) => self.symbol(),
                '"' => self.text(),
                ch if ILLEGAL_CHARS.contains(ch) => {
                    self.move_one_char(); Token::new_error(format!("illegal character: {}", ch).as_str(), position)
                }
                _ => {
                    todo!()
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_and_collect(source: &str) -> String {
        //let chars: Vec<char> = source.chars().collect();
        Scanner::from_str(source)
            .map(|token| token.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }

    #[test]
    fn test_empty_source() {
        let mut sc = Scanner::from_str("");
        let next = sc.next();
        assert!(next.is_none());
    }

    #[test]
    fn test_position() {
        let mut sc = Scanner::from_str("12\n45");
        assert_eq!('1', sc.lookahead[0].value);
        assert_eq!(Position::anonymous(1, 1), sc.current_position());
        sc.move_one_char();

        assert_eq!('2', sc.lookahead[0].value);
        assert_eq!(Position::anonymous(1, 2), sc.current_position());
        sc.move_one_char();

        assert_eq!('\n', sc.lookahead[0].value);
        assert_eq!(Position::anonymous(1, 3), sc.current_position());

        sc.move_one_char();
        assert_eq!('4', sc.lookahead[0].value);
        assert_eq!(Position::anonymous(2, 1), sc.current_position());

        sc.move_one_char();
        assert_eq!('5', sc.lookahead[0].value);
        assert_eq!(Position::anonymous(2, 2), sc.current_position());

        sc.move_one_char();
        assert!(sc.eof());
    }

    #[test]
    fn test_first_line_with_shebang() {
        let mut sc = Scanner::from_str("#!/bin/fpl\ntest");
        let next = sc.next().unwrap();
        assert_eq!(String::from("test"), next.to_string());
        assert_eq!(Position::anonymous(2, 1), next.position.unwrap());
    }

    #[test]
    fn test_left_paren_with_position_check() {
        let mut sc = Scanner::from_str("(");
        let next = sc.next().unwrap();
        assert_eq!(Position::anonymous_start(), next.position.unwrap());
    }

    #[test]
    fn test_left_paren() {
        assert_eq!(String::from("("), scan_and_collect("("));
    }

    #[test]
    fn test_right_paren() {
        assert_eq!(String::from(")"), scan_and_collect(")"));
    }

    #[test]
    fn test_quote() {
        assert_eq!(String::from(")"), scan_and_collect(")"));
    }

    #[test]
    fn test_symbol() {
        assert_eq!(String::from("foo,bar"), scan_and_collect("foo bar"));
    }

    #[test]
    fn test_symbol_with_comment() {
        assert_eq!(
            String::from("symbol"),
            scan_and_collect("symbol;some comment")
        );
    }

    #[test]
    fn test_simple_text() {
        assert_eq!(String::from("\"foo\""), scan_and_collect("\"foo\""));
    }

    #[test]
    fn test_text_with_escapes() {
        assert_eq!(String::from("\"a\"bc\ndef\nhij\""), scan_and_collect("\"a\\\"bc\ndef\\nhij\""));
        assert_eq!(String::from("\"a\tb\rc\n\""), scan_and_collect("\"a\\tb\\rc\\n\""));
        assert_eq!(String::from("\"\u{12ab}\""), scan_and_collect("\"\\u12ab\""));
    }

    #[test]
    fn test_incomplete_hex_sequence() {
        // two digits are not enough
        let mut sc = Scanner::from_str("\"\\u12\"");
        let next = sc.next().unwrap();
        assert_eq!(Position::anonymous(1, 7), next.position.unwrap());
        assert_eq!(Type::NonScanable { message: Arc::from("Illegal character '\"' in unicode hex sequence") }, next.t_type);
        // Just a different way to check for error:
        if let Type::NonScanable {message} = next.t_type {
            assert_eq!(Arc::from("Illegal character '\"' in unicode hex sequence"), message)
        } else {
            panic!("expected error")
        }
    }

    #[test]
    fn test_out_of_range_unicode_value() {
        // Out-of-Range Values (> U+10FFFF): The Unicode standard only defines code points up to
        // U+10FFFF. Any 32-bit value higher than this is invalid.
        let mut sc = Scanner::from_str("\"\\v00110000\"");
        let next = sc.next().unwrap();
        assert_eq!(Position::anonymous(1, 13), next.position.unwrap());
        assert_eq!(Type::NonScanable { message: Arc::from("Illegal unicode value sequence".to_string()) }, next.t_type);
    }

    #[test]
    fn test_unterminated_string() {
        let mut sc = Scanner::from_str(" \"string without end");
        let next = sc.next().unwrap();
        assert_eq!(Position::anonymous(1, 2), next.position.unwrap());
        assert_eq!(Type::NonScanable { message: Arc::from("Unterminated string at end of input".to_string()) }, next.t_type);
    }

    #[test]
    fn test_comment_at_end_of_file() {
        let mut sc = Scanner::from_str("foo \n; bar");
        let next = sc.next().unwrap();
        assert_eq!(Position::anonymous(1, 1), next.position.unwrap());
        assert_eq!(Type::TSymbol { value: Arc::from("foo"), comment: Option::None }, next.t_type);
    }

    #[test]
    fn test_symbol_starts_with_minus() {
        let mut sc = Scanner::from_str("-foo");
        let next = sc.next().unwrap();
        assert_eq!(Position::anonymous(1, 1), next.position.unwrap());
        assert_eq!(Type::TSymbol { value: Arc::from("-foo"), comment: Option::None }, next.t_type);
    }

    #[test]
    fn test_integer() {
        let mut sc = Scanner::from_str("42");
        let next = sc.next().unwrap();
        assert_eq!(Position::anonymous(1, 1), next.position.unwrap());
        assert_eq!(Type::Integer { value: 42 }, next.t_type);
    }

    #[test]
    fn test_numbers() {
        assert_eq!(String::from("42,0.5,3.14,2.78,-7"), scan_and_collect("42 0.5 314e-2 0.0278E+2 -7"));
    }
}
