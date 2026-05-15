## ADDED Requirements

### Requirement: Create Evidence API
The system SHALL provide API to create new evidence records.

#### Scenario: Create new evidence chain
- **WHEN** create_evidence() is called with chain_id, description, and initial content
- **THEN** a new EvidenceChain is created, persisted, and returned

### Requirement: Query Evidence API
The system SHALL provide API to query evidence records.

#### Scenario: Get evidence by ID
- **WHEN** get_evidence() is called with a chain_id
- **THEN** the corresponding EvidenceChain is returned

#### Scenario: List evidence by type
- **WHEN** list_evidence() is called with node_type filter
- **THEN** all evidence chains containing nodes of that type are returned

### Requirement: Verify Evidence API
The system SHALL provide API to verify evidence integrity.

#### Scenario: Verify and report
- **WHEN** verify_evidence() is called with a chain_id
- **THEN** verification is performed and a detailed report is returned

### Requirement: Evidence and Signature Integration
The system SHALL require electronic signature for creating critical evidence records.

#### Scenario: Create signed evidence
- **WHEN** create_signed_evidence() is called
- **THEN** an electronic signature is required and the evidence chain is linked to the signature record