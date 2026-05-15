## ADDED Requirements

### Requirement: Evidence Chain Verification
The system SHALL verify the integrity of an EvidenceChain using the stored integrity_hash.

#### Scenario: Verify untampered chain
- **WHEN** verify_evidence_chain() is called on an unmodified EvidenceChain
- **THEN** the method returns true and verification timestamp is recorded

#### Scenario: Detect tampered chain
- **WHEN** verify_evidence_chain() is called on a modified EvidenceChain
- **THEN** the method returns false and tamper alert is raised

### Requirement: Cross-Chain Verification
The system SHALL support verification between EvidenceChain and AuditChain.

#### Scenario: Verify evidence-audit linkage
- **WHEN** verify_cross_chain() is called with an EvidenceChain and AuditChain
- **THEN** both chains are verified and cross-references are validated

### Requirement: Incremental Verification
The system SHALL support incremental verification for performance on large chains.

#### Scenario: Incremental verify last N nodes
- **WHEN** incremental_verify() is called with last_n parameter
- **THEN** only the last N nodes are verified for integrity