## 1. Performance Schema Infrastructure

- [x] 1.1 Add setup_actors struct and row type to performance_schema.rs
- [x] 1.2 Add setup_instruments struct and row type to performance_schema.rs
- [x] 1.3 Add events_statements_current struct and row type
- [x] 1.4 Add events_statements_history struct and row type
- [x] 1.5 Add events_statements_summary_by_digest struct and row type

## 2. Wait Events

- [x] 2.1 Add events_waits_current struct and row type
- [x] 2.2 Add events_waits_history struct and row type
- [x] 2.3 Implement ring buffer for wait events

## 3. Global Events

- [x] 3.1 Add global_events struct and row type
- [x] 3.2 Implement global event aggregation

## 4. Information Schema Integration

- [x] 4.1 Add PS tables to information_schema tables list
- [x] 4.2 Implement SELECT query handling for PS tables
- [x] 4.3 Add PS table definitions to information_schema.rs

## 5. Testing

- [x] 5.1 Add unit test: `test_ps_setup_actors` - setup_actors table
- [x] 5.2 Add unit test: `test_ps_setup_instruments` - setup_instruments table
- [x] 5.3 Add unit test: `test_ps_events_statements` - statement events
- [x] 5.4 Add unit test: `test_ps_events_waits` - wait events
- [x] 5.5 Add unit test: `test_ps_global_events` - global aggregation
- [x] 5.6 Run ps_* integration tests

## 6. Integration and Verification

- [x] 6.1 Run `cargo build --all-features` - verify no build errors
- [x] 6.2 Run `cargo clippy --all-features -- -D warnings` - zero warnings
- [x] 6.3 Run `cargo test -p sqlrustgo-information-schema --lib -- --test-threads=1` - all IS tests pass
- [x] 6.4 Run full test suite to ensure no regressions
