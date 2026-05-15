## 1. Calibration Module Foundation

- [x] 1.1 Create `crates/gmp/src/calibration/` directory structure
- [x] 1.2 Add `mod calibration` to `crates/gmp/src/lib.rs`
- [x] 1.3 Create `calibration/device.rs` with CalibrationDevice struct
- [x] 1.4 Create `calibration/record.rs` with CalibrationRecord struct
- [x] 1.5 Create `calibration/status.rs` with calibration status enum

## 2. Parser Integration

- [x] 2.1 Add REGISTER CALIBRATION DEVICE, RECORD CALIBRATION tokens to lexer
- [x] 2.2 Add statement types to parser
- [x] 2.3 Implement parse functions for calibration statements
- [x] 2.4 Add calibration statements to Statement enum

## 3. Executor Integration

- [x] 3.1 Add `execute_register_calibration_device()` to ExecutionEngine
- [x] 3.2 Add `execute_record_calibration()` to ExecutionEngine
- [x] 3.3 Integrate calibration statement handling in `execute_statement()`

## 4. Calibration Verification

- [ ] 4.1 Implement calibration status checking
- [ ] 4.2 Integrate verification in GMP-10 mobile collection
- [ ] 4.3 Add error types for DEVICE_CALIBRATION_EXPIRED, DEVICE_CALIBRATION_DUE

## 5. Testing

- [ ] 5.1 Add unit test: `test_device_registration` - calibration device CRUD
- [ ] 5.2 Add unit test: `test_calibration_record_pass` - successful calibration
- [ ] 5.3 Add unit test: `test_calibration_record_fail` - failed calibration
- [ ] 5.4 Add unit test: `test_calibration_status_due` - status transitions to DUE
- [ ] 5.5 Add unit test: `test_calibration_status_expired` - status transitions to EXPIRED
- [ ] 5.6 Add unit test: `test_verification_success` - measurement accepted with CURRENT device
- [ ] 5.7 Add unit test: `test_verification_expired` - measurement rejected with EXPIRED device
- [ ] 5.8 Add integration test: `gmp_calibration_test`

## 6. Integration and Verification

- [ ] 6.1 Run `cargo build --all-features` - verify no build errors
- [ ] 6.2 Run `cargo clippy --all-features -- -D warnings` - zero warnings
- [ ] 6.3 Run `cargo test -p sqlrustgo-gmp --lib -- --test-threads=1` - all GMP tests pass
- [ ] 6.4 Run full test suite to ensure no regressions
