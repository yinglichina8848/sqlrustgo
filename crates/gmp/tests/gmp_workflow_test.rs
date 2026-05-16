//! GMP Workflow Engine Tests (GMP-9)
//!
//! Tests for workflow state machine and approval chain

#[cfg(test)]
mod tests {
    use sqlrustgo_gmp::workflow::{
        ApprovalAction, ApprovalChain, ApprovalRecord, WorkflowEngine,
        WorkflowState,
    };
    use std::collections::HashMap;

    #[test]
    fn test_workflow_state_from_str() {
        assert_eq!(WorkflowState::from_str("draft"), Some(WorkflowState::Draft));
        assert_eq!(WorkflowState::from_str("review"), Some(WorkflowState::Review));
        assert_eq!(WorkflowState::from_str("approval"), Some(WorkflowState::Approval));
        assert_eq!(WorkflowState::from_str("released"), Some(WorkflowState::Released));
        assert_eq!(WorkflowState::from_str("rejected"), Some(WorkflowState::Rejected));
        assert_eq!(WorkflowState::from_str("invalid"), None);
    }

    #[test]
    fn test_workflow_engine_new() {
        let engine = WorkflowEngine::new();
        assert!(engine.list_definitions().is_empty());
        assert!(engine.list_instances().is_empty());
    }

    #[test]
    fn test_create_and_start_workflow() {
        let mut engine = WorkflowEngine::new();

        engine
            .create_definition(
                "test_wf".to_string(),
                vec!["draft", "review", "released"],
                Some(604800),
                2,
            )
            .unwrap();

        let mut ctx = HashMap::new();
        ctx.insert("batch_id".to_string(), serde_json::json!(123));

        let instance_id = engine.start_workflow("test_wf", ctx).unwrap();

        let instance = engine.get_instance(&instance_id).unwrap();
        assert_eq!(instance.definition_name, "test_wf");
        assert_eq!(instance.current_state, WorkflowState::Draft);
    }

    #[test]
    fn test_workflow_reject() {
        let mut engine = WorkflowEngine::new();

        engine
            .create_definition(
                "test".to_string(),
                vec!["draft", "review", "released"],
                None,
                1,
            )
            .unwrap();

        let instance_id = engine.start_workflow("test", HashMap::new()).unwrap();

        engine
            .reject(&instance_id, "Not approved".to_string())
            .unwrap();

        let instance = engine.get_instance(&instance_id).unwrap();
        assert_eq!(instance.current_state, WorkflowState::Rejected);
    }

    #[test]
    fn test_approval_chain_creation() {
        let chain = ApprovalChain::new("instance1".to_string(), 2);
        assert!(!chain.is_complete());
    }

    #[test]
    fn test_approval_record_creation() {
        let record = ApprovalRecord::new(
            "instance1".to_string(),
            "review".to_string(),
            "user1".to_string(),
            "signature1".to_string(),
            ApprovalAction::Approve,
            None,
        );
        assert_eq!(record.approver_id, "user1");
        assert_eq!(record.action, ApprovalAction::Approve);
    }

    #[test]
    fn test_list_definitions_and_instances() {
        let mut engine = WorkflowEngine::new();

        engine
            .create_definition("def1".to_string(), vec!["draft", "released"], None, 1)
            .unwrap();

        let id1 = engine.start_workflow("def1", HashMap::new()).unwrap();

        assert_eq!(engine.list_definitions().len(), 1);
        assert_eq!(engine.list_instances().len(), 1);
        assert!(engine.list_instances().contains(&&id1));
    }
}