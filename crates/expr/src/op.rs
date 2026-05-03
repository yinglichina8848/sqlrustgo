//! Binary and unary operators for expressions

use serde::{Deserialize, Serialize};

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    Plus,
    Minus,
    Multiply,
    Divide,
    Like,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Eq => write!(f, "="),
            BinaryOp::NotEq => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::LtEq => write!(f, "<="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::GtEq => write!(f, ">="),
            BinaryOp::And => write!(f, "AND"),
            BinaryOp::Or => write!(f, "OR"),
            BinaryOp::Plus => write!(f, "+"),
            BinaryOp::Minus => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Like => write!(f, "LIKE"),
        }
    }
}
