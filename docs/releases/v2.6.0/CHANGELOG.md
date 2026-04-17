# SQLRustGo v2.6.0 Changelog

> **Version**: alpha/v2.6.0
> **Date**: 2026-04-18
> **Status**: In Development

---

## Changelog

### [v2.6.0] - 2026-04-XX (Alpha)

#### 🚀 New Features

##### Parser
- **GROUP BY support** - Add `group_by` and `HAVING` fields to SelectStatement (#1498)
- **CREATE INDEX support** - Add CreateIndexStatement struct and parsing
- **ALTER TABLE execution** - Add ALTER TABLE execution support
- **Binary expression support** - Add binary expression support and API compatibility fixes (#1580)
- **CTE (WITH clause)** - Add CTE execution support with WITH clause
- **Subquery support** - Add EXISTS, IN, ALL, ANY, SOME subquery support
- **INSERT SELECT** - Add INSERT SELECT execution support
- **FOREIGN KEY parsing** - Add FOREIGN KEY and constraint parsing (#1379)

##### Storage
- **WAL support** - Add FileStorage::new_with_wal() for WAL-enabled initialization (#1573)
- **add_column/rename_table** - Add add_column and rename_table for FileStorage
- **ColumnDefinition API** - Fix ColumnDefinition and TableInfo API mismatch in tests

##### Executor
- **ALTER TABLE execution** - Full ALTER TABLE execution support
- **CTE execution** - CTE execution support
- **UNIQUE constraint** - Add UNIQUE constraint validation on INSERT
- **PRIMARY KEY validation** - Add PRIMARY KEY uniqueness validation on INSERT
- **Subquery evaluation** - Add subquery evaluation in expression_to_value
- **INSERT SELECT** - INSERT SELECT execution support

##### Distributed
- **gRPC communication** - Add gRPC communication layer for distributed storage
- **Consensus module** - Add consensus, failover, and replica sync modules
- **Distributed tests** - Add distributed storage integration tests
- **ShardedVectorIndex** - Distributed vector storage with hash-based partitioning
- **ShardGraph** - Distributed graph storage with label-based partitioning

##### Vector & Graph
- **Vector Index** - ShardedVectorIndex with hash-based partitioning
- **Graph Storage** - ShardGraph with label-based partitioning

#### 🐛 Bug Fixes

- **SQL Corpus API** - Resolve API compatibility issues with current architecture
- **Binary storage** - Export binary_storage and fix TPC-H benchmark example
- **Test compilation** - Fix example compilation errors and tokio feature
- **PLANNER** - Add missing as_any implementation for LimitExec

#### 📚 Documentation

- **VERSION_HISTORY** - Add complete version evolution history
- **LONG_TERM_ROADMAP** - Update with all released versions
- **SQL-92 support matrix** - Add SQL syntax support matrix
- **Distributed deployment** - Add distributed cluster deployment script

#### 🧪 Testing

- **SQL-92 test failures** - Document SQL-92 test failures
- **TPC-H SF1** - Add TPC-H SF1 tests, standard SQL queries
- **Distributed tests** - Add distributed storage to regression suite
- **Coverage improvement** - Multiple test coverage improvements (#1572, #1577)

#### 🔧 Refactoring

- **SQL-92 compliance** - Enhanced SQL-92 compliance
- **API compatibility** - Fix API compatibility issues
- **Module exports** - Fix parser module exports

---

### [v2.5.0] - 2026-04-03 (GA)

#### Core Features
- MVCC (Multi-Version Concurrency Control)
- Vector storage support
- Graph storage support
- ParallelExecutor
- CBO (Cost-Based Optimizer) framework

---

### [v2.4.0] - 2026-XX-XX (GA)

#### Core Features
- Columnar storage foundation
- Columnar scan
- Column compression

---

### [v2.2.0] - 2026-XX-XX (GA)

#### Core Features
- IVF-PQ index
- HNSW index

---

### [v2.0.0] - 2026-03-29 (GA)

#### Core Features
- Vectorized execution
- Complete OO architecture refactoring
- MySQL wire protocol server

---

## Migration from v2.5.0

See [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md) for detailed migration instructions.

Key changes in v2.6.0:
- New parser features require schema updates for GROUP BY/HAVING
- ALTER TABLE now supports execution phase
- FileStorage supports WAL mode via `new_with_wal()`
- New CTE and subquery features available

---

## Known Issues

- SQL-92 test failures in execution layer for INSERT operations
- Some advanced SQL-92 features still in progress
- Distributed features marked as experimental

---

## Contributors

Thanks to all contributors for this release!

---

*Auto-generated from Git commit history*