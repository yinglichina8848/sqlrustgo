## 1. Mobile Module Foundation

- [x] 1.1 Create `crates/gmp/src/mobile/` directory structure
- [x] 1.2 Add `mod mobile` to `crates/gmp/src/lib.rs`
- [x] 1.3 Create `mobile/device.rs` with MobileDevice struct
- [x] 1.4 Create `mobile/collection.rs` with MobileCollection struct
- [x] 1.5 Create `mobile/trust.rs` with device verification logic

## 2. Parser Integration

- [x] 2.1 Add REGISTER DEVICE, COLLECT DATA, DEVICE HEARTBEAT tokens to lexer
- [x] 2.2 Add statement types to parser (RegisterDevice, CollectData, DeviceHeartbeat)
- [x] 2.3 Implement parse functions for mobile statements
- [x] 2.4 Add mobile statements to Statement enum

## 3. Executor Integration

- [x] 3.1 Add `execute_register_device()` to ExecutionEngine
- [x] 3.2 Add `execute_collect_data()` to ExecutionEngine
- [x] 3.3 Add `execute_device_heartbeat()` to ExecutionEngine
- [x] 3.4 Integrate mobile statement handling in `execute_statement()`

## 4. Mobile Collection Audit

- [ ] 4.1 Create `mobile_collection_audit` table schema
- [ ] 4.2 Implement audit chain integration for mobile collections
- [ ] 4.3 Add trusted timestamp (GMP-6) integration to collections

## 5. Device Trust Verification

- [ ] 5.1 Implement device signature verification
- [ ] 5.2 Add device status checking (REGISTERED, SUSPENDED, REVOKED)
- [ ] 5.3 Implement certificate fingerprint validation

## 6. Testing

- [ ] 6.1 Add unit test: `test_device_registration` - basic device registration
- [ ] 6.2 Add unit test: `test_device_suspension` - device suspension workflow
- [ ] 6.3 Add unit test: `test_mobile_collection_signature` - signature verification
- [ ] 6.4 Add unit test: `test_collection_from_suspended_device` - rejection
- [ ] 6.5 Add integration test: `gmp_mobile_test`

## 7. Integration and Verification

- [ ] 7.1 Run `cargo build --all-features` - verify no build errors
- [ ] 7.2 Run `cargo clippy --all-features -- -D warnings` - zero warnings
- [ ] 7.3 Run `cargo test -p sqlrustgo-gmp --lib -- --test-threads=1` - all GMP tests pass
- [ ] 7.4 Run full test suite to ensure no regressions
