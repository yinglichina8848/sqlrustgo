use super::state::{StateMachine, WorkflowState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub stages: Vec<WorkflowState>,
    pub timeout: Option<u64>,
    pub required_signatures: u32,
}

impl WorkflowDefinition {
    pub fn new(
        name: String,
        stages: Vec<&str>,
        timeout_secs: Option<u64>,
        required_signatures: u32,
    ) -> Result<Self, String> {
        let states: Vec<WorkflowState> = stages
            .iter()
            .filter_map(|s| WorkflowState::from_str(s))
            .collect();

        if states.len() != stages.len() {
            return Err("Invalid stage name".to_string());
        }

        Ok(WorkflowDefinition {
            name,
            stages: states,
            timeout: timeout_secs,
            required_signatures,
        })
    }

    pub fn state_machine(&self) -> StateMachine {
        let mut transitions = Vec::new();
        for window in self.stages.windows(2) {
            transitions.push(super::state::WorkflowTransition::new(
                window[0].clone(),
                window[1].clone(),
            ));
        }
        StateMachine::new(self.stages.clone(), transitions)
    }

    pub fn is_valid_transition(&self, from: &WorkflowState, to: &WorkflowState) -> bool {
        let sm = self.state_machine();
        sm.can_transition(from, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition_creation() {
        let def = WorkflowDefinition::new(
            "batch_release".to_string(),
            vec!["draft", "review", "approval", "released"],
            Some(604800),
            2,
        )
        .unwrap();

        assert_eq!(def.name, "batch_release");
        assert_eq!(def.stages.len(), 4);
        assert_eq!(def.required_signatures, 2);
        assert_eq!(def.timeout, Some(604800));
    }

    #[test]
    fn test_valid_transition() {
        let def = WorkflowDefinition::new(
            "test".to_string(),
            vec!["draft", "review", "released"],
            None,
            1,
        )
        .unwrap();

        assert!(def.is_valid_transition(&WorkflowState::Draft, &WorkflowState::Review));
        assert!(!def.is_valid_transition(&WorkflowState::Draft, &WorkflowState::Released));
    }

    #[test]
    fn test_definition_with_invalid_stage() {
        let def = WorkflowDefinition::new(
            "test".to_string(),
            vec!["draft", "invalid_stage", "released"],
            None,
            1,
        );
        assert!(def.is_err());
        assert_eq!(def.unwrap_err(), "Invalid stage name");
    }

    #[test]
    fn test_definition_without_timeout() {
        let def = WorkflowDefinition::new(
            "test".to_string(),
            vec!["draft", "review", "released"],
            None,
            1,
        )
        .unwrap();

        assert_eq!(def.timeout, None);
    }

    #[test]
    fn test_state_machine() {
        let def = WorkflowDefinition::new(
            "test".to_string(),
            vec!["draft", "review", "released"],
            None,
            1,
        )
        .unwrap();

        let sm = def.state_machine();
        assert!(sm.can_transition(&WorkflowState::Draft, &WorkflowState::Review));
    }
}
