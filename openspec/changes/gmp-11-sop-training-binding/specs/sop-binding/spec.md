# SOP Binding

## ADDED Requirements

### Requirement: Workflow step to SOP binding
The system SHALL allow binding workflow steps (from GMP-9) to required SOP qualifications.

#### Scenario: Bind workflow step to SOP
- **WHEN** an administrator creates a SOP_BINDING with workflow_id, step_id, required_sop_id
- **THEN** any execution of that workflow step requires the operator to have valid training for the required SOP

### Requirement: Multiple SOP requirements per step
The system SHALL support multiple SOP requirements per workflow step (AND logic).

#### Scenario: Step requires multiple SOPs
- **WHEN** a workflow step has multiple SOP_BINDINGs
- **THEN** the operator must have valid training for ALL required SOPs
