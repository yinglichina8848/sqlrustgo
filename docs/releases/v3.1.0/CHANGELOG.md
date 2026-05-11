# v3.1.0 Changelog

> **Version**: 3.1.0  
> **Date**: 2026-05-11  
> **Status**: In Development

---

## v3.1.0-ga (TBD)

### Added

#### INFORMATION_SCHEMA (P0)
- `SCHEMATA` — database schema information
- `TABLES` — table metadata
- `COLUMNS` — column definitions with `CHARACTER_SET_NAME`, `COLLATION_NAME`, `IS_NULLABLE`
- `STATISTICS` — index statistics
- `REFERENTIAL_CONSTRAINTS` — foreign key constraints
- `CHARACTER_SETS` — available character sets
- `COLLATIONS` — collation information

#### SQL Operations (P0)
- `SAVEPOINT` / `ROLLBACK TO SAVEPOINT`
- `SET TRANSACTION ISOLATION LEVEL`
- `LIMIT` / `OFFSET` optimization
- `TRUNCATE TABLE`
- `REPLACE INTO`
- `SHOW` statement variants (`SHOW CREATE TABLE`, `SHOW INDEX`, `SHOW ENGINES`)
- `EXPLAIN ANALYZE`

#### MERGE Statement (P0)
- `MERGE INTO ... WHEN MATCHED ... WHEN NOT MATCHED ...` (MySQL 8.0 compatible)

#### Performance Schema (P1)
- `setup_actors`
- `setup_instruments`
- `events_statements_summary_by_digest`
- `events_statements_history`
- `events_waits_summary_by_thread`

#### CBO CostModel Integration (P1)
- `SimpleCostModel` activated in planner
- Automatic index selection in `EXPLAIN`
- Join reordering based on cost

#### TLS/SSL (P1)
- MySQL protocol TLS handshake complete integration
- `--ssl-mode=REQUIRED` supported

#### Full-Text Search (P1)
- English tokenizer with stop words
- Chinese tokenizer (jieba)
- `MATCH(col) AGAINST('keyword')` syntax
- INPLACE DML incremental index update

#### Event Scheduler (P1)
- `CREATE EVENT ... ON SCHEDULE`
- `ALTER EVENT`
- `DROP EVENT`
- `SHOW EVENTS`

#### Join Algorithms (P1)
- `MERGE JOIN` — Sort-Merge equi-join
- `BNL JOIN` — Block Nested Loop non-equi-join

#### GMP Compliance Foundation (P0)

**Audit Log:**
- Crash-safe audit trail with SHA-256 hash chain
- WAL integration for atomic persistence
- Tamper detection on startup
- Evidence export with JSON signature
- BP2-1~BP2-6 tests passing

**Gap Locking:**
- Next-Key Lock implementation
- SERIALIZABLE isolation level complete
- SSI dead lock detection < 100ms
- Phantom read prevention verified

**Crash Recovery:**
- 5-scenario chaos injection all recoverable
- S1: WAL write before crash
- S2: WAL write after, uncommitted
- S3: pre-commit crash
- S4: checkpoint crash
- S5: torn page

**Storage Encryption:**
- AES-256-GCM page-level encryption
- `KeyProvider` trait (Env / File)
- Key rotation support
- WAL encryption

**Fine-Grained Privileges:**
- Column-level privilege execution
- RBAC execution layer (not just parsing)
- Row-level security policies

**Clustered Index:**
- B+Tree clustered leaf nodes
- Primary key stored directly in leaf
- Secondary indexes point to primary key
- Hidden primary key (UUID) for tables without PK

#### Architecture Refinements
- `bplus_tree/clustered_leaf.rs` — clustered leaf node
- `transaction/gap_lock.rs` — GapLock types
- `transaction/next_key_lock.rs` — Next-Key Lock algorithm
- `encryption/aes_cipher.rs` — AES-256-GCM
- `encryption/key_manager.rs` — Key management
- `gmp/audit_chain.rs` — immutable audit chain

### Changed

- **Coverage thresholds**: GA 65% (up from 22%)
- **SQL Corpus**: GA 98% (up from 95%)
- **Formal proofs**: GA 30 (up from 10)

### Fixed

- `long_run_stability_test` — all #[ignore] removed
- TPC-H SF=1 OOM — resolved via incremental data generation
- Transaction state machine stress tests — implemented

---

## v3.0.0 (2026-05-08)

### Added
- WAL + MVCC (Snapshot Isolation + SSI)
- Point SELECT 398K QPS
- CTE (WITH clause)
- Window functions 6/6
- `EXPLAIN` / `EXPLAIN ANALYZE`
- TLS/SSL (rustls)
- Slow query log
- Group Commit WAL
- Query cache (LRU + DML invalidation)
- Connection pool (Thread Pool)
- Online DDL (ADD/DROP/MODIFY/RENAME)
- MySQL dump export
- 30 formal proofs

### Known Issues
- TPC-H SF=1 OOM (fixed in v3.1.0)
- INFORMATION_SCHEMA ~30% (fixed in v3.1.0)
- SQL Operations 20% (fixed in v3.1.0)
- Gap Locking not implemented (fixed in v3.1.0)
- No storage encryption (fixed in v3.1.0)
