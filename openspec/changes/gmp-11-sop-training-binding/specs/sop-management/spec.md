# SOP Management

## ADDED Requirements

### Requirement: SOP CRUD operations
The system SHALL allow creating, reading, updating, and deactivating Standard Operating Procedures.

#### Scenario: Create SOP
- **WHEN** an administrator submits CREATE SOP with name, version, description, and qualification_requirements
- **THEN** the system creates an SOP record with status=ACTIVE
- **AND** returns an SOP ID

#### Scenario: SOP version control
- **WHEN** a new version of an existing SOP is created
- **THEN** the previous version remains available for historical audit
- **AND** the new version becomes the active version

### Requirement: SOP qualification requirements
The system SHALL define qualification requirements for each SOP.

#### Scenario: SOP with qualification
- **WHEN** an SOP is created with qualification_requirements
- **THEN** operators must have training completions matching these requirements to be considered qualified
