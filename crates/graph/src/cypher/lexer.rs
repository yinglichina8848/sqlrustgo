use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CypherToken {
    Match,
    Return,
    Where,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Colon,
    Dash,
    Arrow,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    Equals,
    NotEquals,
    Comma,
    Dot,
    Identifier(String),
    StringLiteral(String),
    Integer(i64),
    Float(f64),
    And,
    Or,
    Not,
    Eof,
}

impl fmt::Display for CypherToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CypherToken::Match => write!(f, "MATCH"),
            CypherToken::Return => write!(f, "RETURN"),
            CypherToken::Where => write!(f, "WHERE"),
            CypherToken::LParen => write!(f, "("),
            CypherToken::RParen => write!(f, ")"),
            CypherToken::LBracket => write!(f, "["),
            CypherToken::RBracket => write!(f, "]"),
            CypherToken::Colon => write!(f, ":"),
            CypherToken::Dash => write!(f, "-"),
            CypherToken::Arrow => write!(f, "->"),
            CypherToken::Greater => write!(f, ">"),
            CypherToken::GreaterEq => write!(f, ">="),
            CypherToken::Less => write!(f, "<"),
            CypherToken::LessEq => write!(f, "<="),
            CypherToken::Equals => write!(f, "="),
            CypherToken::NotEquals => write!(f, "<>"),
            CypherToken::Comma => write!(f, ","),
            CypherToken::Dot => write!(f, "."),
            CypherToken::Identifier(s) => write!(f, "{}", s),
            CypherToken::StringLiteral(s) => write!(f, "\"{}\"", s),
            CypherToken::Integer(i) => write!(f, "{}", i),
            CypherToken::Float(fl) => write!(f, "{}", fl),
            CypherToken::And => write!(f, "AND"),
            CypherToken::Or => write!(f, "OR"),
            CypherToken::Not => write!(f, "NOT"),
            CypherToken::Eof => write!(f, "EOF"),
        }
    }
}

pub struct CypherLexer {
    input: Vec<char>,
    position: usize,
}

impl CypherLexer {
    pub fn new(input: &str) -> Self {
        CypherLexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> CypherToken {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return CypherToken::Eof;
        }

        let ch = self.peek();

        match ch {
            '(' => {
                self.advance();
                CypherToken::LParen
            }
            ')' => {
                self.advance();
                CypherToken::RParen
            }
            '[' => {
                self.advance();
                CypherToken::LBracket
            }
            ']' => {
                self.advance();
                CypherToken::RBracket
            }
            ':' => {
                self.advance();
                CypherToken::Colon
            }
            ',' => {
                self.advance();
                CypherToken::Comma
            }
            '.' => {
                self.advance();
                CypherToken::Dot
            }
            '-' => {
                self.advance();
                if self.peek() == '>' {
                    self.advance();
                    CypherToken::Arrow
                } else {
                    CypherToken::Dash
                }
            }
            '>' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    CypherToken::GreaterEq
                } else {
                    CypherToken::Greater
                }
            }
            '<' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    CypherToken::LessEq
                } else if self.peek() == '>' {
                    self.advance();
                    CypherToken::NotEquals
                } else {
                    CypherToken::Less
                }
            }
            '=' => {
                self.advance();
                CypherToken::Equals
            }
            '"' | '\'' => self.read_string(),
            c if c.is_ascii_digit() => self.read_number(),
            c if c.is_alphabetic() || c == '_' => self.read_identifier(),
            _ => {
                self.advance();
                CypherToken::Eof
            }
        }
    }

    fn peek(&self) -> char {
        self.input.get(self.position).copied().unwrap_or('\0')
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.peek().is_whitespace() {
            self.advance();
        }
    }

    fn read_identifier(&mut self) -> CypherToken {
        let mut result = String::new();
        while self.position < self.input.len() {
            let ch = self.peek();
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        match result.to_uppercase().as_str() {
            "MATCH" => CypherToken::Match,
            "RETURN" => CypherToken::Return,
            "WHERE" => CypherToken::Where,
            "AND" => CypherToken::And,
            "OR" => CypherToken::Or,
            "NOT" => CypherToken::Not,
            _ => CypherToken::Identifier(result),
        }
    }

    fn read_string(&mut self) -> CypherToken {
        let quote = self.peek();
        self.advance();
        let mut result = String::new();
        while self.position < self.input.len() && self.peek() != quote {
            result.push(self.peek());
            self.advance();
        }
        if self.position < self.input.len() {
            self.advance();
        }
        CypherToken::StringLiteral(result)
    }

    fn read_number(&mut self) -> CypherToken {
        let mut result = String::new();
        let mut has_dot = false;
        while self.position < self.input.len() {
            let ch = self.peek();
            if ch.is_ascii_digit() {
                result.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if has_dot {
            result
                .parse()
                .map(CypherToken::Float)
                .unwrap_or(CypherToken::Eof)
        } else {
            result
                .parse()
                .map(CypherToken::Integer)
                .unwrap_or(CypherToken::Eof)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_match() {
        let mut lexer = CypherLexer::new("MATCH (n) RETURN n");
        assert_eq!(lexer.next_token(), CypherToken::Match);
        assert_eq!(lexer.next_token(), CypherToken::LParen);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("n".to_string()));
        assert_eq!(lexer.next_token(), CypherToken::RParen);
        assert_eq!(lexer.next_token(), CypherToken::Return);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("n".to_string()));
        assert_eq!(lexer.next_token(), CypherToken::Eof);
    }

    #[test]
    fn test_tokenize_relationship() {
        let mut lexer = CypherLexer::new("MATCH (n)-[:REL]->(m)");
        assert_eq!(lexer.next_token(), CypherToken::Match);
        assert_eq!(lexer.next_token(), CypherToken::LParen);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("n".to_string()));
        assert_eq!(lexer.next_token(), CypherToken::RParen);
        assert_eq!(lexer.next_token(), CypherToken::Dash);
        assert_eq!(lexer.next_token(), CypherToken::LBracket);
        assert_eq!(lexer.next_token(), CypherToken::Colon);
        assert_eq!(
            lexer.next_token(),
            CypherToken::Identifier("REL".to_string())
        );
        assert_eq!(lexer.next_token(), CypherToken::RBracket);
        assert_eq!(lexer.next_token(), CypherToken::Arrow);
        assert_eq!(lexer.next_token(), CypherToken::LParen);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("m".to_string()));
        assert_eq!(lexer.next_token(), CypherToken::RParen);
        assert_eq!(lexer.next_token(), CypherToken::Eof);
    }

    #[test]
    fn test_tokenize_where() {
        let mut lexer = CypherLexer::new("WHERE n.age > 30");
        assert_eq!(lexer.next_token(), CypherToken::Where);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("n".to_string()));
        assert_eq!(lexer.next_token(), CypherToken::Dot);
        assert_eq!(
            lexer.next_token(),
            CypherToken::Identifier("age".to_string())
        );
        assert_eq!(lexer.next_token(), CypherToken::Greater);
        assert_eq!(lexer.next_token(), CypherToken::Integer(30));
        assert_eq!(lexer.next_token(), CypherToken::Eof);
    }

    #[test]
    fn test_tokenize_string_literal() {
        let mut lexer = CypherLexer::new("\"Alice\"");
        assert_eq!(
            lexer.next_token(),
            CypherToken::StringLiteral("Alice".to_string())
        );
        assert_eq!(lexer.next_token(), CypherToken::Eof);
    }

    #[test]
    fn test_tokenize_integer() {
        let mut lexer = CypherLexer::new("42");
        assert_eq!(lexer.next_token(), CypherToken::Integer(42));
        assert_eq!(lexer.next_token(), CypherToken::Eof);
    }

    #[test]
    fn test_tokenize_float() {
        let mut lexer = CypherLexer::new("3.14");
        assert_eq!(lexer.next_token(), CypherToken::Float(3.14));
        assert_eq!(lexer.next_token(), CypherToken::Eof);
    }

    #[test]
    fn test_tokenize_label() {
        let mut lexer = CypherLexer::new("(n:User)");
        assert_eq!(lexer.next_token(), CypherToken::LParen);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("n".to_string()));
        assert_eq!(lexer.next_token(), CypherToken::Colon);
        assert_eq!(
            lexer.next_token(),
            CypherToken::Identifier("User".to_string())
        );
        assert_eq!(lexer.next_token(), CypherToken::RParen);
        assert_eq!(lexer.next_token(), CypherToken::Eof);
    }

    #[test]
    fn test_tokenize_not_equals() {
        let mut lexer = CypherLexer::new("<>");
        assert_eq!(lexer.next_token(), CypherToken::NotEquals);
        assert_eq!(lexer.next_token(), CypherToken::Eof);
    }
}
