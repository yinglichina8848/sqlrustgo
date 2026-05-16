# Mobile Device Registration

## ADDED Requirements

### Requirement: Device registration with certificate
The system SHALL allow mobile devices to register with a unique device ID and X.509 certificate for GMP-compliant data collection.

#### Scenario: Successful device registration
- **WHEN** a mobile device submits REGISTER DEVICE with device_id, certificate_fingerprint, and public_key
- **THEN** the system creates a MobileDevice record with status=REGISTERED
- **AND** returns a registration receipt with timestamp

#### Scenario: Device re-registration
- **WHEN** an already registered device submits REGISTER DEVICE
- **THEN** the system updates the existing record with new certificate if provided
- **AND** preserves the original registration timestamp

### Requirement: Device status tracking
The system SHALL track device status: REGISTERED, SUSPENDED, REVOKED.

#### Scenario: Device suspension
- **WHEN** an administrator issues SUSPEND DEVICE for a registered device
- **THEN** the device status changes to SUSPENDED
- **AND** the device cannot submit new collections until reactivated

#### Scenario: Device revocation
- **WHEN** an administrator issues REVOKE DEVICE for a device
- **THEN** the device status changes to REVOKED
- **AND** all collection records from this device remain valid (revocation is for future collections only)
