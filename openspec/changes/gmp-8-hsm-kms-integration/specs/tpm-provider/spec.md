## ADDED Requirements

### Requirement: TPM Provider Interface
The system SHALL provide a TPM provider that implements HsmProvider trait.

#### Scenario: Create TPM provider instance
- **WHEN** TpmProvider::new() is called with valid config
- **THEN** provider instance is created and ready to use

### Requirement: TPM Key Generation
The system SHALL generate keys in TPM hardware.

#### Scenario: Generate signing key in TPM
- **WHEN** provider.generate_key(key_id) is called
- **THEN** key is generated in TPM and key handle is returned

### Requirement: TPM Signing
The system SHALL perform signing operations using TPM.

#### Scenario: Sign data with TPM
- **WHEN** provider.sign(key_id, data) is called
- **THEN** signature is computed inside TPM and returned
