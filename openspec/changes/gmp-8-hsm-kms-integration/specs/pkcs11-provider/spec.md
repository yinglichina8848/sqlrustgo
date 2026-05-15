## ADDED Requirements

### Requirement: PKCS#11 Provider Interface
The system SHALL provide a PKCS#11 provider that implements HsmProvider trait.

#### Scenario: Create PKCS#11 provider instance
- **WHEN** Pkcs11Provider::new(slot_id, library_path) is called
- **THEN** provider connects to HSM slot and is ready

### Requirement: PKCS#11 Key Generation
The system SHALL generate keys using PKCS#11 API.

#### Scenario: Generate key via PKCS#11
- **WHEN** provider.generate_key(key_id) is called
- **THEN** key is generated in HSM via C_GenerateKey

### Requirement: PKCS#11 Signing
The system SHALL perform signing using PKCS#11 API.

#### Scenario: Sign data via PKCS#11
- **WHEN** provider.sign(key_id, data) is called
- **THEN** signature is computed via C_Sign and returned
