## ADDED Requirements

### Requirement: Automatic Key Rotation
The system SHALL support automatic key rotation based on time.

#### Scenario: Automatic rotation after expiry
- **WHEN** key has exceeded rotation_period
- **THEN** system automatically generates new key and deprecates old key

### Requirement: Manual Key Rotation
The system SHALL support manually triggered key rotation.

#### Scenario: Trigger manual rotation
- **WHEN** rotate_key(key_id) is called
- **THEN** new key is generated and old key is marked deprecated

### Requirement: Key Rotation History
The system SHALL maintain history of all key rotations.

#### Scenario: Query rotation history
- **WHEN** get_key_history(key_id) is called
- **THEN** list of all key versions and rotation timestamps is returned
