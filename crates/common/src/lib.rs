// SQLRustGo Common Module - Common types and errors

#[derive(Debug)]
pub struct SqlError {
    message: String,
}

impl std::fmt::Display for SqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SqlError: {}", self.message)
    }
}

impl std::error::Error for SqlError {}

impl SqlError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}

pub type SqlResult<T> = Result<T, SqlError>;
