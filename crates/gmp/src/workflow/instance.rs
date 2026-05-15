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
    }

    #[test]
    fn test_state_transition() {
        let instance = WorkflowInstance::new("test".to_string(), HashMap::new());
        assert_eq!(instance.current_state, WorkflowState::Draft);
    }
}
