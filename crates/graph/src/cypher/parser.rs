//! Cypher query parser

use super::lexer::{CypherLexer, CypherToken};
use crate::error::GraphError;

/// Parsed Cypher query
#[derive(Debug, Clone, PartialEq)]
pub struct CypherQuery {
    /// Match pattern
    pub pattern: CypherPattern,
    /// WHERE clause predicate
    pub where_clause: Option<CypherPredicate>,
    /// RETURN clause items
    pub return_items: Vec<ReturnItem>,
}

/// Cypher pattern - node or relationship pattern
#[derive(Debug, Clone, PartialEq)]
pub enum CypherPattern {
    /// Node pattern: (var:Label)
    Node(NodePattern),
    /// Relationship pattern: (a)-[r:REL]->(b)
    Relationship {
        from: Box<NodePattern>,
        to: Box<NodePattern>,
        rel_label: Option<String>,
        rel_vars: Option<String>,
    },
}

/// Node pattern in a Cypher query
#[derive(Debug, Clone, PartialEq)]
pub struct NodePattern {
    /// Variable name (e.g., 'n' in (n:Label))
    pub variable: Option<String>,
    /// Node label (e.g., 'User' in (n:User))
    pub label: Option<String>,
}

/// RETURN clause item
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnItem {
    /// Variable to return
    pub variable: String,
    /// Optional property accessor
    pub property: Option<String>,
}

/// Predicate for WHERE clause
#[derive(Debug, Clone, PartialEq)]
pub enum CypherPredicate {
    /// Property comparison: n.age > 30
    PropertyComparison {
        variable: String,
        property: String,
        operator: ComparisonOp,
        value: Literal,
    },
    /// AND predicate
    And(Box<CypherPredicate>, Box<CypherPredicate>),
    /// OR predicate
    Or(Box<CypherPredicate>, Box<CypherPredicate>),
    /// NOT predicate
    Not(Box<CypherPredicate>),
}

/// Comparison operators
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOp {
    Equals,
    NotEquals,
    Greater,
    Less,
    GreaterEq,
    LessEq,
}

/// Literal value in Cypher
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

/// Cypher query parser
pub struct CypherParser {
    tokens: Vec<CypherToken>,
    position: usize,
}

impl CypherParser {
    /// Create a new parser with the given tokens
    pub fn new(tokens: Vec<CypherToken>) -> Self {
        CypherParser {
            tokens,
            position: 0,
        }
    }

    pub fn parse_query(&mut self) -> Result<CypherQuery, GraphError> {
        let pattern = self.parse_pattern()?;
        let where_clause = self.parse_where_clause().ok();
        self.expect_token(&CypherToken::Return)?;
        let return_items = self.parse_return_items()?;

        Ok(CypherQuery {
            pattern,
            where_clause,
            return_items,
        })
    }

    fn parse_pattern(&mut self) -> Result<CypherPattern, GraphError> {
        self.expect_token(&CypherToken::Match)?;
        self.parse_pattern_body()
    }

    fn parse_pattern_body(&mut self) -> Result<CypherPattern, GraphError> {
        let from = self.parse_node_pattern()?;

        if self.current_token() == &CypherToken::Dash {
            self.advance();
            let rel_label = self.parse_relationship_pattern()?;
            let to = self.parse_node_pattern()?;

            return Ok(CypherPattern::Relationship {
                from: Box::new(from),
                to: Box::new(to),
                rel_label: Some(rel_label),
                rel_vars: None,
            });
        }

        Ok(CypherPattern::Node(from))
    }

    fn parse_node_pattern(&mut self) -> Result<NodePattern, GraphError> {
        self.expect_token(&CypherToken::LParen)?;

        let variable = match self.current_token() {
            CypherToken::Identifier(_) => {
                let var = self.advance_and_get_identifier();
                Some(var)
            }
            _ => None,
        };

        let label = if self.current_token() == &CypherToken::Colon {
            self.advance();
            Some(self.advance_and_get_identifier())
        } else {
            None
        };

        self.expect_token(&CypherToken::RParen)?;

        Ok(NodePattern { variable, label })
    }

    fn parse_relationship_pattern(&mut self) -> Result<String, GraphError> {
        self.expect_token(&CypherToken::LBracket)?;
        self.expect_token(&CypherToken::Colon)?;
        let rel_label = self.advance_and_get_identifier();
        self.expect_token(&CypherToken::RBracket)?;
        self.expect_token(&CypherToken::Arrow)?;
        Ok(rel_label)
    }

    fn parse_where_clause(&mut self) -> Result<CypherPredicate, GraphError> {
        if self.current_token() != &CypherToken::Where {
            return Err(GraphError::ParseError("Expected WHERE".to_string()));
        }
        self.advance();
        self.parse_predicate()
    }

    fn parse_predicate(&mut self) -> Result<CypherPredicate, GraphError> {
        // Handle leading NOT (standard Cypher syntax: NOT condition)
        if self.current_token() == &CypherToken::Not {
            self.advance();
            return Ok(CypherPredicate::Not(Box::new(self.parse_comparison()?)));
        }

        let left = self.parse_comparison()?;

        match self.current_token() {
            CypherToken::And => {
                self.advance();
                let right = self.parse_predicate()?;
                Ok(CypherPredicate::And(Box::new(left), Box::new(right)))
            }
            CypherToken::Or => {
                self.advance();
                let right = self.parse_predicate()?;
                Ok(CypherPredicate::Or(Box::new(left), Box::new(right)))
            }
            CypherToken::Not => {
                self.advance();
                Ok(CypherPredicate::Not(Box::new(self.parse_comparison()?)))
            }
            _ => Ok(left),
        }
    }

    fn parse_comparison(&mut self) -> Result<CypherPredicate, GraphError> {
        let variable = self.advance_and_get_identifier();

        if self.current_token() == &CypherToken::Colon || self.current_token() == &CypherToken::Dot
        {
            self.advance();
            let property = self.advance_and_get_identifier();

            let operator = match self.current_token() {
                CypherToken::Equals => {
                    self.advance();
                    ComparisonOp::Equals
                }
                CypherToken::NotEquals => {
                    self.advance();
                    ComparisonOp::NotEquals
                }
                CypherToken::Greater => {
                    self.advance();
                    ComparisonOp::Greater
                }
                CypherToken::Less => {
                    self.advance();
                    ComparisonOp::Less
                }
                CypherToken::GreaterEq => {
                    self.advance();
                    ComparisonOp::GreaterEq
                }
                CypherToken::LessEq => {
                    self.advance();
                    ComparisonOp::LessEq
                }
                _ => {
                    return Err(GraphError::ParseError(format!(
                        "Expected comparison operator, got {:?}",
                        self.current_token()
                    )));
                }
            };

            let value = self.parse_literal()?;

            Ok(CypherPredicate::PropertyComparison {
                variable,
                property,
                operator,
                value,
            })
        } else {
            Err(GraphError::ParseError(
                "Expected ':' after variable".to_string(),
            ))
        }
    }

    fn parse_literal(&mut self) -> Result<Literal, GraphError> {
        match self.current_token().clone() {
            CypherToken::StringLiteral(s) => {
                self.advance();
                Ok(Literal::String(s))
            }
            CypherToken::Integer(i) => {
                self.advance();
                Ok(Literal::Integer(i))
            }
            CypherToken::Float(fl) => {
                self.advance();
                Ok(Literal::Float(fl))
            }
            CypherToken::Identifier(s) => {
                self.advance();
                match s.to_uppercase().as_str() {
                    "TRUE" => Ok(Literal::Boolean(true)),
                    "FALSE" => Ok(Literal::Boolean(false)),
                    _ => Err(GraphError::ParseError(format!("Unknown identifier: {}", s))),
                }
            }
            _ => Err(GraphError::ParseError(format!(
                "Expected literal, got {:?}",
                self.current_token()
            ))),
        }
    }

    fn parse_return_items(&mut self) -> Result<Vec<ReturnItem>, GraphError> {
        let mut items = Vec::new();

        loop {
            let variable = self.advance_and_get_identifier();
            let property = if self.current_token() == &CypherToken::Colon
                || self.current_token() == &CypherToken::Dot
            {
                self.advance();
                Some(self.advance_and_get_identifier())
            } else {
                None
            };

            items.push(ReturnItem { variable, property });

            if self.current_token() == &CypherToken::Comma {
                self.advance();
            } else {
                break;
            }
        }

        Ok(items)
    }

    fn current_token(&self) -> &CypherToken {
        self.tokens.get(self.position).unwrap_or(&CypherToken::Eof)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn advance_and_get_identifier(&mut self) -> String {
        if let CypherToken::Identifier(s) = self
            .tokens
            .get(self.position)
            .cloned()
            .unwrap_or(CypherToken::Identifier(String::new()))
        {
            self.advance();
            s
        } else {
            String::new()
        }
    }

    fn expect_token(&mut self, expected: &CypherToken) -> Result<(), GraphError> {
        if self.current_token() == expected {
            self.advance();
            Ok(())
        } else {
            Err(GraphError::ParseError(format!(
                "Expected {:?}, got {:?}",
                expected,
                self.current_token()
            )))
        }
    }
}

pub fn tokenize_and_parse(query: &str) -> Result<CypherQuery, GraphError> {
    let mut lexer = CypherLexer::new(query);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == CypherToken::Eof {
            break;
        }
        tokens.push(token);
    }

    let mut parser = CypherParser::new(tokens);
    parser.parse_query()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_match() {
        let tokens = vec![
            CypherToken::Match,
            CypherToken::LParen,
            CypherToken::Identifier("n".to_string()),
            CypherToken::RParen,
            CypherToken::Return,
            CypherToken::Identifier("n".to_string()),
        ];
        let mut parser = CypherParser::new(tokens);
        let query = parser.parse_query().unwrap();

        assert!(matches!(query.pattern, CypherPattern::Node(_)));
        assert_eq!(query.return_items.len(), 1);
        assert_eq!(query.return_items[0].variable, "n");
    }

    #[test]
    fn test_parse_match_with_label() {
        let tokens = vec![
            CypherToken::Match,
            CypherToken::LParen,
            CypherToken::Identifier("n".to_string()),
            CypherToken::Colon,
            CypherToken::Identifier("User".to_string()),
            CypherToken::RParen,
            CypherToken::Return,
            CypherToken::Identifier("n".to_string()),
        ];
        let mut parser = CypherParser::new(tokens);
        let query = parser.parse_query().unwrap();

        match &query.pattern {
            CypherPattern::Node(node) => {
                assert_eq!(node.label.as_deref(), Some("User"));
            }
            _ => panic!("Expected Node pattern"),
        }
    }

    #[test]
    fn test_parse_match_with_relationship() {
        let tokens = vec![
            CypherToken::Match,
            CypherToken::LParen,
            CypherToken::Identifier("n".to_string()),
            CypherToken::RParen,
            CypherToken::Dash,
            CypherToken::LBracket,
            CypherToken::Colon,
            CypherToken::Identifier("KNOWS".to_string()),
            CypherToken::RBracket,
            CypherToken::Arrow,
            CypherToken::LParen,
            CypherToken::Identifier("m".to_string()),
            CypherToken::RParen,
            CypherToken::Return,
            CypherToken::Identifier("n".to_string()),
            CypherToken::Comma,
            CypherToken::Identifier("m".to_string()),
        ];
        let mut parser = CypherParser::new(tokens);
        let query = parser.parse_query().unwrap();

        match &query.pattern {
            CypherPattern::Relationship {
                from,
                to,
                rel_label,
                ..
            } => {
                assert_eq!(from.variable.as_deref(), Some("n"));
                assert_eq!(to.variable.as_deref(), Some("m"));
                assert_eq!(rel_label.as_deref(), Some("KNOWS"));
            }
            _ => panic!("Expected Relationship pattern"),
        }
    }

    #[test]
    fn test_tokenize_and_parse() {
        let result = tokenize_and_parse("MATCH (n) RETURN n").unwrap();
        assert!(matches!(result.pattern, CypherPattern::Node(_)));
        assert_eq!(result.return_items.len(), 1);
    }
}
