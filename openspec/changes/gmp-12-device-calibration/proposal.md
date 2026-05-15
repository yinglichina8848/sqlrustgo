## Why

GMP-regulated production requires that measurement devices (scales, thermometers, analyzers) be calibrated at defined intervals. Measurements taken with uncalibrated or expired devices may be invalid. This ensures all production measurements trace back to calibrated instrumentation with documented calibration history.

## What Changes

- **New**: `CalibrationDevice` entity for tracking measurement devices
- **New**: `CalibrationRecord` entity for calibration events
- **New**: Calibration intervals and expiry tracking
- **New**: Measurement verification against device calibration status
- **New**: SQL statements for calibration management
- **Modified**: Mobile collection (GMP-10) to verify device calibration before accepting measurements

## Capabilities

### New Capabilities

- `device-calibration-management`: Track measurement devices with calibration intervals and status
- `calibration-record`: Record calibration events with measurement standards and results
- `calibration-verification`: Verify device calibration status before accepting measurements
- `calibration-audit`: Full audit trail of calibration history for regulatory compliance

### Modified Capabilities

- `mobile-collection`: Mobile device collection (GMP-10) will verify device is calibrated before accepting data

## Impact

- **New module**: `crates/gmp/src/calibration/` with calibration management
- **Parser**: New statement types for calibration CRUD
- **Executor**: Calibration verification in data collection
- **Integration**: With GMP-10 mobile collection for device calibration checks
- **Tests**: `tests/gmp/calibration_test.rs`
