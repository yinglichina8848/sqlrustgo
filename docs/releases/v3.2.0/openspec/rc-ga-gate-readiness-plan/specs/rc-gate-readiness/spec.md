## ADDED Requirements

### Requirement: RC Gate R1 Build Success
The system SHALL pass `cargo build --release` without errors.

#### Scenario: Release build
- **WHEN** running `cargo build --release`
- **THEN** compilation succeeds with exit code 0

### Requirement: RC Gate R2 Test Pass Rate
The system SHALL achieve ≥90% test pass rate in `cargo test --lib`.

#### Scenario: Library tests
- **WHEN** running `cargo test --lib`
- **THEN** at least 90% of tests pass

### Requirement: RC Gate R3 Clippy Zero Warnings
The system SHALL pass `cargo clippy --all-features -- -D warnings` with zero warnings.

#### Scenario: Clippy linting
- **WHEN** running `cargo clippy --all-features -- -D warnings`
- **THEN** no warnings or errors are produced

### Requirement: RC Gate R4 Format Check
The system SHALL pass `cargo fmt --check`.

#### Scenario: Code formatting
- **WHEN** running `cargo fmt --check`
- **THEN** no formatting issues are reported

### Requirement: RC Gate R5 Coverage ≥85%
The system SHALL achieve ≥85% line coverage in `cargo llvm-cov`.

#### Scenario: Coverage measurement
- **WHEN** running `cargo llvm-cov`
- **THEN** line coverage is at least 85%

### Requirement: RC Gate R6 Security Audit
The system SHALL pass `cargo audit` with no vulnerabilities.

#### Scenario: Security scan
- **WHEN** running `cargo audit`
- **THEN** no security vulnerabilities are reported

### Requirement: RC Gate R7 SQL Compat MERGE
The system SHALL implement MERGE statement per SQL standard.

#### Scenario: MERGE statement
- **WHEN** executing MERGE INTO ... WHEN MATCHED/NOT MATCHED
- **THEN** the operation completes correctly

### Requirement: RC Gate R8 SQL Compat Event Scheduler
The system SHALL implement Event Scheduler functionality.

#### Scenario: Event scheduler
- **WHEN** creating and executing scheduled events
- **THEN** events execute at specified times

### Requirement: RC Gate R9 GMP Workflow State Machine
The system SHALL implement GMP workflow state machine.

#### Scenario: Workflow state transitions
- **WHEN** workflow moves through states
- **THEN** state transitions follow defined rules

### Requirement: RC Gate R10 GMP Mobile Trusted Collection
The system SHALL implement mobile trusted collection protocol.

#### Scenario: Mobile device binding
- **WHEN** mobile device submits data
- **THEN** device identity is verified

### Requirement: RC Gate R11 GMP SOP/Training Binding Check
The system SHALL verify SOP/training completion before operations.

#### Scenario: SOP binding verification
- **WHEN** operator attempts restricted operation
- **THEN** system verifies SOP training completion

### Requirement: RC Gate R12 GMP Device Calibration
The system SHALL implement device calibration management.

#### Scenario: Calibration tracking
- **WHEN** device calibration is recorded
- **THEN** calibration history is maintained

### Requirement: RC Gate R13 TPC-H SF=10
The system SHALL pass all 22 TPC-H queries at SF=10.

#### Scenario: TPC-H SF=10 execution
- **WHEN** running TPC-H SF=10 queries
- **THEN** all 22 queries return correct results without OOM

### Requirement: RC Gate R14 Sysbench point_select ≥30K QPS
The system SHALL achieve ≥30,000 QPS on point_select benchmark.

#### Scenario: Point select performance
- **WHEN** running sysbench point_select
- **THEN** QPS is at least 30,000

### Requirement: RC Gate R15 Stability 72h Test
The system SHALL pass 72-hour stability test.

#### Scenario: Long-running stability
- **WHEN** system runs continuously for 72 hours
- **THEN** no crashes or data corruption occurs

### Requirement: RC Gate R16 OO Documentation
The system SHALL have all 13 OO documentation files present.

#### Scenario: Documentation existence
- **WHEN** checking docs/releases/v3.2.0/oo/
- **THEN** all 13 required documents exist
