pub mod error;
pub mod gateway;
pub mod schema;
pub mod stats;

pub use error::AgentSqlError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agentsql_error_display() {
        let err = AgentSqlError::ParseError("test error".to_string());
        assert_eq!(format!("{}", err), "Parse error: test error");
    }

    #[test]
    fn test_schema_service_new() {
        let schema = schema::SchemaService::new();
        let tables = schema.list_tables();
        assert!(!tables.is_empty());
    }

    #[test]
    fn test_stats_service_new() {
        let stats = stats::StatsService::new();
        let json = stats.get_stats();
        assert!(json.get("tables").is_some());
    }
}
