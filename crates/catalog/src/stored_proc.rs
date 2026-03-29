//! Stored Procedure definitions for the catalog

use serde::{Deserialize, Serialize};

/// Stored procedure definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredProcedure {
    pub name: String,
    pub params: Vec<StoredProcParam>,
    pub body: Vec<StoredProcStatement>,
}

/// Stored procedure parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredProcParam {
    pub name: String,
    pub mode: ParamMode,
    pub data_type: String,
}

/// Parameter mode for stored procedure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParamMode {
    In,
    Out,
    InOut,
}

/// Stored procedure statement types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StoredProcStatement {
    /// Raw SQL statement
    RawSql(String),
    /// SELECT ... INTO var1, var2 FROM ...
    SelectInto {
        columns: Vec<String>,
        into_vars: Vec<String>,
        table: String,
        where_clause: Option<String>,
    },
    /// SET variable = value
    Set { variable: String, value: String },
}

impl StoredProcedure {
    /// Create a new stored procedure
    pub fn new(name: String, params: Vec<StoredProcParam>, body: Vec<StoredProcStatement>) -> Self {
        Self { name, params, body }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_procedure_creation() {
        let proc = StoredProcedure::new(
            "test_proc".to_string(),
            vec![StoredProcParam {
                name: "param1".to_string(),
                mode: ParamMode::In,
                data_type: "INTEGER".to_string(),
            }],
            vec![StoredProcStatement::RawSql("SELECT 1".to_string())],
        );

        assert_eq!(proc.name, "test_proc");
        assert_eq!(proc.params.len(), 1);
        assert_eq!(proc.body.len(), 1);
    }

    #[test]
    fn test_param_mode() {
        assert!(matches!(ParamMode::In, ParamMode::In));
        assert!(matches!(ParamMode::Out, ParamMode::Out));
        assert!(matches!(ParamMode::InOut, ParamMode::InOut));
    }
}
