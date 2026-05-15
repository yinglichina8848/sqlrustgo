# Device Calibration Management

## ADDED Requirements

### Requirement: Device registration with calibration interval
The system SHALL allow registering measurement devices with calibration intervals.

#### Scenario: Register calibration device
- **WHEN** an administrator submits REGISTER CALIBRATION DEVICE with device_id, device_type, calibration_interval_days, and tolerance_criteria
- **THEN** the system creates a CalibrationDevice record with status=CURRENT
- **AND** the device is tracked for calibration tracking

### Requirement: Device calibration status
The system SHALL track device calibration status: CURRENT, DUE, EXPIRED.

#### Scenario: Device becomes due
- **WHEN** a device's calibration_interval_days have passed since last calibration
- **THEN** the device status changes to DUE
- **AND** measurements from this device should be flagged

#### Scenario: Device becomes expired
- **WHEN** a device remains uncalibrated past its calibration interval
- **THEN** the device status changes to EXPIRED
- **AND** measurements from this device should be rejected
