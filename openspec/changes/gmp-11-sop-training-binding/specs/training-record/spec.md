# Training Record

## ADDED Requirements

### Requirement: Training record creation
The system SHALL record training completions linking operators to SOPs they have completed.

#### Scenario: Record training completion
- **WHEN** a training completion is submitted with operator_id, sop_id, sop_version, completion_date, and trainer_signature
- **THEN** the system creates a TrainingRecord with status=VALID
- **AND** the record is immutable (cannot be modified, only superseded)

### Requirement: Training record validity
The system SHALL track validity period for training records.

#### Scenario: Training with expiry date
- **WHEN** a TrainingRecord is created with expiry_date
- **THEN** the record is considered VALID until the expiry_date
- **AND** after expiry_date, the record is considered EXPIRED

#### Scenario: Training without expiry
- **WHEN** a TrainingRecord is created without expiry_date
- **THEN** the record never expires
