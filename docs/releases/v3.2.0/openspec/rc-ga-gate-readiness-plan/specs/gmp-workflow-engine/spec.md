## ADDED Requirements

### Requirement: GMP Workflow State Machine
The system SHALL implement a state machine for GMP workflows.

#### Scenario: State transition
- **WHEN** workflow receives valid transition trigger
- **THEN** workflow moves to next state

#### Scenario: Invalid transition
- **WHEN** workflow receives invalid transition trigger
- **THEN** transition is rejected with error
