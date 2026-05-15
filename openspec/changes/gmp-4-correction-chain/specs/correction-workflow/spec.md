## ADDED Requirements

### Requirement: Correction Requires Approval
The system SHALL require approval before a correction can be applied.

#### Scenario: Correction rejected without approval
- **WHEN** user submits correction without required approvals
- **THEN** system rejects the correction with approval_required error

### Requirement: Approval Policy Integration
The system SHALL use ApprovalPolicyEvaluator to determine required approvals based on correction type.

#### Scenario: High-risk correction requires multiple approvers
- **WHEN** correction involves sensitive data fields
- **THEN** system requires signatures from multiple authorized personnel

### Requirement: Authorization Verification
The system SHALL verify that authorized_by field matches actual approver.

#### Scenario: Unauthorized correction attempt
- **WHEN** user attempts to set authorized_by to a user who did not approve
- **THEN** system rejects the correction
