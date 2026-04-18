//! Token definitions for SQL lexer
//! SQL token types and token struct

use std::fmt;

/// Token type enumeration representing all SQL lexical elements
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,
    Create,
    Table,
    Drop,
    Alter,
    Index,
    On,
    Primary,
    Key,
    Add,
    Column,
    Rename,
    To,
    Begin,
    Commit,
    Rollback,
    Grant,
    Revoke,
    Analyze,

    // Constraint keywords
    Foreign,
    References,
    Unique,
    Check,
    Constraint,
    Cascade,
    Restrict,
    No,
    Action,
    Default,

    // Subquery keywords
    Exists,
    In,
    All,
    Any,
    Some,

    // CTE keywords
    With,
    Recursive,
    As,

    // Aggregate function keywords
    Count,
    Sum,
    Avg,
    Min,
    Max,

    // JOIN keywords
    Join,
    Inner,
    Left,
    Right,
    Full,
    Cross,
    Outer,
    Natural,

    // Trigger keywords
    Trigger,
    Before,
    After,
    Instead,
    Of,
    New,
    Old,
    For,
    Each,
    Row,
    Statement,
    Referencing,
    End,

    // Other SQL keywords
    Group,
    By,
    Having,
    Order,
    Limit,
    Offset,
    Distinct,
    Union,
    Intersect,
    Except,
    AsOf,
    Window,
    Partition,
    Over,
    Between,
    Unbounded,
    Preceding,
    Following,
    Current,
    Rows,
    Grouping,
    Rollup,
    Cube,

    // Transaction keywords
    Transaction,
    Work,
    Savepoint,
    Release,
    Isolation,
    Level,
    Serializable,
    Repeatable,
    Read,
    Write,
    Only,

    // Data Types
    Integer,
    Text,
    Float,
    Boolean,
    Blob,
    Null,

    // Operators
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    And,
    Or,
    Not,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,

    // Syntax
    LParen,
    RParen,
    Comma,
    Dot,
    Semicolon,
    Colon,
    SingleQuote,

    // Wildcard
    Star, // * for SELECT *

    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    BooleanLiteral(bool),

    // Special
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "IDENTIFIER({})", s),
            Token::StringLiteral(s) => write!(f, "'{}'", s),
            Token::NumberLiteral(s) => write!(f, "{}", s),
            Token::BooleanLiteral(b) => write!(f, "{}", b),
            // Keywords and data types - uppercase
            Token::Select => write!(f, "SELECT"),
            Token::From => write!(f, "FROM"),
            Token::Where => write!(f, "WHERE"),
            Token::Insert => write!(f, "INSERT"),
            Token::Into => write!(f, "INTO"),
            Token::Values => write!(f, "VALUES"),
            Token::Update => write!(f, "UPDATE"),
            Token::Set => write!(f, "SET"),
            Token::Delete => write!(f, "DELETE"),
            Token::Create => write!(f, "CREATE"),
            Token::Table => write!(f, "TABLE"),
            Token::Drop => write!(f, "DROP"),
            Token::Alter => write!(f, "ALTER"),
            Token::Index => write!(f, "INDEX"),
            Token::On => write!(f, "ON"),
            Token::Primary => write!(f, "PRIMARY"),
            Token::Key => write!(f, "KEY"),
            Token::Add => write!(f, "ADD"),
            Token::Column => write!(f, "COLUMN"),
            Token::Rename => write!(f, "RENAME"),
            Token::To => write!(f, "TO"),
            Token::Begin => write!(f, "BEGIN"),
            Token::Commit => write!(f, "COMMIT"),
            Token::Rollback => write!(f, "ROLLBACK"),
            Token::Grant => write!(f, "GRANT"),
            Token::Revoke => write!(f, "REVOKE"),
            Token::Analyze => write!(f, "ANALYZE"),
            // Constraint keywords
            Token::Foreign => write!(f, "FOREIGN"),
            Token::References => write!(f, "REFERENCES"),
            Token::Unique => write!(f, "UNIQUE"),
            Token::Check => write!(f, "CHECK"),
            Token::Constraint => write!(f, "CONSTRAINT"),
            Token::Cascade => write!(f, "CASCADE"),
            Token::Restrict => write!(f, "RESTRICT"),
            Token::No => write!(f, "NO"),
            Token::Action => write!(f, "ACTION"),
            Token::Default => write!(f, "DEFAULT"),
            Token::Exists => write!(f, "EXISTS"),
            Token::In => write!(f, "IN"),
            Token::All => write!(f, "ALL"),
            Token::Any => write!(f, "ANY"),
            Token::Some => write!(f, "SOME"),
            Token::With => write!(f, "WITH"),
            Token::Recursive => write!(f, "RECURSIVE"),
            Token::As => write!(f, "AS"),
            Token::Count => write!(f, "COUNT"),
            Token::Sum => write!(f, "SUM"),
            Token::Avg => write!(f, "AVG"),
            Token::Min => write!(f, "MIN"),
            Token::Max => write!(f, "MAX"),
            Token::Group => write!(f, "GROUP"),
            Token::By => write!(f, "BY"),
            Token::Having => write!(f, "HAVING"),
            Token::Order => write!(f, "ORDER"),
            Token::Limit => write!(f, "LIMIT"),
            Token::Offset => write!(f, "OFFSET"),
            Token::Distinct => write!(f, "DISTINCT"),
            Token::Join => write!(f, "JOIN"),
            Token::Inner => write!(f, "INNER"),
            Token::Left => write!(f, "LEFT"),
            Token::Right => write!(f, "RIGHT"),
            Token::Full => write!(f, "FULL"),
            Token::Cross => write!(f, "CROSS"),
            Token::Outer => write!(f, "OUTER"),
            Token::Natural => write!(f, "NATURAL"),
            // Trigger tokens
            Token::Trigger => write!(f, "TRIGGER"),
            Token::Before => write!(f, "BEFORE"),
            Token::After => write!(f, "AFTER"),
            Token::Instead => write!(f, "INSTEAD"),
            Token::Of => write!(f, "OF"),
            Token::New => write!(f, "NEW"),
            Token::Old => write!(f, "OLD"),
            Token::For => write!(f, "FOR"),
            Token::Each => write!(f, "EACH"),
            // Token::Row is defined elsewhere
            Token::Statement => write!(f, "STATEMENT"),
            Token::Referencing => write!(f, "REFERENCING"),
            Token::End => write!(f, "END"),
            Token::Union => write!(f, "UNION"),
            Token::Intersect => write!(f, "INTERSECT"),
            Token::Except => write!(f, "EXCEPT"),
            Token::Transaction => write!(f, "TRANSACTION"),
            Token::Work => write!(f, "WORK"),
            Token::Savepoint => write!(f, "SAVEPOINT"),
            Token::Release => write!(f, "RELEASE"),
            Token::Isolation => write!(f, "ISOLATION"),
            Token::Level => write!(f, "LEVEL"),
            Token::Serializable => write!(f, "SERIALIZABLE"),
            Token::Repeatable => write!(f, "REPEATABLE"),
            Token::Read => write!(f, "READ"),
            Token::Write => write!(f, "WRITE"),
            Token::Only => write!(f, "ONLY"),
            Token::Unbounded => write!(f, "UNBOUNDED"),
            Token::Preceding => write!(f, "PRECEDING"),
            Token::Following => write!(f, "FOLLOWING"),
            Token::Current => write!(f, "CURRENT"),
            Token::Row => write!(f, "ROW"),
            Token::Rows => write!(f, "ROWS"),
            Token::Grouping => write!(f, "GROUPING"),
            Token::Rollup => write!(f, "ROLLUP"),
            Token::Cube => write!(f, "CUBE"),
            Token::AsOf => write!(f, "ASOF"),
            Token::Window => write!(f, "WINDOW"),
            Token::Partition => write!(f, "PARTITION"),
            Token::Over => write!(f, "OVER"),
            Token::Between => write!(f, "BETWEEN"),
            Token::Integer => write!(f, "INTEGER"),
            Token::Text => write!(f, "TEXT"),
            Token::Float => write!(f, "FLOAT"),
            Token::Boolean => write!(f, "BOOLEAN"),
            Token::Blob => write!(f, "BLOB"),
            Token::Null => write!(f, "NULL"),
            // Operators - uppercase
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "<>"),
            Token::Greater => write!(f, ">"),
            Token::Less => write!(f, "<"),
            Token::GreaterEqual => write!(f, ">="),
            Token::LessEqual => write!(f, "<="),
            Token::And => write!(f, "AND"),
            Token::Or => write!(f, "OR"),
            Token::Not => write!(f, "NOT"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Asterisk => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            // Syntax
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::SingleQuote => write!(f, "'"),
            Token::Star => write!(f, "*"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Check if a string is a SQL keyword
pub fn is_keyword(s: &str) -> bool {
    matches!(
        s.to_uppercase().as_str(),
        "SELECT"
            | "FROM"
            | "WHERE"
            | "INSERT"
            | "INTO"
            | "VALUES"
            | "UPDATE"
            | "SET"
            | "DELETE"
            | "CREATE"
            | "TABLE"
            | "DROP"
            | "ALTER"
            | "INDEX"
            | "ON"
            | "PRIMARY"
            | "KEY"
            | "ADD"
            | "COLUMN"
            | "RENAME"
            | "TO"
            | "BEGIN"
            | "COMMIT"
            | "ROLLBACK"
            | "GRANT"
            | "REVOKE"
            | "INTEGER"
            | "TEXT"
            | "FLOAT"
            | "BOOLEAN"
            | "BLOB"
            | "NULL"
            | "TRUE"
            | "FALSE"
            | "AND"
            | "OR"
            | "NOT"
            | "EXISTS"
            | "IN"
            | "ALL"
            | "ANY"
            | "SOME"
            | "WITH"
            | "RECURSIVE"
            | "AS"
    )
}

/// Convert a keyword string to its corresponding Token
pub fn from_keyword(s: &str) -> Option<Token> {
    match s.to_uppercase().as_str() {
        "SELECT" => Some(Token::Select),
        "FROM" => Some(Token::From),
        "WHERE" => Some(Token::Where),
        "INSERT" => Some(Token::Insert),
        "INTO" => Some(Token::Into),
        "VALUES" => Some(Token::Values),
        "UPDATE" => Some(Token::Update),
        "SET" => Some(Token::Set),
        "DELETE" => Some(Token::Delete),
        "CREATE" => Some(Token::Create),
        "TABLE" => Some(Token::Table),
        "DROP" => Some(Token::Drop),
        "ALTER" => Some(Token::Alter),
        "INDEX" => Some(Token::Index),
        "ON" => Some(Token::On),
        "PRIMARY" => Some(Token::Primary),
        "KEY" => Some(Token::Key),
        "ADD" => Some(Token::Add),
        "COLUMN" => Some(Token::Column),
        "RENAME" => Some(Token::Rename),
        "TO" => Some(Token::To),
        "BEGIN" => Some(Token::Begin),
        "COMMIT" => Some(Token::Commit),
        "ROLLBACK" => Some(Token::Rollback),
        "GRANT" => Some(Token::Grant),
        "REVOKE" => Some(Token::Revoke),
        "INTEGER" => Some(Token::Integer),
        "TEXT" => Some(Token::Text),
        "FLOAT" => Some(Token::Float),
        "BOOLEAN" => Some(Token::Boolean),
        "BLOB" => Some(Token::Blob),
        "NULL" => Some(Token::Null),
        "TRUE" => Some(Token::BooleanLiteral(true)),
        "FALSE" => Some(Token::BooleanLiteral(false)),
        "AND" => Some(Token::And),
        "OR" => Some(Token::Or),
        "NOT" => Some(Token::Not),
        "FOREIGN" => Some(Token::Foreign),
        "REFERENCES" => Some(Token::References),
        "UNIQUE" => Some(Token::Unique),
        "CHECK" => Some(Token::Check),
        "CONSTRAINT" => Some(Token::Constraint),
        "CASCADE" => Some(Token::Cascade),
        "RESTRICT" => Some(Token::Restrict),
        "NO" => Some(Token::No),
        "ACTION" => Some(Token::Action),
        "DEFAULT" => Some(Token::Default),
        "EXISTS" => Some(Token::Exists),
        "IN" => Some(Token::In),
        "ALL" => Some(Token::All),
        "ANY" => Some(Token::Any),
        "SOME" => Some(Token::Some),
        "WITH" => Some(Token::With),
        "RECURSIVE" => Some(Token::Recursive),
        "AS" => Some(Token::As),
        "COUNT" => Some(Token::Count),
        "SUM" => Some(Token::Sum),
        "AVG" => Some(Token::Avg),
        "MIN" => Some(Token::Min),
        "MAX" => Some(Token::Max),
        "GROUP" => Some(Token::Group),
        "BY" => Some(Token::By),
        "HAVING" => Some(Token::Having),
        "ORDER" => Some(Token::Order),
        "LIMIT" => Some(Token::Limit),
        "OFFSET" => Some(Token::Offset),
        "DISTINCT" => Some(Token::Distinct),
        "JOIN" => Some(Token::Join),
        "INNER" => Some(Token::Inner),
        "LEFT" => Some(Token::Left),
        "RIGHT" => Some(Token::Right),
        "FULL" => Some(Token::Full),
        "CROSS" => Some(Token::Cross),
        "OUTER" => Some(Token::Outer),
        "NATURAL" => Some(Token::Natural),
        // Trigger keywords
        "TRIGGER" => Some(Token::Trigger),
        "BEFORE" => Some(Token::Before),
        "AFTER" => Some(Token::After),
        "INSTEAD" => Some(Token::Instead),
        "OF" => Some(Token::Of),
        "NEW" => Some(Token::New),
        "OLD" => Some(Token::Old),
        "FOR" => Some(Token::For),
        "EACH" => Some(Token::Each),
        "ROW" => Some(Token::Row),
        "STATEMENT" => Some(Token::Statement),
        "REFERENCING" => Some(Token::Referencing),
        "UNION" => Some(Token::Union),
        "INTERSECT" => Some(Token::Intersect),
        "EXCEPT" => Some(Token::Except),
        "TRANSACTION" => Some(Token::Transaction),
        "WORK" => Some(Token::Work),
        "SAVEPOINT" => Some(Token::Savepoint),
        "RELEASE" => Some(Token::Release),
        "ISOLATION" => Some(Token::Isolation),
        "LEVEL" => Some(Token::Level),
        "SERIALIZABLE" => Some(Token::Serializable),
        "REPEATABLE" => Some(Token::Repeatable),
        "READ" => Some(Token::Read),
        "WRITE" => Some(Token::Write),
        "ONLY" => Some(Token::Only),
        "UNBOUNDED" => Some(Token::Unbounded),
        "PRECEDING" => Some(Token::Preceding),
        "FOLLOWING" => Some(Token::Following),
        "CURRENT" => Some(Token::Current),
        // ROW is defined above
        "ROWS" => Some(Token::Rows),
        "GROUPING" => Some(Token::Grouping),
        "ROLLUP" => Some(Token::Rollup),
        "CUBE" => Some(Token::Cube),
        "ASOF" => Some(Token::AsOf),
        "WINDOW" => Some(Token::Window),
        "PARTITION" => Some(Token::Partition),
        "OVER" => Some(Token::Over),
        "BETWEEN" => Some(Token::Between),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_display() {
        assert_eq!(Token::Select.to_string(), "SELECT");
        assert_eq!(Token::Integer.to_string(), "INTEGER");
        assert_eq!(
            Token::Identifier("users".to_string()).to_string(),
            "IDENTIFIER(users)"
        );
        assert_eq!(
            Token::StringLiteral("hello".to_string()).to_string(),
            "'hello'"
        );
        assert_eq!(Token::NumberLiteral("42".to_string()).to_string(), "42");
    }

    #[test]
    fn test_token_display_operators() {
        assert_eq!(Token::Equal.to_string(), "=");
        assert_eq!(Token::NotEqual.to_string(), "<>");
        assert_eq!(Token::Greater.to_string(), ">");
        assert_eq!(Token::Less.to_string(), "<");
        assert_eq!(Token::GreaterEqual.to_string(), ">=");
        assert_eq!(Token::LessEqual.to_string(), "<=");
        assert_eq!(Token::And.to_string(), "AND");
        assert_eq!(Token::Or.to_string(), "OR");
        assert_eq!(Token::Not.to_string(), "NOT");
        assert_eq!(Token::Plus.to_string(), "+");
        assert_eq!(Token::Minus.to_string(), "-");
        assert_eq!(Token::Asterisk.to_string(), "*");
        assert_eq!(Token::Slash.to_string(), "/");
        assert_eq!(Token::Percent.to_string(), "%");
    }

    #[test]
    fn test_token_display_syntax() {
        assert_eq!(Token::LParen.to_string(), "(");
        assert_eq!(Token::RParen.to_string(), ")");
        assert_eq!(Token::Comma.to_string(), ",");
        assert_eq!(Token::Dot.to_string(), ".");
        assert_eq!(Token::Semicolon.to_string(), ";");
        assert_eq!(Token::Colon.to_string(), ":");
        assert_eq!(Token::SingleQuote.to_string(), "'");
        assert_eq!(Token::Star.to_string(), "*");
        assert_eq!(Token::Eof.to_string(), "EOF");
    }

    #[test]
    fn test_token_display_keywords() {
        assert_eq!(Token::From.to_string(), "FROM");
        assert_eq!(Token::Where.to_string(), "WHERE");
        assert_eq!(Token::Insert.to_string(), "INSERT");
        assert_eq!(Token::Into.to_string(), "INTO");
        assert_eq!(Token::Values.to_string(), "VALUES");
        assert_eq!(Token::Update.to_string(), "UPDATE");
        assert_eq!(Token::Set.to_string(), "SET");
        assert_eq!(Token::Delete.to_string(), "DELETE");
        assert_eq!(Token::Create.to_string(), "CREATE");
        assert_eq!(Token::Table.to_string(), "TABLE");
        assert_eq!(Token::Drop.to_string(), "DROP");
        assert_eq!(Token::Alter.to_string(), "ALTER");
        assert_eq!(Token::Index.to_string(), "INDEX");
    }

    #[test]
    fn test_token_display_data_types() {
        assert_eq!(Token::Text.to_string(), "TEXT");
        assert_eq!(Token::Float.to_string(), "FLOAT");
        assert_eq!(Token::Boolean.to_string(), "BOOLEAN");
        assert_eq!(Token::Blob.to_string(), "BLOB");
        assert_eq!(Token::Null.to_string(), "NULL");
    }

    #[test]
    fn test_token_display_transaction() {
        assert_eq!(Token::Begin.to_string(), "BEGIN");
        assert_eq!(Token::Commit.to_string(), "COMMIT");
        assert_eq!(Token::Rollback.to_string(), "ROLLBACK");
        assert_eq!(Token::Grant.to_string(), "GRANT");
        assert_eq!(Token::Revoke.to_string(), "REVOKE");
    }

    #[test]
    fn test_token_display_other_keywords() {
        assert_eq!(Token::On.to_string(), "ON");
        assert_eq!(Token::Primary.to_string(), "PRIMARY");
        assert_eq!(Token::Key.to_string(), "KEY");
    }

    #[test]
    fn test_token_boolean_literal() {
        assert_eq!(Token::BooleanLiteral(true).to_string(), "true");
        assert_eq!(Token::BooleanLiteral(false).to_string(), "false");
    }

    #[test]
    fn test_token_clone() {
        let token = Token::Identifier("test".to_string());
        let cloned = token.clone();
        assert_eq!(token, cloned);
    }

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("SELECT"));
        assert!(is_keyword("select"));
        assert!(!is_keyword("users"));
        assert!(is_keyword("TRUE"));
        assert!(is_keyword("null"));
    }

    #[test]
    fn test_is_keyword_various() {
        // Data type keywords
        assert!(is_keyword("INTEGER"));
        assert!(is_keyword("TEXT"));
        assert!(is_keyword("FLOAT"));
        assert!(is_keyword("BOOLEAN"));
        assert!(is_keyword("BLOB"));

        // Boolean keywords
        assert!(is_keyword("TRUE"));
        assert!(is_keyword("FALSE"));

        // Logical operators
        assert!(is_keyword("AND"));
        assert!(is_keyword("OR"));
        assert!(is_keyword("NOT"));

        // Non-keyword
        assert!(!is_keyword("users"));
        assert!(!is_keyword("foo"));
        assert!(!is_keyword("bar123"));
    }

    #[test]
    fn test_token_from_keyword() {
        assert_eq!(from_keyword("SELECT"), Some(Token::Select));
        assert_eq!(from_keyword("INSERT"), Some(Token::Insert));
        assert_eq!(from_keyword("UNKNOWN"), None);
        assert_eq!(from_keyword("select"), Some(Token::Select));
        assert_eq!(from_keyword("TRUE"), Some(Token::BooleanLiteral(true)));
    }

    #[test]
    fn test_from_keyword_various() {
        // DML keywords
        assert_eq!(from_keyword("FROM"), Some(Token::From));
        assert_eq!(from_keyword("WHERE"), Some(Token::Where));
        assert_eq!(from_keyword("UPDATE"), Some(Token::Update));
        assert_eq!(from_keyword("SET"), Some(Token::Set));
        assert_eq!(from_keyword("DELETE"), Some(Token::Delete));

        // DDL keywords
        assert_eq!(from_keyword("CREATE"), Some(Token::Create));
        assert_eq!(from_keyword("TABLE"), Some(Token::Table));
        assert_eq!(from_keyword("DROP"), Some(Token::Drop));
        assert_eq!(from_keyword("ALTER"), Some(Token::Alter));
        assert_eq!(from_keyword("INDEX"), Some(Token::Index));

        // Transaction keywords
        assert_eq!(from_keyword("BEGIN"), Some(Token::Begin));
        assert_eq!(from_keyword("COMMIT"), Some(Token::Commit));
        assert_eq!(from_keyword("ROLLBACK"), Some(Token::Rollback));

        // Data types
        assert_eq!(from_keyword("INTEGER"), Some(Token::Integer));
        assert_eq!(from_keyword("TEXT"), Some(Token::Text));
        assert_eq!(from_keyword("FLOAT"), Some(Token::Float));
        assert_eq!(from_keyword("BOOLEAN"), Some(Token::Boolean));
        assert_eq!(from_keyword("BLOB"), Some(Token::Blob));
        assert_eq!(from_keyword("NULL"), Some(Token::Null));

        // Boolean literals
        assert_eq!(from_keyword("FALSE"), Some(Token::BooleanLiteral(false)));

        // Logical operators
        assert_eq!(from_keyword("AND"), Some(Token::And));
        assert_eq!(from_keyword("OR"), Some(Token::Or));
        assert_eq!(from_keyword("NOT"), Some(Token::Not));

        // Case insensitive
        assert_eq!(from_keyword("Select"), Some(Token::Select));
        assert_eq!(from_keyword("FROM"), Some(Token::From));

        // Non-keywords return None
        assert_eq!(from_keyword("users"), None);
        assert_eq!(from_keyword("foo"), None);
    }

    #[test]
    fn test_token_debug() {
        let token = Token::Identifier("test".to_string());
        let debug_str = format!("{:?}", token);
        assert!(debug_str.contains("Identifier"));
    }
}
