use super::approval::{ApprovalAction, ApprovalChain, ApprovalRecord};
use super::definition::WorkflowDefinition;
use super::instance::WorkflowInstance;
use super::state::WorkflowState;
use super::timeout::TimeoutChecker;
use std::collections::HashMap;
use std::time::Duration;

pub struct WorkflowEngine {
    definitions: HashMap<String, WorkflowDefinition>,
    instances: HashMap<String, WorkflowInstance>,
    approval_chains: HashMap<String, ApprovalChain>,
    timeout_checker: TimeoutChecker,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        WorkflowEngine {
            definitions: HashMap::new(),
            instances: HashMap::new(),
            approval_chains: HashMap::new(),
            timeout_checker: TimeoutChecker::new(Duration::from_secs(60)),
        }
    }

    pub fn create_definition(
        &mut self,
        name: String,
        stages: Vec<&str>,
        timeout_secs: Option<u64>,
        required_signatures: u32,
    ) -> Result<(), String> {
        let def = WorkflowDefinition::new(name.clone(), stages, timeout_secs, required_signatures)?;
        self.definitions.insert(name, def);
        Ok(())
    }

    pub fn start_workflow(
        &mut self,
        definition_name: &str,
        context: HashMap<String, serde_json::Value>,
    ) -> Result<String, String> {
        let definition = self
            .definitions
            .get(definition_name)
            .ok_or_else(|| "Workflow definition not found".to_string())?;

        let instance = WorkflowInstance::new(definition_name.to_string(), context);
        let instance_id = instance.instance_id.clone();

        let approval_chain =
            ApprovalChain::new(instance_id.clone(), definition.required_signatures);
        self.approval_chains
            .insert(instance_id.clone(), approval_chain);

        self.instances.insert(instance_id.clone(), instance);

        Ok(instance_id)
    }

    pub fn get_instance(&self, instance_id: &str) -> Option<&WorkflowInstance> {
        self.instances.get(instance_id)
    }

    pub fn approve(
        &mut self,
        instance_id: &str,
        stage: &str,
        approver_id: &str,
        signature: &str,
        comment: Option<String>,
    ) -> Result<(), String> {
        let instance = self
            .instances
            .get_mut(instance_id)
            .ok_or_else(|| "Instance not found".to_string())?;

        let chain = self
            .approval_chains
            .get_mut(instance_id)
            .ok_or_else(|| "Approval chain not found".to_string())?;

        let record = ApprovalRecord::new(
            instance_id.to_string(),
            stage.to_string(),
            approver_id.to_string(),
            signature.to_string(),
            ApprovalAction::Approve,
            comment,
        );

        chain.add_approval(record)?;

        if chain.is_complete() {
            let definition = self
                .definitions
                .get(&instance.definition_name)
                .ok_or_else(|| "Definition not found".to_string())?;

            let sm = definition.state_machine();
            if let Some(next_state) = sm.get_valid_transitions(&instance.current_state).first() {
                instance.transition_to((*next_state).clone());
            }
        }

        Ok(())
    }

    pub fn reject(&mut self, instance_id: &str, _reason: String) -> Result<(), String> {
        let instance = self
            .instances
            .get_mut(instance_id)
            .ok_or_else(|| "Instance not found".to_string())?;

        instance.transition_to(WorkflowState::Rejected);

        Ok(())
    }

    pub fn check_timeouts(&mut self) -> Vec<String> {
        let mut expired_ids = Vec::new();

        for (id, instance) in &self.instances {
            if let Some(def) = self.definitions.get(&instance.definition_name) {
                if let Some(timeout) = def.timeout {
                    if self.timeout_checker.is_expired(instance, timeout) {
                        expired_ids.push(id.clone());
                    }
                }
            }
        }

        for id in &expired_ids {
            if let Some(instance) = self.instances.get_mut(id) {
                instance.transition_to(WorkflowState::Rejected);
            }
        }

        expired_ids
    }

    pub fn list_definitions(&self) -> Vec<&String> {
        self.definitions.keys().collect()
    }

    pub fn list_instances(&self) -> Vec<&String> {
        self.instances.keys().collect()
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_start_workflow() {
        let mut engine = WorkflowEngine::new();

        engine
            .create_definition(
                "batch_release".to_string(),
                vec!["draft", "review", "approval", "released"],
                Some(604800),
                2,
            )
            .unwrap();

        let mut ctx = HashMap::new();
        ctx.insert("batch_id".to_string(), serde_json::json!(123));

        let instance_id = engine.start_workflow("batch_release", ctx).unwrap();

        let instance = engine.get_instance(&instance_id).unwrap();
        assert_eq!(instance.definition_name, "batch_release");
        assert_eq!(instance.current_state, WorkflowState::Draft);
    }

    #[test]
    fn test_reject_workflow() {
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
    fn test_workflow_engine_new() {
        let engine = WorkflowEngine::new();
        assert!(engine.list_definitions().is_empty());
        assert!(engine.list_instances().is_empty());
    }

    #[test]
    fn test_create_definition_error() {
        let mut engine = WorkflowEngine::new();

        let result =
            engine.create_definition("test".to_string(), vec!["draft", "invalid_stage"], None, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_start_workflow_definition_not_found() {
        let mut engine = WorkflowEngine::new();
        let result = engine.start_workflow("nonexistent", HashMap::new());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Workflow definition not found");
    }

    #[test]
    fn test_approve_single_signature() {
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
            .approve(&instance_id, "review", "user-1", "signature-1", None)
            .unwrap();

        let instance = engine.get_instance(&instance_id).unwrap();
        assert_eq!(instance.current_state, WorkflowState::Review);
    }

    #[test]
    fn test_approve_instance_not_found() {
        let mut engine = WorkflowEngine::new();
        let result = engine.approve("nonexistent", "review", "user-1", "sig", None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Instance not found");
    }

    #[test]
    fn test_reject_instance_not_found() {
        let mut engine = WorkflowEngine::new();
        let result = engine.reject("nonexistent", "reason".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Instance not found");
    }

    #[test]
    fn test_check_timeouts_no_timeout() {
        let mut engine = WorkflowEngine::new();

        engine
            .create_definition(
                "test".to_string(),
                vec!["draft", "review", "released"],
                None,
                1,
            )
            .unwrap();

        engine.start_workflow("test", HashMap::new()).unwrap();

        let expired = engine.check_timeouts();
        assert!(expired.is_empty());
    }

    #[test]
    fn test_check_timeouts_no_timeout_set() {
        let mut engine = WorkflowEngine::new();

        engine
            .create_definition(
                "test".to_string(),
                vec!["draft", "review", "released"],
                None,
                1,
            )
            .unwrap();

        let _instance_id = engine.start_workflow("test", HashMap::new()).unwrap();

        let expired = engine.check_timeouts();
        assert!(expired.is_empty());
    }

    #[test]
    fn test_list_definitions() {
        let mut engine = WorkflowEngine::new();

        engine
            .create_definition("def1".to_string(), vec!["draft", "released"], None, 1)
            .unwrap();

        engine
            .create_definition(
                "def2".to_string(),
                vec!["draft", "review", "released"],
                None,
                1,
            )
            .unwrap();

        let defs = engine.list_definitions();
        assert_eq!(defs.len(), 2);
        assert!(defs.contains(&&"def1".to_string()));
        assert!(defs.contains(&&"def2".to_string()));
    }

    #[test]
    fn test_list_instances() {
        let mut engine = WorkflowEngine::new();

        engine
            .create_definition("test".to_string(), vec!["draft", "released"], None, 1)
            .unwrap();

        let id1 = engine.start_workflow("test", HashMap::new()).unwrap();
        let id2 = engine.start_workflow("test", HashMap::new()).unwrap();

        let instances = engine.list_instances();
        assert_eq!(instances.len(), 2);
        assert!(instances.contains(&&id1));
        assert!(instances.contains(&&id2));
    }

    #[test]
    fn test_workflow_engine_default() {
        let engine = WorkflowEngine::default();
        assert!(engine.list_definitions().is_empty());
    }
}
