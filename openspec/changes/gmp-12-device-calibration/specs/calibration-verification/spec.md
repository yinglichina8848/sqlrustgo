# Calibration Verification

## ADDED Requirements

### Requirement: Verify calibration before measurement
The system SHALL verify device calibration status before accepting measurements from mobile collection.

#### Scenario: Verification success
- **WHEN** a mobile device submits COLLECT DATA with device_id
- **THEN** the system checks the device's calibration status
- **AND** accepts the measurement if status=CURRENT
- **AND** includes calibration status in the collection record

#### Scenario: Verification failure - due calibration
- **WHEN** a mobile device submits COLLECT DATA with device_id
- **THEN** the system finds the device status=DUE
- **AND** accepts the measurement with a warning
- **AND** flags the measurement as requiring review

#### Scenario: Verification failure - expired calibration
- **WHEN** a mobile device submits COLLECT DATA with device_id
- **THEN** the system finds the device status=EXPIRED
- **AND** rejects the measurement with error DEVICE_CALIBRATION_EXPIRED
