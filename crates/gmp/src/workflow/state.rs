use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowState {
    Draft,
    Review,
    Approval,
    Released,
    Rejected,
}

impl WorkflowState {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Some(WorkflowState::Draft),
            "review" => Some(WorkflowState::Review),
            "approval" => Some(WorkflowState::Approval),
            "released" => Some(WorkflowState::Released),
            "rejected" => Some(WorkflowState::Rejected),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            WorkflowState::Draft => "draft",
            WorkflowState::Review => "review",
            WorkflowState::Approval => "approval",
            WorkflowState::Released => "released",
            WorkflowState::Rejected => "rejected",
        }
    }
}

impl fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    pub from_state: WorkflowState,
    pub to_state: WorkflowState,
}

impl WorkflowTransition {
    pub fn new(from: WorkflowState, to: WorkflowState) -> Self {
        WorkflowTransition {
            from_state: from,
            to_state: to,
        }
    }

    pub fn is_valid_transition(&self, current: &WorkflowState) -> bool {
        &self.from_state == current
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StateMachine {
    states: Vec<WorkflowState>,
    transitions: Vec<WorkflowTransition>,
}

impl StateMachine {
    pub fn new(states: Vec<WorkflowState>, transitions: Vec<WorkflowTransition>) -> Self {
        StateMachine {
            states,
            transitions,
        }
    }

    pub fn from_stages(stages: Vec<&str>) -> Option<Self> {
        let states: Vec<WorkflowState> = stages
            .iter()
            .filter_map(|s| WorkflowState::from_str(s))
            .collect();

        if states.len() != stages.len() {
            return None;
        }

        let mut transitions = Vec::new();
        for window in states.windows(2) {
            transitions.push(WorkflowTransition::new(
                window[0].clone(),
                window[1].clone(),
            ));
        }

        Some(StateMachine {
            states,
            transitions,
        })
    }

    pub fn can_transition(&self, current: &WorkflowState, target: &WorkflowState) -> bool {
        self.transitions
            .iter()
            .any(|t| t.from_state == *current && t.to_state == *target)
    }

    pub fn transition_to(
        &self,
        current: &WorkflowState,
        target: &WorkflowState,
    ) -> Result<WorkflowState, String> {
        if self.can_transition(current, target) {
            Ok(target.clone())
        } else {
            Err(format!(
                "Invalid transition from {:?} to {:?}",
                current, target
            ))
        }
    }

    pub fn get_valid_transitions(&self, current: &WorkflowState) -> Vec<&WorkflowState> {
        self.transitions
            .iter()
            .filter(|t| t.from_state == *current)
            .map(|t| &t.to_state)
            .collect()
    }

    pub fn is_terminal_state(&self, state: &WorkflowState) -> bool {
        state == &WorkflowState::Released || state == &WorkflowState::Rejected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = WorkflowState::from_str("draft");
        assert_eq!(state, Some(WorkflowState::Draft));
    }

    #[test]
    fn test_state_from_str_all_variants() {
        assert_eq!(WorkflowState::from_str("draft"), Some(WorkflowState::Draft));
        assert_eq!(WorkflowState::from_str("review"), Some(WorkflowState::Review));
        assert_eq!(WorkflowState::from_str("approval"), Some(WorkflowState::Approval));
        assert_eq!(WorkflowState::from_str("released"), Some(WorkflowState::Released));
        assert_eq!(WorkflowState::from_str("rejected"), Some(WorkflowState::Rejected));
        assert_eq!(WorkflowState::from_str("invalid"), None);
    }

    #[test]
    fn test_state_as_str() {
        assert_eq!(WorkflowState::Draft.as_str(), "draft");
        assert_eq!(WorkflowState::Review.as_str(), "review");
        assert_eq!(WorkflowState::Approval.as_str(), "approval");
        assert_eq!(WorkflowState::Released.as_str(), "released");
        assert_eq!(WorkflowState::Rejected.as_str(), "rejected");
    }

    #[test]
    fn test_state_display() {
        let state = WorkflowState::Draft;
        assert_eq!(format!("{}", state), "draft");
    }

    #[test]
    fn test_state_machine_valid_transition() {
        let sm =
            StateMachine::from_stages(vec!["draft", "review", "approval", "released"]).unwrap();
        assert!(sm.can_transition(&WorkflowState::Draft, &WorkflowState::Review));
        assert!(!sm.can_transition(&WorkflowState::Draft, &WorkflowState::Released));
    }

    #[test]
    fn test_terminal_state() {
        let sm = StateMachine::from_stages(vec!["draft", "review", "released"]).unwrap();
        assert!(sm.is_terminal_state(&WorkflowState::Released));
        assert!(!sm.is_terminal_state(&WorkflowState::Draft));
        assert!(!sm.is_terminal_state(&WorkflowState::Review));
    }

    #[test]
    fn test_state_machine_from_stages_invalid() {
        let sm = StateMachine::from_stages(vec!["draft", "invalid", "released"]);
        assert!(sm.is_none());
    }

    #[test]
    fn test_state_machine_transition_to_valid() {
        let sm = StateMachine::from_stages(vec!["draft", "review", "released"]).unwrap();
        let result = sm.transition_to(&WorkflowState::Draft, &WorkflowState::Review);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), WorkflowState::Review);
    }

    #[test]
    fn test_state_machine_transition_to_invalid() {
        let sm = StateMachine::from_stages(vec!["draft", "review", "released"]).unwrap();
        let result = sm.transition_to(&WorkflowState::Draft, &WorkflowState::Released);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_valid_transitions() {
        let sm = StateMachine::from_stages(vec!["draft", "review", "approval", "released"]).unwrap();
        let transitions = sm.get_valid_transitions(&WorkflowState::Draft);
        assert_eq!(transitions.len(), 1);
        assert_eq!(*transitions[0], WorkflowState::Review);

        let no_transitions = sm.get_valid_transitions(&WorkflowState::Released);
        assert!(no_transitions.is_empty());
    }

    #[test]
    fn test_workflow_transition_is_valid() {
        let t = WorkflowTransition::new(WorkflowState::Draft, WorkflowState::Review);
        assert!(t.is_valid_transition(&WorkflowState::Draft));
        assert!(!t.is_valid_transition(&WorkflowState::Review));
    }
}
