## ADDED Requirements

### Requirement: EvidenceChain Activation
The system SHALL export and activate the EvidenceChain module from evidence.rs.

#### Scenario: Export EvidenceChain module
- **WHEN** GMP crate is compiled
- **THEN** EvidenceChain, EvidenceNode, EvidenceMetadata are publicly accessible

#### Scenario: EvidenceChain immutability
- **WHEN** an EvidenceChain is created with nodes
- **THEN** the chain SHALL be immutable and verify() returns true for untampered chains

### Requirement: Immutable Record Interface
The system SHALL provide an ImmutableRecord trait that wraps EvidenceChain for GMP workflows.

#### Scenario: Create immutable record
- **WHEN** create_immutable_record() is called with content and metadata
- **THEN** a new EvidenceChain is created with the content as the first node

#### Scenario: Add node to immutable record
- **WHEN** add_evidence() is called on an existing immutable record
- **THEN** a new EvidenceNode is appended and chain integrity hash is updated