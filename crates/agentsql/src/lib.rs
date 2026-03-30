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
        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"products".to_string()));
        assert!(tables.contains(&"orders".to_string()));
    }

    #[test]
    fn test_schema_service_views() {
        let schema = schema::SchemaService::new();
        let views = schema.list_views();
        assert!(!views.is_empty());
        assert!(views.contains(&"user_orders".to_string()));
    }

    #[test]
    fn test_schema_service_get_table() {
        let schema = schema::SchemaService::new();
        let table = schema.get_table_schema("users");
        assert!(table.is_some());
        let table_json = table.unwrap();
        assert_eq!(table_json["name"], "users");
    }

    #[test]
    fn test_stats_service_new() {
        let stats = stats::StatsService::new();
        let json = stats.get_stats();
        assert!(json.get("tables").is_some());
        assert!(json.get("query_statistics").is_some());
    }

    #[test]
    fn test_stats_service_query_stats() {
        let stats = stats::StatsService::new();
        let query_stats = stats.get_query_stats();
        assert!(query_stats.get("total_queries").is_some());
        assert!(query_stats.get("select_queries").is_some());
    }
}
