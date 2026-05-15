# Training Verification

## ADDED Requirements

### Requirement: Training verification at workflow start
The system SHALL verify operator training qualifications before allowing workflow execution to begin.

#### Scenario: Verification success
- **WHEN** an operator starts a workflow with SOP_BINDINGs
- **THEN** the system checks all TrainingRecords for the operator
- **AND** verifies training is VALID (not expired) for each required SOP
- **AND** allows workflow to start if all qualifications are met

#### Scenario: Verification failure - missing training
- **WHEN** an operator starts a workflow with SOP_BINDINGs
- **THEN** the system finds the operator lacks training for a required SOP
- **AND** rejects workflow start with error MISSING_TRAINING
- **AND** includes which SOP is missing

#### Scenario: Verification failure - expired training
- **WHEN** an operator starts a workflow with SOP_BINDINGs
- **THEN** the system finds the operator's training for a required SOP is expired
- **AND** rejects workflow start with error TRAINING_EXPIRED
- **AND** includes which SOP training needs renewal
