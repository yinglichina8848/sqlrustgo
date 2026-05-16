# Calibration Audit

## ADDED Requirements

### Requirement: Calibration audit trail
The system SHALL maintain a complete audit trail of all calibration events.

#### Scenario: Audit trail for device
- **WHEN** an auditor queries calibration history for a device
- **THEN** the system returns all CalibrationRecords in chronological order
- **AND** each record includes calibration_date, performed_by, result, standard_used

### Requirement: Measurement traceability to calibration
The system SHALL link measurements to the calibration record valid at measurement time.

#### Scenario: Trace measurement to calibration
- **WHEN** a measurement is queried by audit_id
- **THEN** the system returns the calibration status at collection time
- **AND** the calibration record that was valid at that time
