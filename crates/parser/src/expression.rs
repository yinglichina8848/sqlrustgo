use crate::token::Token;

/// ORDER BY single item
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItem {
    pub expr: Expression,
    pub asc: bool,         // true = ASC, false = DESC
    pub nulls_first: bool, // true = NULLS FIRST, false = NULLS LAST
}

/// Window frame info parsed from SQL
#[derive(Debug, Clone, PartialEq)]
pub struct WindowFrameInfo {
    pub mode: String, // ROWS, RANGE, or GROUPS
    pub start: FrameBoundInfo,
    pub end: FrameBoundInfo,
    pub exclude: Option<String>, // NO OTHERS, CURRENT ROW, GROUP, or TIES
}

/// Frame bound for window frame
#[derive(Debug, Clone, PartialEq)]
pub enum FrameBoundInfo {
    UnboundedPreceding,
    Preceding(i64),
    CurrentRow,
    Following(i64),
    UnboundedFollowing,
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(String),
    Identifier(String),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    Wildcard,
    /// Function call expression (for HAVING clause aggregates like COUNT(*), SUM(col))
    FunctionCall(String, Vec<Expression>),
    /// Subquery expression: (SELECT ...)
    Subquery(Box<crate::Statement>),
    /// Qualified column: table.column
    QualifiedColumn(String, String),
    /// Window function expression: ROW_NUMBER() OVER (PARTITION BY ... ORDER BY ...)
    WindowFunction {
        func: String,          // Function name: ROW_NUMBER, RANK, LEAD, etc.
        args: Vec<Expression>, // Arguments for LEAD/LAG/NTH_VALUE
        partition_by: Vec<Expression>,
        order_by: Vec<OrderByItem>,
        frame: Option<WindowFrameInfo>,
    },
    /// Parameter placeholder for prepared statements (?)
    Placeholder,
    /// BETWEEN expression: expr BETWEEN low AND high
    Between {
        expr: Box<Expression>,
        low: Box<Expression>,
        high: Box<Expression>,
    },
    /// IN value list expression: expr IN (value1, value2, ...)
    InList {
        expr: Box<Expression>,
        values: Vec<Expression>,
    },
    /// CASE WHEN expression: CASE WHEN cond THEN val ELSE default END
    CaseWhen {
        conditions: Vec<(Expression, Expression)>,
        else_result: Option<Box<Expression>>,
    },
    /// EXTRACT expression: EXTRACT(field FROM date)
    Extract {
        field: String,
        expr: Box<Expression>,
    },
    /// SUBSTRING expression: SUBSTRING(expr FROM start FOR len)
    Substring {
        expr: Box<Expression>,
        start: Box<Expression>,
        len: Option<Box<Expression>>,
    },
}

/// Expression parser
#[derive(Debug, Clone)]
pub struct ExpressionParser {
    tokens: Vec<Token>,
    position: usize,
}

impl ExpressionParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn next(&mut self) {
        self.position += 1;
    }

    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and_expression()?;

        while let Some(Token::Or) = self.current() {
            self.next();
            let right = self.parse_and_expression()?;
            left = Expression::BinaryOp(Box::new(left), "OR".to_string(), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison_expression()?;

        while let Some(Token::And) = self.current() {
            self.next();
            let right = self.parse_comparison_expression()?;
            left = Expression::BinaryOp(Box::new(left), "AND".to_string(), Box::new(right));
        }

        Ok(left)
    }

    fn parse_comparison_expression(&mut self) -> Result<Expression, String> {
        let left = self.parse_arithmetic_expression()?;

        // Check for comparison operator
        let op = match self.current() {
            Some(Token::Equal) => "=",
            Some(Token::NotEqual) => "!=",
            Some(Token::Greater) => ">",
            Some(Token::Less) => "<",
            Some(Token::GreaterEqual) => ">=",
            Some(Token::LessEqual) => "<=",
            _ => {
                // Check for BETWEEN
                if matches!(self.current(), Some(Token::Between)) {
                    self.next();
                    let low = self.parse_arithmetic_expression()?;
                    self.expect(Token::And)?;
                    let high = self.parse_arithmetic_expression()?;
                    return Ok(Expression::Between {
                        expr: Box::new(left),
                        low: Box::new(low),
                        high: Box::new(high),
                    });
                }
                // Check for IN
                if matches!(self.current(), Some(Token::In)) {
                    self.next();
                    self.expect(Token::LParen)?;
                    let mut values = Vec::new();
                    loop {
                        values.push(self.parse_arithmetic_expression()?);
                        match self.current() {
                            Some(Token::Comma) => {
                                self.next();
                            }
                            Some(Token::RParen) => {
                                self.next();
                                break;
                            }
                            _ => {
                                return Err("Expected ',' or ')' after value in IN list".to_string())
                            }
                        }
                    }
                    return Ok(Expression::InList {
                        expr: Box::new(left),
                        values,
                    });
                }
                // Check for LIKE
                if matches!(self.current(), Some(Token::Like)) {
                    self.next();
                    let pattern = self.parse_arithmetic_expression()?;
                    return Ok(Expression::BinaryOp(
                        Box::new(left),
                        "LIKE".to_string(),
                        Box::new(pattern),
                    ));
                }
                return Ok(left);
            }
        };

        self.next(); // consume operator
        let right = self.parse_arithmetic_expression()?;

        Ok(Expression::BinaryOp(
            Box::new(left),
            op.to_string(),
            Box::new(right),
        ))
    }

    fn parse_arithmetic_expression(&mut self) -> Result<Expression, String> {
        let left = self.parse_primary_expression()?;

        // Check for arithmetic operator
        let op = match self.current() {
            Some(Token::Plus) => "+",
            Some(Token::Minus) => "-",
            Some(Token::Star) => "*",
            Some(Token::Slash) => "/",
            _ => return Ok(left),
        };

        self.next(); // consume operator
        let right = self.parse_arithmetic_expression()?;

        Ok(Expression::BinaryOp(
            Box::new(left),
            op.to_string(),
            Box::new(right),
        ))
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        let token = self.current().cloned();

        match token {
            Some(Token::Identifier(name)) => {
                if name.to_uppercase() == "NULL" {
                    return Ok(Expression::Literal("NULL".to_string()));
                }
                self.next();

                // Check for qualified column name: table.column
                if matches!(self.current(), Some(Token::Dot)) {
                    self.next();
                    match self.current() {
                        Some(Token::Identifier(col_name)) => {
                            let expr = Expression::QualifiedColumn(name.clone(), col_name.clone());
                            self.next();
                            return Ok(expr);
                        }
                        _ => return Err("Expected column name after dot".to_string()),
                    }
                }

                Ok(Expression::Identifier(name.clone()))
            }
            Some(Token::NumberLiteral(n)) => {
                let expr = Expression::Literal(n.clone());
                self.next();
                Ok(expr)
            }
            Some(Token::StringLiteral(s)) => {
                let expr = Expression::Literal(s.to_string());
                self.next();
                Ok(expr)
            }
            Some(Token::DateLiteral(s)) => {
                let expr = Expression::Literal(s.to_string());
                self.next();
                Ok(expr)
            }
            Some(Token::TimestampLiteral(s)) => {
                let expr = Expression::Literal(s.to_string());
                self.next();
                Ok(expr)
            }
            Some(Token::Minus) => {
                self.next();
                if let Some(Token::NumberLiteral(n)) = self.current() {
                    let expr = Expression::Literal(format!("-{}", n));
                    self.next();
                    Ok(expr)
                } else {
                    Err("Expected number after -".to_string())
                }
            }
            Some(Token::LParen) => {
                self.next();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::Count) | Some(Token::Sum) | Some(Token::Avg) | Some(Token::Min)
            | Some(Token::Max) => {
                let func_name = match self.current() {
                    Some(Token::Count) => "COUNT",
                    Some(Token::Sum) => "SUM",
                    Some(Token::Avg) => "AVG",
                    Some(Token::Min) => "MIN",
                    Some(Token::Max) => "MAX",
                    _ => return Err("Unknown aggregate function".to_string()),
                };
                self.next();
                self.expect(Token::LParen)?;

                let mut args = Vec::new();
                match self.current() {
                    Some(Token::Star) => {
                        args.push(Expression::Wildcard);
                        self.next();
                    }
                    Some(Token::Identifier(_))
                    | Some(Token::Minus)
                    | Some(Token::LParen)
                    | Some(Token::Case) => {
                        let expr = self.parse_expression()?;
                        args.push(expr);
                    }
                    _ => {
                        return Err(
                            "Expected *, column name, or expression in aggregate".to_string()
                        )
                    }
                }

                self.expect(Token::RParen)?;
                Ok(Expression::FunctionCall(func_name.to_string(), args))
            }
            Some(Token::QuestionMark) => {
                self.next();
                Ok(Expression::Placeholder)
            }
            _ => Err("Expected expression".to_string()),
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), String> {
        match self.current() {
            Some(t) if t == &token => {
                self.next();
                Ok(())
            }
            _ => Err(format!("Expected {:?}", token)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Lexer;

    #[test]
    fn test_parse_simple_expression() {
        let tokens = Lexer::new("1 + 2").tokenize();
        let mut parser = ExpressionParser::new(tokens);
        let result = parser.parse_expression();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_identifier() {
        let tokens = Lexer::new("name").tokenize();
        let mut parser = ExpressionParser::new(tokens);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        match result.unwrap() {
            Expression::Identifier(s) => assert_eq!(s, "name"),
            _ => panic!("Expected Identifier"),
        }
    }

    #[test]
    fn test_parse_binary_op() {
        let tokens = Lexer::new("a + b").tokenize();
        let mut parser = ExpressionParser::new(tokens);
        let result = parser.parse_expression();
        assert!(result.is_ok());
    }
}
