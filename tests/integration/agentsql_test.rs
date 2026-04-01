// AgentSQL Extension Integration Tests
//
// Tests for AgentSQL features: Schema, NL2SQL, Memory, Policy, Explain, Optimizer

use sqlrustgo_agentsql::{
    column_masking::{ColumnMasker, MaskingType},
    explain::{ExplainFormat, ExplainOptions, ExplainService},
    memory::{
        LoadMemoryRequest, MemoryService, MemoryType, SaveMemoryRequest, SearchMemoryRequest,
    },
    nl2sql::Nl2SqlService,
    optimizer::{OptimizerService, Priority},
    policy_engine::{
        ConditionOperator, PolicyCheckRequest, PolicyCondition, PolicyEffect, PolicyEngine,
    },
    schema::SchemaService,
    stats::StatsService,
};

#[test]
fn test_schema_service_introspection() {
    let schema = SchemaService::new();

    let tables = schema.list_tables();
    assert!(!tables.is_empty());
    assert!(tables.contains(&"users".to_string()));
    assert!(tables.contains(&"products".to_string()));
    assert!(tables.contains(&"orders".to_string()));
}

#[test]
fn test_schema_service_views() {
    let schema = SchemaService::new();
    let views = schema.list_views();
    assert!(!views.is_empty());
    assert!(views.contains(&"user_orders".to_string()));
}

#[test]
fn test_schema_get_table_schema() {
    let schema = SchemaService::new();
    let table = schema.get_table_schema("users");
    assert!(table.is_some());
}

#[test]
fn test_stats_service_basic() {
    let stats = StatsService::new();
    let json = stats.get_stats();
    assert!(json.get("tables").is_some());
    assert!(json.get("query_statistics").is_some());
}

#[test]
fn test_nl2sql_simple_select() {
    let schema = std::sync::Arc::new(SchemaService::new());
    let nl2sql = Nl2SqlService::new(schema);

    let result = nl2sql.natural_language_to_sql("show all users");
    assert!(result.sql.contains("SELECT"));
    assert!(result.sql.contains("FROM users"));
    assert!(result.confidence > 0.5);
}

#[test]
fn test_nl2sql_with_limit() {
    let schema = std::sync::Arc::new(SchemaService::new());
    let nl2sql = Nl2SqlService::new(schema);

    let result = nl2sql.natural_language_to_sql("show top 10 users");
    assert!(result.sql.contains("LIMIT 10"));
}

#[test]
fn test_memory_save_and_load() {
    let service = MemoryService::new();
    let mut service = service;

    let save_response = service.save_memory(SaveMemoryRequest {
        content: "Test memory content".to_string(),
        memory_type: Some(MemoryType::Conversation),
        tags: Some(vec!["test".to_string()]),
        agent_id: Some("agent1".to_string()),
        session_id: None,
        importance: Some(0.8),
        metadata: None,
    });

    assert!(save_response.success);

    let load_response = service.load_memory(LoadMemoryRequest {
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
    let mut service = MemoryService::new();

    service.save_memory(SaveMemoryRequest {
        content: "How to query users table".to_string(),
        memory_type: Some(MemoryType::Query),
        tags: None,
        agent_id: Some("agent1".to_string()),
        session_id: None,
        importance: None,
        metadata: None,
    });

    let response = service.search_memory(SearchMemoryRequest {
        query: "query users".to_string(),
        agent_id: Some("agent1".to_string()),
        memory_type: None,
        limit: Some(10),
    });

    assert_eq!(response.total, 1);
}

#[test]
fn test_policy_engine_basic_allow() {
    let engine = PolicyEngine::new();
    let request = PolicyCheckRequest {
        user_id: "user1".to_string(),
        resource: "table:users".to_string(),
        action: "SELECT".to_string(),
        context: None,
    };
    let response = engine.check(&request);
    assert!(response.allowed);
}

#[test]
fn test_policy_engine_basic_deny() {
    let engine = PolicyEngine::new();
    let request = PolicyCheckRequest {
        user_id: "user1".to_string(),
        resource: "table:audit_log".to_string(),
        action: "DELETE".to_string(),
        context: None,
    };
    let response = engine.check(&request);
    assert!(!response.allowed);
}

#[test]
fn test_column_masker_partial() {
    let masker = ColumnMasker::new();
    let value = serde_json::json!("test@example.com");
    let masked = masker.mask_value("email", &value);
    assert!(masked.as_str().unwrap().contains('*'));
}

#[test]
fn test_explain_service_select() {
    let service = ExplainService::new();
    let result = service.explain("SELECT * FROM users WHERE id = 1");
    assert_eq!(result.plan.node_type, "Select");
    assert!(result.estimated_cost > 0.0);
}

#[test]
fn test_optimizer_service() {
    let optimizer = OptimizerService::new();
    let result = optimizer.optimize("SELECT * FROM users");
    assert!(result.suggestions.iter().any(|s| s.id == "add_limit"));
}
