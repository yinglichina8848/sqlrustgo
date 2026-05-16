use super::instance::WorkflowInstance;
use super::state::WorkflowState;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct TimeoutChecker {
    #[allow(dead_code)]
    check_interval: Duration,
}

impl TimeoutChecker {
    pub fn new(check_interval: Duration) -> Self {
        TimeoutChecker { check_interval }
    }

    pub fn check_timeouts(&self, instances: &[WorkflowInstance], timeout_secs: u64) -> Vec<String> {
        let mut expired_ids = Vec::new();
        let now = current_timestamp();

        for instance in instances {
            if self.is_active_state(&instance.current_state) {
                let elapsed = now.saturating_sub(instance.updated_at);
                if elapsed > timeout_secs {
                    expired_ids.push(instance.instance_id.clone());
                }
            }
        }

        expired_ids
    }

    pub fn is_expired(&self, instance: &WorkflowInstance, timeout_secs: u64) -> bool {
        if !self.is_active_state(&instance.current_state) {
            return false;
        }

        let elapsed = current_timestamp().saturating_sub(instance.updated_at);
        elapsed > timeout_secs
    }

    fn is_active_state(&self, state: &WorkflowState) -> bool {
        matches!(
            state,
            WorkflowState::Draft | WorkflowState::Review | WorkflowState::Approval
        )
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_timeout_checker() {
        let checker = TimeoutChecker::new(Duration::from_secs(60));

        let mut ctx = HashMap::new();
        ctx.insert("batch_id".to_string(), serde_json::json!(1));

        let instance = WorkflowInstance::new("test".to_string(), ctx);

        let timeout_secs = 604800;
        assert!(!checker.is_expired(&instance, timeout_secs));
    }

    #[test]
    fn test_active_states() {
        let checker = TimeoutChecker::new(Duration::from_secs(60));

        assert!(checker.is_active_state(&WorkflowState::Draft));
        assert!(checker.is_active_state(&WorkflowState::Review));
        assert!(checker.is_active_state(&WorkflowState::Approval));
        assert!(!checker.is_active_state(&WorkflowState::Released));
        assert!(!checker.is_active_state(&WorkflowState::Rejected));
    }

    #[test]
    fn test_check_timeouts_empty() {
        let checker = TimeoutChecker::new(Duration::from_secs(60));
        let instances = vec![];
        let expired = checker.check_timeouts(&instances, 60);
        assert!(expired.is_empty());
    }

    #[test]
    fn test_check_timeouts_no_expired() {
        let checker = TimeoutChecker::new(Duration::from_secs(60));

        let mut ctx = HashMap::new();
        ctx.insert("batch_id".to_string(), serde_json::json!(1));

        let instance = WorkflowInstance::new("test".to_string(), ctx);
        let instances = vec![instance];

        let expired = checker.check_timeouts(&instances, 604800);
        assert!(expired.is_empty());
    }

    #[test]
    fn test_timeout_checker_new() {
        let checker = TimeoutChecker::new(Duration::from_secs(120));
        assert!(checker.check_interval.as_secs() == 120);
    }

    #[test]
    fn test_is_expired_rejected_state() {
        let checker = TimeoutChecker::new(Duration::from_secs(60));

        let mut ctx = HashMap::new();
        ctx.insert("batch_id".to_string(), serde_json::json!(1));

        let mut instance = WorkflowInstance::new("test".to_string(), ctx);
        instance.transition_to(WorkflowState::Rejected);

        assert!(!checker.is_expired(&instance, 0));
    }
}
