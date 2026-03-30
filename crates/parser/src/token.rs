//! Token definitions for SQL lexer
//! SQL token types and token struct

use std::fmt;

/// Token type enumeration representing all SQL lexical elements
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Add,
    Modify,
    Column,
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
    To,
    Alter,
    Unique,
    Index,
    On,
    As,
    Primary,
    Key,
    Duplicate,
    Replace,
    Ignore,
    Begin,
    Commit,
    Rollback,
    Savepoint,
    Release,
    Grant,
    Revoke,
    Analyze,
    Explain,
    Union,
    Intersect,
    Except,
    Trigger,
    Do,
    Before,
    After,
    Each,
    Row,
    New,
    Old,
    Referencing,
    Procedure,
    Call,
    Return,
    If,
    Then,
    Else,
    Elsif,
    EndIf,
    While,
    Loop,
    EndLoop,
    Leave,
    Iterate,
    Signal,
    Declare,
    Delimiter,
    View,
    All,
    Limit,
    Offset,

    // SHOW keywords (运维监控)
    Show,
    Status,
    Processlist,

    // KILL command keywords
    Kill,
    Connection,
    Query,

    // COPY keywords
    Copy,
    Format,
    Parquet,

    // Prepared Statements
    Prepare,
    Execute,
    Deallocate,
    Using,

    // Group By / Order By keywords
    Group,
    By,
    Having,
    Order,
    Asc,
    Desc,
    Nulls,
    First,
    Last,

    // Window Functions
    RowNumber,
    Rank,
    DenseRank,
    Lead,
    Lag,
    FirstValue,
    LastValue,
    NthValue,
    Over,
    Partition,
    Within,
    Rows,
    Range,
    Groups,
    Unbounded,
    Preceding,
    Following,
    Exclude,
    Current,
    Ties,
    NoOthers,
    Between,

    // CTE (Common Table Expression) - SQL-99
    With,
    Recursive,

    // MERGE and TRUNCATE - SQL-2003
    Merge,
    Truncate,
    When,
    Matched,

    // Aggregate Functions
    Length,
    Upper,
    Lower,
    Substr,
    Substring,
    Trim,
    Now,
    Curdate,
    Curtime,
    DateAdd,
    DateFormat,
    Count,
    Sum,
    Avg,
    Min,
    Max,

    // Data Types
    Integer,
    Text,
    Float,
    For,
    Decimal,
    Boolean,
    Blob,
    Json,
    Date,
    Timestamp,

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
    QuestionMark,

    // Column constraints
    AutoIncrement,
    References,

    // Wildcard
    Star, // * for SELECT *

    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    BooleanLiteral(bool),
    DateLiteral(String),
    TimestampLiteral(String),

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
            Token::DateLiteral(s) => write!(f, "DATE '{}'", s),
            Token::TimestampLiteral(s) => write!(f, "TIMESTAMP '{}'", s),
            // Keywords and data types - uppercase
            Token::Add => write!(f, "ADD"),
            Token::Modify => write!(f, "MODIFY"),
            Token::Column => write!(f, "COLUMN"),
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
            Token::To => write!(f, "TO"),
            Token::Unique => write!(f, "UNIQUE"),
            Token::Index => write!(f, "INDEX"),
            Token::On => write!(f, "ON"),
            Token::As => write!(f, "AS"),
            Token::Primary => write!(f, "PRIMARY"),
            Token::Key => write!(f, "KEY"),
            Token::Duplicate => write!(f, "DUPLICATE"),
            Token::Replace => write!(f, "REPLACE"),
            Token::Ignore => write!(f, "IGNORE"),
            Token::Begin => write!(f, "BEGIN"),
            Token::Commit => write!(f, "COMMIT"),
            Token::Rollback => write!(f, "ROLLBACK"),
            Token::Savepoint => write!(f, "SAVEPOINT"),
            Token::Release => write!(f, "RELEASE"),
            Token::Grant => write!(f, "GRANT"),
            Token::Revoke => write!(f, "REVOKE"),
            Token::Analyze => write!(f, "ANALYZE"),
            Token::Explain => write!(f, "EXPLAIN"),
            Token::Union => write!(f, "UNION"),
            Token::Intersect => write!(f, "INTERSECT"),
            Token::Except => write!(f, "EXCEPT"),
            Token::Trigger => write!(f, "TRIGGER"),
            Token::Do => write!(f, "DO"),
            Token::Before => write!(f, "BEFORE"),
            Token::After => write!(f, "AFTER"),

            Token::Each => write!(f, "EACH"),
            Token::Row => write!(f, "ROW"),
            Token::New => write!(f, "NEW"),
            Token::Old => write!(f, "OLD"),
            Token::Referencing => write!(f, "REFERENCING"),
            Token::Procedure => write!(f, "PROCEDURE"),
            Token::Call => write!(f, "CALL"),
            Token::Return => write!(f, "RETURN"),
            Token::If => write!(f, "IF"),
            Token::Then => write!(f, "THEN"),
            Token::Else => write!(f, "ELSE"),
            Token::Elsif => write!(f, "ELSIF"),
            Token::EndIf => write!(f, "END IF"),
            Token::While => write!(f, "WHILE"),
            Token::Loop => write!(f, "LOOP"),
            Token::EndLoop => write!(f, "END LOOP"),
            Token::Leave => write!(f, "LEAVE"),
            Token::Iterate => write!(f, "ITERATE"),
            Token::Signal => write!(f, "SIGNAL"),
            Token::Declare => write!(f, "DECLARE"),
            Token::Delimiter => write!(f, "DELIMITER"),
            Token::View => write!(f, "VIEW"),
            Token::All => write!(f, "ALL"),
            Token::Limit => write!(f, "LIMIT"),
            Token::Offset => write!(f, "OFFSET"),
            // Group By / Order By keywords
            Token::Group => write!(f, "GROUP"),
            Token::By => write!(f, "BY"),
            Token::Having => write!(f, "HAVING"),
            Token::Order => write!(f, "ORDER"),
            Token::Asc => write!(f, "ASC"),
            Token::Desc => write!(f, "DESC"),
            Token::Nulls => write!(f, "NULLS"),
            Token::First => write!(f, "FIRST"),
            Token::Last => write!(f, "LAST"),
            // Window Functions
            Token::RowNumber => write!(f, "ROW_NUMBER"),
            Token::Rank => write!(f, "RANK"),
            Token::DenseRank => write!(f, "DENSE_RANK"),
            Token::Lead => write!(f, "LEAD"),
            Token::Lag => write!(f, "LAG"),
            Token::FirstValue => write!(f, "FIRST_VALUE"),
            Token::LastValue => write!(f, "LAST_VALUE"),
            Token::NthValue => write!(f, "NTH_VALUE"),
            Token::Over => write!(f, "OVER"),
            Token::Partition => write!(f, "PARTITION"),
            Token::Within => write!(f, "WITHIN"),
            Token::Rows => write!(f, "ROWS"),
            Token::Range => write!(f, "RANGE"),
            Token::Groups => write!(f, "GROUPS"),
            Token::Unbounded => write!(f, "UNBOUNDED"),
            Token::Preceding => write!(f, "PRECEDING"),
            Token::Following => write!(f, "FOLLOWING"),
            Token::Exclude => write!(f, "EXCLUDE"),
            Token::Current => write!(f, "CURRENT"),
            Token::Ties => write!(f, "TIES"),
            Token::NoOthers => write!(f, "NO OTHERS"),
            Token::Between => write!(f, "BETWEEN"),
            Token::With => write!(f, "WITH"),
            Token::Recursive => write!(f, "RECURSIVE"),
            Token::Merge => write!(f, "MERGE"),
            Token::Truncate => write!(f, "TRUNCATE"),
            Token::When => write!(f, "WHEN"),
            Token::Matched => write!(f, "MATCHED"),
            Token::Integer => write!(f, "INTEGER"),
            Token::Text => write!(f, "TEXT"),
            Token::Float => write!(f, "FLOAT"),
            Token::For => write!(f, "FOR"),
            Token::Decimal => write!(f, "DECIMAL"),
            Token::Boolean => write!(f, "BOOLEAN"),
            Token::Blob => write!(f, "BLOB"),
            Token::Json => write!(f, "JSON"),
            Token::Date => write!(f, "DATE"),
            Token::Timestamp => write!(f, "TIMESTAMP"),
            // Aggregate Functions
            Token::Length => write!(f, "LENGTH"),
            Token::Upper => write!(f, "UPPER"),
            Token::Lower => write!(f, "LOWER"),
            Token::Substr => write!(f, "SUBSTR"),
            Token::Substring => write!(f, "SUBSTRING"),
            Token::Trim => write!(f, "TRIM"),
            Token::Now => write!(f, "NOW"),
            Token::Curdate => write!(f, "CURDATE"),
            Token::Curtime => write!(f, "CURTIME"),
            Token::DateAdd => write!(f, "DATE_ADD"),
            Token::DateFormat => write!(f, "DATE_FORMAT"),
            Token::Count => write!(f, "COUNT"),
            Token::Sum => write!(f, "SUM"),
            Token::Avg => write!(f, "AVG"),
            Token::Min => write!(f, "MIN"),
            Token::Max => write!(f, "MAX"),
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
            // Column constraints
            Token::AutoIncrement => write!(f, "AUTO_INCREMENT"),
            Token::References => write!(f, "REFERENCES"),
            // Syntax
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::SingleQuote => write!(f, "'"),
            Token::QuestionMark => write!(f, "?"),
            Token::Star => write!(f, "*"),
            Token::Eof => write!(f, "EOF"),
            // SHOW keywords
            Token::Show => write!(f, "SHOW"),
            Token::Status => write!(f, "STATUS"),
            Token::Processlist => write!(f, "PROCESSLIST"),
            // KILL keywords
            Token::Kill => write!(f, "KILL"),
            Token::Connection => write!(f, "CONNECTION"),
            Token::Query => write!(f, "QUERY"),
            // COPY keywords
            Token::Copy => write!(f, "COPY"),
            Token::Format => write!(f, "FORMAT"),
            Token::Parquet => write!(f, "PARQUET"),
            // Prepared Statements
            Token::Prepare => write!(f, "PREPARE"),
            Token::Execute => write!(f, "EXECUTE"),
            Token::Deallocate => write!(f, "DEALLOCATE"),
            Token::Using => write!(f, "USING"),
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
            | "VIEW"
            | "EXPLAIN"
            | "ON"
            | "AS"
            | "PRIMARY"
            | "KEY"
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
            | "DATE"
            | "TIMESTAMP"
            | "TRUE"
            | "FALSE"
            | "AND"
            | "OR"
            | "NOT"
            | "OVER"
            | "PARTITION"
            | "ROW_NUMBER"
            | "RANK"
            | "DENSE_RANK"
            | "LEAD"
            | "LAG"
            | "FIRST_VALUE"
            | "LAST_VALUE"
            | "NTH_VALUE"
            | "UNBOUNDED"
            | "PRECEDING"
            | "FOLLOWING"
            | "EXCLUDE"
            | "CURRENT ROW"
            | "TIES"
            | "NO OTHERS"
            | "BETWEEN"
            | "PREPARE"
            | "EXECUTE"
            | "DEALLOCATE"
            | "USING"
            | "COPY"
            | "FORMAT"
            | "PARQUET"
            | "KILL"
            | "CONNECTION"
            | "QUERY"
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
        "TO" => Some(Token::To),
        "ON" => Some(Token::On),
        "PRIMARY" => Some(Token::Primary),
        "KEY" => Some(Token::Key),
        "DUPLICATE" => Some(Token::Duplicate),
        "REPLACE" => Some(Token::Replace),
        "IGNORE" => Some(Token::Ignore),
        "BEGIN" => Some(Token::Begin),
        "COMMIT" => Some(Token::Commit),
        "ROLLBACK" => Some(Token::Rollback),
        "SAVEPOINT" => Some(Token::Savepoint),
        "RELEASE" => Some(Token::Release),
        "GRANT" => Some(Token::Grant),
        "REVOKE" => Some(Token::Revoke),
        "ALL" => Some(Token::All),
        "INTEGER" => Some(Token::Integer),
        "TEXT" => Some(Token::Text),
        "FLOAT" => Some(Token::Float),
        "BOOLEAN" => Some(Token::Boolean),
        "BLOB" => Some(Token::Blob),
        "DATE" => Some(Token::Date),
        "TIMESTAMP" => Some(Token::Timestamp),
        "AUTO_INCREMENT" | "AUTOINCREMENT" => Some(Token::AutoIncrement),
        "REFERENCES" => Some(Token::References),
        "TRUE" => Some(Token::BooleanLiteral(true)),
        "FALSE" => Some(Token::BooleanLiteral(false)),
        "AND" => Some(Token::And),
        "OR" => Some(Token::Or),
        "NOT" => Some(Token::Not),
        "OVER" => Some(Token::Over),
        "PARTITION" => Some(Token::Partition),
        "ROW_NUMBER" => Some(Token::RowNumber),
        "RANK" => Some(Token::Rank),
        "DENSE_RANK" => Some(Token::DenseRank),
        "LEAD" => Some(Token::Lead),
        "LAG" => Some(Token::Lag),
        "FIRST_VALUE" => Some(Token::FirstValue),
        "LAST_VALUE" => Some(Token::LastValue),
        "NTH_VALUE" => Some(Token::NthValue),
        "UNBOUNDED" => Some(Token::Unbounded),
        "PRECEDING" => Some(Token::Preceding),
        "FOLLOWING" => Some(Token::Following),
        "EXCLUDE" => Some(Token::Exclude),
        "CURRENT ROW" => Some(Token::Current),
        "TIES" => Some(Token::Ties),
        "NO OTHERS" => Some(Token::NoOthers),
        "BETWEEN" => Some(Token::Between),
        "VIEW" => Some(Token::View),
        "AS" => Some(Token::As),
        "EXPLAIN" => Some(Token::Explain),
        "SHOW" => Some(Token::Show),
        "STATUS" => Some(Token::Status),
        "PROCESSLIST" => Some(Token::Processlist),
        "PREPARE" => Some(Token::Prepare),
        "EXECUTE" => Some(Token::Execute),
        "DEALLOCATE" => Some(Token::Deallocate),
        "USING" => Some(Token::Using),
        "COPY" => Some(Token::Copy),
        "FORMAT" => Some(Token::Format),
        "PARQUET" => Some(Token::Parquet),
        "KILL" => Some(Token::Kill),
        "CONNECTION" => Some(Token::Connection),
        "QUERY" => Some(Token::Query),
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
        assert_eq!(from_keyword("DATE"), Some(Token::Date));
        assert_eq!(from_keyword("TIMESTAMP"), Some(Token::Timestamp));

        // Boolean literals
        assert_eq!(from_keyword("FALSE"), Some(Token::BooleanLiteral(false)));

        // Logical operators
        assert_eq!(from_keyword("AND"), Some(Token::And));
        assert_eq!(from_keyword("OR"), Some(Token::Or));
        assert_eq!(from_keyword("NOT"), Some(Token::Not));

        // Case insensitive
        assert_eq!(from_keyword("Select"), Some(Token::Select));
        assert_eq!(from_keyword("FROM"), Some(Token::From));
        assert_eq!(from_keyword("date"), Some(Token::Date));
        assert_eq!(from_keyword("Timestamp"), Some(Token::Timestamp));

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
