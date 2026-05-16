use super::state::WorkflowState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub instance_id: String,
    pub definition_name: String,
    pub current_state: WorkflowState,
    pub context: HashMap<String, serde_json::Value>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl WorkflowInstance {
    pub fn new(definition_name: String, context: HashMap<String, serde_json::Value>) -> Self {
        let now = current_timestamp();
        WorkflowInstance {
            instance_id: uuid_simple(),
            definition_name,
            current_state: WorkflowState::Draft,
            context,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn transition_to(&mut self, new_state: WorkflowState) {
        self.current_state = new_state;
        self.updated_at = current_timestamp();
    }

    pub fn set_context(&mut self, key: String, value: serde_json::Value) {
        self.context.insert(key, value);
        self.updated_at = current_timestamp();
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn uuid_simple() -> String {
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("wf-{:x}-{:x}", dur.as_secs(), dur.subsec_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_creation() {
        let mut ctx = HashMap::new();
        ctx.insert("batch_id".to_string(), serde_json::json!(123));

        let instance = WorkflowInstance::new("batch_release".to_string(), ctx);

        assert_eq!(instance.definition_name, "batch_release");
        assert_eq!(instance.current_state, WorkflowState::Draft);
        assert!(instance.instance_id.starts_with("wf-"));
        assert!(instance.created_at > 0);
        assert!(instance.updated_at > 0);
    }

    #[test]
    fn test_state_transition() {
        let instance = WorkflowInstance::new("test".to_string(), HashMap::new());
        assert_eq!(instance.current_state, WorkflowState::Draft);
    }

    #[test]
    fn test_set_context() {
        let mut ctx = HashMap::new();
        ctx.insert("key1".to_string(), serde_json::json!("value1"));

        let mut instance = WorkflowInstance::new("test".to_string(), ctx);
        instance.set_context("key2".to_string(), serde_json::json!("value2"));

        assert_eq!(
            instance.context.get("key1").unwrap(),
            &serde_json::json!("value1")
        );
        assert_eq!(
            instance.context.get("key2").unwrap(),
            &serde_json::json!("value2")
        );
    }

    #[test]
    fn test_transition_to() {
        let mut instance = WorkflowInstance::new("test".to_string(), HashMap::new());
        assert_eq!(instance.current_state, WorkflowState::Draft);

        instance.transition_to(WorkflowState::Review);
        assert_eq!(instance.current_state, WorkflowState::Review);
        assert!(instance.updated_at >= instance.created_at);
    }

    #[test]
    fn test_instance_context_update() {
        let mut ctx = HashMap::new();
        ctx.insert("initial".to_string(), serde_json::json!(true));

        let mut instance = WorkflowInstance::new("test".to_string(), ctx);
        let old_updated_at = instance.updated_at;

        instance.set_context("new_key".to_string(), serde_json::json!("new_value"));
        assert!(instance.updated_at >= old_updated_at);
    }
}
