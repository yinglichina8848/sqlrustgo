# Calibration Record

## ADDED Requirements

### Requirement: Record calibration event
The system SHALL record calibration events linking devices to calibration results.

#### Scenario: Record successful calibration
- **WHEN** a calibration event is submitted with device_id, calibration_date, performed_by, standard_used, result=PASS, and next_calibration_due_date
- **THEN** the system creates a CalibrationRecord
- **AND** updates the device status to CURRENT
- **AND** sets the next calibration due date

#### Scenario: Record failed calibration
- **WHEN** a calibration event is submitted with result=FAIL
- **THEN** the system creates a CalibrationRecord with result=FAIL
- **AND** updates the device status to EXPIRED
- **AND** all measurements from this device should be rejected
