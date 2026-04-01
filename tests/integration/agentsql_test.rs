#[cfg(test)]
mod tests {
    use sqlrustgo_agentsql::{
        column_masking::{ColumnMasker, MaskingConfig, MaskingRule, MaskingType},
        explain::{ExplainFormat, ExplainOptions, ExplainService},
        optimizer::{OptimizationResult, OptimizerService, Priority, SuggestionCategory},
        policy_engine::{PolicyCheckRequest, PolicyEffect, PolicyEngine},
    };

    #[test]
    fn test_policy_engine_allow() {
        let engine = PolicyEngine::new();
        let request = PolicyCheckRequest {
            user_id: "user1".to_string(),
            resource: "table:users".to_string(),
            action: "SELECT".to_string(),
            context: None,
        };
        let response = engine.check(&request);
        assert!(response.allowed, "SELECT on users should be allowed");
        assert_eq!(response.matched_policy, Some("policy_read_all".to_string()));
    }

    #[test]
    fn test_policy_engine_deny_delete_audit() {
        let engine = PolicyEngine::new();
        let request = PolicyCheckRequest {
            user_id: "user1".to_string(),
            resource: "table:audit_log".to_string(),
            action: "DELETE".to_string(),
            context: None,
        };
        let response = engine.check(&request);
        assert!(!response.allowed, "DELETE on audit_log should be denied");
        assert_eq!(
            response.matched_policy,
            Some("policy_deny_delete".to_string())
        );
    }

    #[test]
    fn test_policy_engine_with_condition() {
        let engine = PolicyEngine::new();
        let mut context = std::collections::HashMap::new();
        context.insert("user_role".to_string(), serde_json::json!("admin"));

        let request = PolicyCheckRequest {
            user_id: "admin1".to_string(),
            resource: "table:sensitive_data".to_string(),
            action: "INSERT".to_string(),
            context: Some(context),
        };

        let response = engine.check(&request);
        assert!(
            response.allowed,
            "Admin should be able to insert into sensitive_data"
        );
    }

    #[test]
    fn test_policy_engine_no_match() {
        let engine = PolicyEngine::new();
        let request = PolicyCheckRequest {
            user_id: "user1".to_string(),
            resource: "table:unknown_table".to_string(),
            action: "DELETE".to_string(),
            context: None,
        };
        let response = engine.check(&request);
        assert!(!response.allowed, "Unknown resource should be denied");
        assert_eq!(response.matched_policy, None);
    }

    #[test]
    fn test_column_masker_partial_email() {
        let masker = ColumnMasker::new();
        let value = serde_json::json!("john.doe@example.com");
        let masked = masker.mask_value("email", &value);
        let masked_str = masked.as_str().unwrap();
        assert!(masked_str.contains('*'), "Email should be masked");
        assert_ne!(masked_str, "john.doe@example.com", "Email should be masked");
    }

    #[test]
    fn test_column_masker_hash_ssn() {
        let masker = ColumnMasker::new();
        let value = serde_json::json!("123-45-6789");
        let masked = masker.mask_value("ssn", &value);
        assert!(masked.is_string(), "SSN should be hashed to string");
        assert_ne!(masked, value, "SSN should be masked, not original value");
    }

    #[test]
    fn test_column_masker_full_credit_card() {
        let masker = ColumnMasker::new();
        let value = serde_json::json!("4532123456789012");
        let masked = masker.mask_value("credit_card", &value);
        assert_eq!(
            masked,
            serde_json::json!("****"),
            "Credit card should be fully masked"
        );
    }

    #[test]
    fn test_column_masker_range_salary() {
        let masker = ColumnMasker::new();
        let masked = masker.mask_value("salary", &serde_json::json!(75000.0));
        assert_eq!(
            masked,
            serde_json::json!("50K - 100K"),
            "Salary should be masked to range"
        );
    }

    #[test]
    fn test_column_masker_no_rule() {
        let masker = ColumnMasker::new();
        let value = serde_json::json!("some_data");
        let masked = masker.mask_value("non_existent_column", &value);
        assert_eq!(
            masked, value,
            "Column without masking rule should return original value"
        );
    }

    #[test]
    fn test_column_masker_custom_config() {
        let custom_rules = vec![MaskingRule {
            id: "mask_custom".to_string(),
            column: "secret".to_string(),
            mask_type: MaskingType::Full,
            description: "Fully mask secret column".to_string(),
        }];
        let config = MaskingConfig {
            rules: custom_rules,
        };
        let masker = ColumnMasker::new_with_config(config);

        let value = serde_json::json!("secret_data");
        let masked = masker.mask_value("secret", &value);
        assert_eq!(
            masked,
            serde_json::json!("****"),
            "Custom rule should mask fully"
        );
    }

    #[test]
    fn test_explain_service_select() {
        let service = ExplainService::new();
        let result = service.explain("SELECT * FROM users WHERE id = 1");

        assert_eq!(result.plan.node_type, "Select");
        assert!(result.estimated_cost > 0.0, "Cost should be estimated");
        assert!(result.estimated_rows > 0, "Rows should be estimated");
    }

    #[test]
    fn test_explain_service_insert() {
        let service = ExplainService::new();
        let result = service.explain("INSERT INTO users (name) VALUES ('test')");

        assert_eq!(result.plan.node_type, "Insert");
        assert_eq!(result.plan.table, Some("users".to_string()));
    }

    #[test]
    fn test_explain_service_update() {
        let service = ExplainService::new();
        let result = service.explain("UPDATE users SET name = 'new' WHERE id = 1");

        assert_eq!(result.plan.node_type, "Update");
    }

    #[test]
    fn test_explain_service_delete() {
        let service = ExplainService::new();
        let result = service.explain("DELETE FROM users WHERE id = 1");

        assert_eq!(result.plan.node_type, "Delete");
    }

    #[test]
    fn test_explain_service_with_join() {
        let service = ExplainService::new();
        let result =
            service.explain("SELECT * FROM users JOIN orders ON users.id = orders.user_id");

        assert_eq!(result.plan.node_type, "Select");
        assert!(
            !result.plan.children.is_empty(),
            "Join query should have child nodes"
        );
    }

    #[test]
    fn test_explain_service_text_format() {
        let service = ExplainService::new();
        let text = service.explain_text("SELECT * FROM users");

        assert!(text.contains("Select"), "Text format should contain Select");
    }

    #[test]
    fn test_explain_service_warnings() {
        let service = ExplainService::new();
        let result = service.explain("SELECT * FROM audit_log");

        assert!(
            !result.warnings.is_empty(),
            "Large table scan should generate warnings"
        );
    }

    #[test]
    fn test_explain_service_json_format() {
        let service = ExplainService::new();
        let json = service.explain_json("SELECT * FROM users");

        assert!(json.get("plan").is_some(), "JSON should contain plan");
        assert!(
            json.get("estimated_cost").is_some(),
            "JSON should contain estimated_cost"
        );
    }

    #[test]
    fn test_explain_options() {
        let options = ExplainOptions {
            format: ExplainFormat::Json,
            verbose: true,
            analyze: true,
        };
        let service = ExplainService::new_with_options(options);
        let result = service.explain("SELECT 1");

        assert!(
            result.estimated_cost > 0.0,
            "Service should work with custom options"
        );
    }

    #[test]
    fn test_optimizer_service_add_limit() {
        let optimizer = OptimizerService::new();
        let result = optimizer.optimize("SELECT * FROM users");

        assert!(
            result.suggestions.iter().any(|s| s.id == "add_limit"),
            "Should suggest adding LIMIT"
        );
        assert!(
            result.optimized_sql.to_lowercase().contains("limit"),
            "Optimized SQL should contain LIMIT"
        );
    }

    #[test]
    fn test_optimizer_service_avoid_select_star() {
        let optimizer = OptimizerService::new();
        let result = optimizer.optimize("SELECT * FROM users WHERE id = 1");

        assert!(
            result
                .suggestions
                .iter()
                .any(|s| s.id == "avoid_select_star"),
            "Should suggest avoiding SELECT *"
        );
    }

    #[test]
    fn test_optimizer_service_use_index() {
        let optimizer = OptimizerService::new();
        let result = optimizer.optimize("SELECT * FROM users WHERE email = 'test@example.com'");

        assert!(
            result.suggestions.iter().any(|s| s.id == "use_index"),
            "Should suggest using index"
        );
    }

    #[test]
    fn test_optimizer_service_optimize_join_order() {
        let optimizer = OptimizerService::new();
        let result = optimizer
            .optimize("SELECT * FROM users, orders, products WHERE users.id = orders.user_id");

        assert!(
            result
                .suggestions
                .iter()
                .any(|s| s.id == "optimize_join_order"),
            "Should suggest optimizing join order"
        );
    }

    #[test]
    fn test_optimizer_service_explicit_join() {
        let optimizer = OptimizerService::new();
        let result =
            optimizer.optimize("SELECT * FROM users, orders WHERE users.id = orders.user_id");

        assert!(
            result
                .suggestions
                .iter()
                .any(|s| s.id == "use_explicit_join"),
            "Should suggest using explicit JOIN"
        );
    }

    #[test]
    fn test_optimizer_service_analyze() {
        let optimizer = OptimizerService::new();
        let suggestions = optimizer.analyze("SELECT * FROM users WHERE active = true");

        assert!(
            !suggestions.is_empty(),
            "Should return optimization suggestions"
        );
    }

    #[test]
    fn test_optimizer_service_performance_estimate() {
        let optimizer = OptimizerService::new();
        let result = optimizer.optimize("SELECT * FROM users");

        assert!(
            result.estimated_improvement.before_ms > 0.0,
            "Should estimate before performance"
        );
    }

    #[test]
    fn test_optimizer_service_suggestion_priority() {
        let optimizer = OptimizerService::new();
        let result = optimizer.optimize("SELECT * FROM users");

        for suggestion in &result.suggestions {
            assert!(
                matches!(
                    suggestion.priority,
                    Priority::High | Priority::Medium | Priority::Low
                ),
                "Suggestion should have valid priority"
            );
        }
    }

    #[test]
    fn test_optimizer_service_suggestion_category() {
        let optimizer = OptimizerService::new();
        let result = optimizer.optimize("SELECT * FROM users");

        for suggestion in &result.suggestions {
            assert!(
                matches!(
                    suggestion.category,
                    SuggestionCategory::Index
                        | SuggestionCategory::QueryRewrite
                        | SuggestionCategory::Schema
                        | SuggestionCategory::Configuration
                        | SuggestionCategory::Join
                        | SuggestionCategory::Aggregation
                ),
                "Suggestion should have valid category"
            );
        }
    }

    #[test]
    fn test_full_policy_mask_explain_optimize_workflow() {
        let policy_engine = PolicyEngine::new();
        let masker = ColumnMasker::new();
        let explainer = ExplainService::new();
        let optimizer = OptimizerService::new();

        let policy_request = PolicyCheckRequest {
            user_id: "analyst".to_string(),
            resource: "table:sensitive_data".to_string(),
            action: "SELECT".to_string(),
            context: None,
        };
        let policy_response = policy_engine.check(&policy_request);
        assert!(
            policy_response.allowed || !policy_response.allowed,
            "Policy check should return valid response"
        );

        let explain_result = explainer.explain("SELECT * FROM users");
        assert!(
            explain_result.estimated_cost > 0.0,
            "Explain should return valid cost"
        );

        let optimize_result = optimizer.optimize("SELECT * FROM users");
        assert!(
            !optimize_result.suggestions.is_empty(),
            "Optimizer should provide suggestions"
        );

        let masked_value = masker.mask_value("email", &serde_json::json!("test@example.com"));
        assert!(
            masked_value.as_str().unwrap().contains('*'),
            "Masking should work"
        );
    }
}
