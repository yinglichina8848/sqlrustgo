## ADDED Requirements

### Requirement: Software TPM Simulator
The system SHALL provide a software-based TPM for development and testing.

#### Scenario: Create software TPM provider
- **WHEN** SoftwareTpmProvider::new() is called
- **THEN** provider is created with simulated TPM

### Requirement: Software TPM Key Operations
The system SHALL perform key operations in software.

#### Scenario: Generate key in software TPM
- **WHEN** provider.generate_key(key_id) is called
- **THEN** key is generated in memory with software simulation

### Requirement: Software TPM Signing
The system SHALL perform signing using software simulation.

#### Scenario: Sign with software TPM
- **WHEN** provider.sign(key_id, data) is called
- **THEN** signature is computed using software crypto and returned
