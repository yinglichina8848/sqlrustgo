## ADDED Requirements

### Requirement: Evidence Storage Table
The system SHALL create gmp_evidence_records table for persistent storage of evidence chains.

#### Scenario: Create evidence records table
- **WHEN** create_evidence_tables() is called
- **THEN** gmp_evidence_records table is created with columns: id, chain_id, description, nodes_json, integrity_hash, created_at, updated_at

### Requirement: Evidence Persistence
The system SHALL persist EvidenceChain to gmp_evidence_records table.

#### Scenario: Save evidence chain
- **WHEN** save_evidence_chain() is called with an EvidenceChain
- **THEN** the chain is serialized to JSON and stored in gmp_evidence_records

#### Scenario: Load evidence chain
- **WHEN** load_evidence_chain() is called with a chain_id
- **THEN** the EvidenceChain is retrieved from storage and deserialized

### Requirement: Evidence Index
The system SHALL support querying evidence by chain_id and timestamp range.

#### Scenario: Query by chain_id
- **WHEN** get_evidence_by_chain_id() is called
- **THEN** all evidence records matching the chain_id are returned

#### Scenario: Query by time range
- **WHEN** get_evidence_by_time_range() is called with from/to timestamps
- **THEN** all evidence records created within the range are returned