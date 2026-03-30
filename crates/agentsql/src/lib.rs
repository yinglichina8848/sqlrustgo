pub mod error;
pub mod gateway;
pub mod memory;
pub mod nl2sql;
pub mod schema;
pub mod stats;

pub use error::AgentSqlError;
pub use memory::{MemoryEntry, MemoryService, MemoryStats, MemoryType};
pub use nl2sql::{Nl2SqlService, NlQueryRequest, NlQueryResponse, SqlExplanation};

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

    #[test]
    fn test_nl2sql_simple_select() {
        let schema = std::sync::Arc::new(schema::SchemaService::new());
        let nl2sql = nl2sql::Nl2SqlService::new(schema);
        let result = nl2sql.natural_language_to_sql("show all users");
        
        assert!(result.sql.contains("SELECT"));
        assert!(result.sql.contains("FROM users"));
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_nl2sql_with_limit() {
        let schema = std::sync::Arc::new(schema::SchemaService::new());
        let nl2sql = nl2sql::Nl2SqlService::new(schema);
        let result = nl2sql.natural_language_to_sql("show top 10 users");
        
        assert!(result.sql.contains("LIMIT 10"));
    }

    #[test]
    fn test_nl2sql_with_where() {
        let schema = std::sync::Arc::new(schema::SchemaService::new());
        let nl2sql = nl2sql::Nl2SqlService::new(schema);
        let result = nl2sql.natural_language_to_sql("show users where active");
        
        assert!(result.sql.contains("WHERE"));
        assert!(!result.where_conditions.is_empty());
    }

    #[test]
    fn test_memory_save_and_load() {
        let service = memory::MemoryService::new();
        let mut service = service;
        
        let save_response = service.save_memory(memory::SaveMemoryRequest {
            content: "Test memory content".to_string(),
            memory_type: Some(memory::MemoryType::Conversation),
            tags: Some(vec!["test".to_string()]),
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: Some(0.8),
            metadata: None,
        });
        
        assert!(save_response.success);
        
        let load_response = service.load_memory(memory::LoadMemoryRequest {
            agent_id: Some("agent1".to_string()),
            session_id: None,
            memory_type: None,
            tags: None,
            limit: None,
            since: None,
        });
        
        assert_eq!(load_response.total, 1);
        assert_eq!(load_response.memories[0].content, "Test memory content");
    }

    #[test]
    fn test_memory_search() {
        let mut service = memory::MemoryService::new();
        
        service.save_memory(memory::SaveMemoryRequest {
            content: "How to query users table".to_string(),
            memory_type: Some(memory::MemoryType::Query),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: None,
            metadata: None,
        });

        let response = service.search_memory(memory::SearchMemoryRequest {
            query: "query users".to_string(),
            agent_id: Some("agent1".to_string()),
            memory_type: None,
            limit: Some(10),
        });

        assert_eq!(response.total, 1);
        assert!(response.results[0].content.contains("query"));
    }

    #[test]
    fn test_memory_stats() {
        let mut service = memory::MemoryService::new();
        
        service.save_memory(memory::SaveMemoryRequest {
            content: "Memory 1".to_string(),
            memory_type: Some(memory::MemoryType::Conversation),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: None,
            metadata: None,
        });

        let stats = service.get_stats();
        assert_eq!(stats.total_memories, 1);
        assert_eq!(stats.unique_agents, 1);
    }
}
