# v3.2.0 Changelog

> **Version**: 3.2.0
> **Date**: 2026-05-16
> **Status**: RC Transition
> **Branch**: develop/v3.2.0
> **HEAD**: `17fda5f6`

---

## v3.2.0-beta (2026-05-16)

### Added

#### GMP Framework (P0)

**GMP-1: Digital Signature Audit Chain**
- `AuditChain` — SHA-256 hash chain for audit records
- `AuditChainVerifier` — tamper detection on startup
- `AuditRecord` with digital signature (k256 ECDSA)
- Evidence export with JSON signature
- PR: #1012

**GMP-2: Electronic Signature**
- `ElectronicSignatureProvider` — 21 CFR Part 11 compliance
- `SignatureRecord` — electronic signature with reason
- SQL parsing for `SIGNATURE` column type
- `ApprovalPolicyEvaluator` — multi-signature approval
- PR: #1004, #1015, #1017, #1018

**GMP-3: Immutable Record / Evidence Chain**
- `EvidenceChain` — immutable evidence chain with integrity hash
- `EvidenceNode` — document, SQL, vector, graph nodes
- `ImmutableRecord` — EBR (Evidence-Based Record) system
- `VerificationReport` — chain verification results
- PR: #1029

**GMP-4: Correction Chain**
- `CorrectionChain` — chain for record corrections
- `CorrectionRecord` — correction metadata and reason
- Audit trail for modified records
- PR: #1027

**GMP-5: Provenance Tracking**
- `ProvenanceRecord` — data origin and transformation tracking
- `LineageGraph` — full data lineage visualization
- `LineageNode` — provenance nodes with metadata
- PR: #1024

**GMP-6: Trusted Timestamp**
- `TrustedTimestampProvider` — RFC 3161 trusted timestamping
- `TimestampVerifier` — timestamp validation
- Integration with audit chain
- PR: #1017

**GMP-7: Audit Chain Verification Tool**
- `AuditChainVerifier` — incremental and full verification
- `evidence_incremental_verify()` — partial chain verification
- PR: #1020

**GMP-8: HSM/KMS Integration**
- `HSMProvider` trait — hardware security module abstraction
- `SoftwareTPM` — software-based TPM implementation
- PKCS#11 interface support
- PR: #1025

**GMP-9: Workflow Engine**
- `WorkflowEngine` — GMP workflow orchestration
- `WorkflowDefinition` — workflow DSL
- `WorkflowInstance` — workflow execution state
- `ApprovalPolicyEvaluator` — multi-signature approval
- PR: #1046

#### Performance (P1)

**PERF-1: MySQL Flush Optimization**
- MySQL protocol flush optimization
- Reduced latency for client responses
- PR: #1059, #1060

**PERF-2: TPC-H SF=10 (Spill Framework)**
- Spill framework integration into LocalExecutor
- Disk-based sorting for large result sets
- 22/22 TPC-H queries pass without OOM
- PR: #1064

**PERF-3: Concurrent Connection Pool 200+**
- Thread pool implementation
- `MAX_CONNECTIONS` configuration
- Connection multiplexing
- PR: #1013

**PERF-4: Deadlock Detection**
- SSI deadlock detection optimization
- Reduced detection latency to <50ms
- PR: #1043

**PERF-5: Memory Optimization**
- Memory allocation optimization
- Buffer pool improvements
- FxHashMap replacement
- PR: #1045

#### SQL Features

**SQL-1: RECURSIVE CTE**
- Complete RECURSIVE CTE implementation
- WITH RECURSIVE support
- PR: #1065

**SQL-2: Performance Schema**
- SQL-2 Performance Schema implementation
- Setup and Events tables
- PR: #1071

#### Multi-Table DML (M6)

**Multi-Table UPDATE Execution**
- `execute_multi_table_update()` — cross-table UPDATE
- `eval_predicate_with_multi_table()` — multi-table predicate evaluation
- `HashJoin` for table references
- `MERGE` statement support
- PR: #1021

#### Cold Storage

**StorageTierManager**
- Hot/cold storage tiering
- AWS S3 Signature V4 implementation
- Remote backup storage support
- PR: #1091, #1093

#### Parser Improvements

**Aggregate Expressions**
- `SUM(col1 * col2)` — aggregate with expressions
- `AVG(column / divisor)` — arithmetic in aggregates
- Complex aggregate argument support
- PR: #1048

#### DCL (Data Control Language)

**RowLevelSecurity + Role Nesting**
- DCL permission chain implementation
- Role-based access control with nesting
- PR: #1090

---

### Changed

#### Electronic Signature
- Split into separate module files
- `gmp_electronic_signature_test.rs` — test refactoring

#### Coverage Improvements
- GMP module test coverage expanded to 111 tests

#### Gate Scripts
- Parallel coverage execution to avoid timeout
- Added 300s timeout to R9/R-S1 stability checks
- Version detection updated for v3.2.0

---

### Fixed

#### UUID Generation
- `uuid_simple()` collision fix using atomic counter
- Thread-safe unique ID generation

#### Evidence Chain
- `test_evidence_chain_tamper_detection` — proper tamper detection
- `evidence_incremental_verify()` — function reference fix

#### AWS S3 SigV4 Signing
- Proper AWS Signature Version 4 implementation
- Fixed path vs key handling in sign_request

#### Library Imports
- `correction_chain` duplicate import removal
- Proper module re-exports

---

## v3.1.0-ga (2026-05-11)

See [v3.1.0 CHANGELOG](../v3.1.0/CHANGELOG.md) for full details.

---

## v3.0.0-ga (Previous Release)

Previous stable release.

---

**Maintenance**: hermes-z6g4
**Generated**: 2026-05-16
