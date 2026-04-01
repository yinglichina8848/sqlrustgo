use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentSqlError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Plan error: {0}")]
    PlanError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<String> for AgentSqlError {
    fn from(s: String) -> Self {
        AgentSqlError::InternalError(s)
    }
}
