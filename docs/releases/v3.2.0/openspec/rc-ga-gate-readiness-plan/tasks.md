## 1. RC Gate 核心检查 (R1-R5)

- [ ] 1.1 Ensure `cargo build --release` succeeds
- [ ] 1.2 Ensure `cargo test --lib` ≥90% pass rate
- [ ] 1.3 Ensure `cargo clippy --all-features -- -D warnings` passes
- [ ] 1.4 Ensure `cargo fmt --check` passes
- [ ] 1.5 Ensure `cargo llvm-cov` ≥85% coverage

## 2. RC Gate SQL 兼容性 (R7-R8)

- [ ] 2.1 Implement MERGE statement (SQL Compat R7)
- [ ] 2.2 Implement Event Scheduler (SQL Compat R8)

## 3. RC Gate GMP 功能 (R9-R12)

- [ ] 3.1 Implement GMP Workflow State Machine (R9)
- [ ] 3.2 Implement Mobile Trusted Collection Protocol (R10)
- [ ] 3.3 Implement SOP/Training Binding Check (R11)
- [ ] 3.4 Implement Device Calibration Management (R12)

## 4. RC Gate 性能测试 (R13-R15)

- [ ] 4.1 Implement TPC-H SF=10 support (R13)
- [ ] 4.2 Optimize Sysbench point_select ≥30K QPS (R14)
- [ ] 4.3 Implement 72h Stability Test (R15)

## 5. RC Gate 稳定性测试 (R-S1~R-S16)

- [ ] 5.1 Run `concurrency_stress_test` (R-S1)
- [ ] 5.2 Run `crash_recovery_test` (R-S2)
- [ ] 5.3 Run `long_run_stability_test` (R-S3)
- [ ] 5.4 Run `wal_integration_test` (R-S4)
- [ ] 5.5 Run `network_tcp_smoke_test` (R-S5)
- [ ] 5.6 Run `ssi_stress_test` (R-S6)
- [ ] 5.7 Run `wal_crash_recovery_test` (R-S7)
- [ ] 5.8 Run `audit_trail_test` (R-S8)
- [ ] 5.9 Run `gap_locking_e2e_test` (R-S9)
- [ ] 5.10 Run `digital_signature_test` (R-S10)
- [ ] 5.11 Run `immutable_record_test` (R-S11)
- [ ] 5.12 Run `correction_chain_test` (R-S12)
- [ ] 5.13 Run `provenance_tracking_test` (R-S13)
- [ ] 5.14 Run `workflow_engine_test` (R-S14)
- [ ] 5.15 Run `trusted_timestamp_test` (R-S15)
- [ ] 5.16 Run `hsm_integration_test` (R-S16)

## 6. GA Gate QA 增强 (G-QA1~G-QA10)

- [ ] 6.1 Run `check_electronic_signature.sh` (G-QA1)
- [ ] 6.2 Run `check_immutable_record.sh` (G-QA2)
- [ ] 6.3 Run `check_correction_chain.sh` (G-QA3)
- [ ] 6.4 Run `check_provenance.sh` (G-QA4)
- [ ] 6.5 Run `check_timestamp.sh` (G-QA5)
- [ ] 6.6 Run `check_workflow.sh` (G-QA6)
- [ ] 6.7 Run `check_hsm.sh` (G-QA7)
- [ ] 6.8 Run `check_digital_signature.sh` (G-QA8)
- [ ] 6.9 Run `check_four_eyes.sh` (G-QA9)
- [ ] 6.10 Run `check_mobile.sh` (G-QA10)

## 7. GA Gate 稳定性测试 (G-S1~G-S20)

- [ ] 7.1 Run `integration_test` (G-S1)
- [ ] 7.2 Run `sysbench point_select` ≥30K QPS (G-S2)
- [ ] 7.3 Run `wal_crash_recovery_test` (G-S3)
- [ ] 7.4 Run `long_run_stability` 72h (G-S4)
- [ ] 7.5 Run `signature_chain_test` (G-S5)
- [ ] 7.6 Run `electronic_signature_test` (G-S6)
- [ ] 7.7 Run `immutable_record_test` (G-S7)
- [ ] 7.8 Run `correction_chain_test` (G-S8)
- [ ] 7.9 Run `provenance_tracking_test` (G-S9)
- [ ] 7.10 Run `trusted_timestamp_test` (G-S10)
- [ ] 7.11 Run `hsm_integration_test` (G-S11)
- [ ] 7.12 Run `workflow_engine_test` (G-S12)
- [ ] 7.13 Run `four_eyes_test` (G-S13)
- [ ] 7.14 Run `device_binding_test` (G-S14)
- [ ] 7.15 Run `audit_trail_test` (G-S15)
- [ ] 7.16 Run `concurrency_stress_test` (G-S16)
- [ ] 7.17 Run `gap_locking_e2e_test` (G-S17)
- [ ] 7.18 Run `window_function_boundary_test` (G-S18)
- [ ] 7.19 Run `set_operation_test` (G-S19)
- [ ] 7.20 Run `ssi_stress_test` (G-S20)

## 8. Formal Proofs (G10)

- [ ] 8.1 Create TLA+ specs for core modules
- [ ] 8.2 Verify ≥30 TLA+ proofs pass
