# TPC-H Real Data Testing Plan for SQLRustGo

**Date**: 2025-05-03
**Author**: 李哥 / Hermes Agent
**Status**: Draft
**Branch**: s01s05-v5 → develop/v2.9.0

---

## 1. Background

### 1.1 Problem Statement

SQLRustGo currently lacks real TPC-H data testing. The existing `bench-cli tpch` command uses `MemoryStorage` with no actual data loaded, making Q1-Q22 tests meaningless for performance benchmarking.

### 1.2 Analysis (from deepseek review)

- `PostgresStorage` as `StorageEngine` trait implementation is NOT the correct approach
- Correct path: Use `ColumnarStorage` with real data + native `ExecutionEngine`
- External PostgreSQL can be used as a comparison baseline (via separate process), not as a storage backend

### 1.3 Requirements Summary

| Requirement | Decision |
|-------------|----------|
| Storage backend | ColumnarStorage (NOT external PostgreSQL) |
| Data persistence | Parquet format |
| Data format | `.tbl` files (standard TPC-H, `\|` separated) |
| Scale factors | SF=0.1 → SF=1 → SF=10 (progressive) |
| SQL source | Standard Q1-Q22 in `queries/` directory |
| Index strategy | Phase A: Standard TPC-H indexes (PK, FK), Phase B: Full indexes |
| Validation | Phase 1: Row count + sampling, Phase 2: checksum |

---

## 2. Architecture

```
                    ┌─────────────────┐
                    │   dbgen 生成    │
                    │  .tbl 文件      │
                    └────────┬────────┘
                             │
                             ▼
┌──────────────────────────────────────────────────────────────┐
│           tpch_binary_importer (新工具)                       │
│  - 解析 DDL (CREATE TABLE statements)                        │
│  - 创建 ColumnarStorage schema                               │
│  - 解析 .tbl 文件批量导入                                    │
│  - 类型转换（基于 DDL 类型）                                  │
│  - 创建索引（PRIMARY KEY, FOREIGN KEY, 其他索引）            │
│  - 持久化为 Parquet 文件                                     │
│  - 验证：行数校验 + 抽样校验                                 │
└──────────────────────┬─────────────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────────────┐
│              ColumnarStorage (Parquet 持久化)                 │
│  - SF=0.1 数据落盘                                           │
│  - 8 张表：region/nation/customer/orders/                    │
│           part/supplier/partsupp/lineitem                    │
│  - 标准索引：主键 + 外键                                     │
└──────────────────────┬─────────────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────────────┐
│                    tpch-bench 命令                            │
│  - 加载 Parquet 数据到 ColumnarStorage                       │
│  - 执行 queries/q1.sql ~ q22.sql                             │
│  - 收集 latency/p99/QPS 指标                                 │
│  - 结果输出为 JSON 报告                                      │
└──────────────────────────────────────────────────────────────┘
```

---

## 3. CLI Design

### 3.1 tpch-import Command

```bash
cargo run -p sqlrustgo-bench-cli -- tpch-import \
  --ddl scripts/pg_tpch_setup.sql \
  --data data/tpch-sf01 \
  --output storage/tpch-sf01 \
  [--verify-only]
```

**Options**:
- `--ddl`: Path to DDL file (e.g., `scripts/pg_tpch_setup.sql`)
- `--data`: Path to `.tbl` data files directory
- `--output`: Output directory for Parquet files
- `--verify-only`: Only verify data without importing (for validation)

### 3.2 tpch-bench Command

```bash
cargo run -p sqlrustgo-bench-cli -- tpch-bench \
  --storage storage/tpch-sf01 \
  --queries queries/ \
  --output results/sf01-results.json \
  [--iterations 3]
```

**Options**:
- `--storage`: Path to Parquet storage directory
- `--queries`: Path to queries directory (q1.sql ~ q22.sql)
- `--output`: Output JSON file for results
- `--iterations`: Number of iterations per query (default: 3)

---

## 4. Data Schema (TPC-H SF=0.1)

### 4.1 Table Row Counts

| Table | SF=0.1 Rows | SF=1 Rows | SF=10 Rows |
|-------|-------------|-----------|------------|
| REGION | 5 | 5 | 5 |
| NATION | 25 | 25 | 25 |
| CUSTOMER | 15,000 | 150,000 | 1,500,000 |
| ORDERS | 150,000 | 1,500,000 | 15,000,000 |
| LINEITEM | 600,000 | 6,000,000 | 60,000,000 |
| PART | 20,000 | 200,000 | 2,000,000 |
| SUPPLIER | 1,000 | 10,000 | 100,000 |
| PARTSUPP | 80,000 | 800,000 | 8,000,000 |

### 4.2 TPC-H Indexes (Phase A)

```sql
-- Primary Keys
PRIMARY KEY (r_regionkey)           -- REGION
PRIMARY KEY (n_nationkey)           -- NATION
PRIMARY KEY (c_custkey)             -- CUSTOMER
PRIMARY KEY (o_orderkey)            -- ORDERS
PRIMARY KEY (l_orderkey, l_linenumber)  -- LINEITEM
PRIMARY KEY (p_partkey)             -- PART
PRIMARY KEY (s_suppkey)             -- SUPPLIER
PRIMARY KEY (ps_partkey, ps_suppkey) -- PARTSUPP

-- Foreign Key Indexes
INDEX idx_customer_nationkey (c_nationkey)
INDEX idx_orders_custkey (o_custkey)
INDEX idx_orders_orderdate (o_orderdate)
INDEX idx_lineitem_orderkey (l_orderkey)
INDEX idx_lineitem_partkey (l_partkey)
INDEX idx_lineitem_suppkey (l_suppkey)
INDEX idx_lineitem_shipdate (l_shipdate)
INDEX idx_partsupp_partkey (ps_partkey)
INDEX idx_partsupp_suppkey (ps_suppkey)
INDEX idx_supplier_nationkey (s_nationkey)
INDEX idx_nation_regionkey (n_regionkey)
```

---

## 5. Implementation Plan

### Phase 1: Data Import Tool (tpch-import)

#### 1.1 DDL Parser
- Parse `CREATE TABLE` statements from `scripts/pg_tpch_setup.sql`
- Extract: table name, column names, column types, constraints (PRIMARY KEY, FOREIGN KEY)
- Support standard SQL types: INT, INTEGER, BIGINT, VARCHAR, CHAR, DATE, DECIMAL, REAL, DOUBLE

#### 1.2 .tbl File Parser
- Parse `*.tbl` files with `|` delimiter
- Type conversion based on DDL schema (NOT auto-inference)
- Support for quoted strings with `|` inside

#### 1.3 ColumnarStorage Integration
- Create tables via `ColumnarStorage::create_table()`
- Batch insert using `TableStore::insert_row()`
- Build indexes after data load

#### 1.4 Parquet Persistence
- Use existing `crates/storage/src/columnar/parquet.rs`
- Persist each table as separate Parquet file
- Store metadata (schema, row counts, indexes) in `storage.json`

#### 1.5 Verification
- **Row count verification**: Compare `.tbl` line count vs stored row count
- **Sampling verification**: Random sample 100 rows, print first 10 for manual inspection

### Phase 2: Benchmark Tool (tpch-bench)

#### 2.1 Data Loading
- Load Parquet files from storage directory
- Initialize `ColumnarStorage` from persisted data

#### 2.2 Query Execution
- Read SQL from `queries/q1.sql` ~ `queries/q22.sql`
- Execute via `ExecutionEngine::execute()`
- Collect: latency per iteration, avg, min, max, p50, p90, p99

#### 2.3 Results Output
- JSON format with full metrics
- Human-readable summary table

### Phase 3: Validation (Post-Implementation)

#### 3.1 SF=0.1 Validation (Immediate)
- Verify row counts match
- Verify query results match expected patterns

#### 3.2 SF=1 Performance Testing (After SF=0.1 passes)
- Full SF=1 data import
- Run complete Q1-Q22 benchmark
- Collect performance metrics

#### 3.3 SF=10 RC Testing (Final)
- Large-scale validation
- Performance regression detection

---

## 6. File Structure

```
crates/bench-cli/
├── src/
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── tpch_import.rs    # NEW: tpch-import command
│   │   └── tpch_bench.rs     # EXISTING: will be refactored
│   ├── ddl_parser.rs         # NEW: DDL parsing logic
│   ├── tbl_parser.rs         # NEW: .tbl file parsing
│   └── main.rs
├── Cargo.toml
└── benches/
    └── tpch_import_bench.rs

crates/storage/src/columnar/
├── storage.rs                 # EXISTING: ColumnarStorage
├── parquet.rs                # EXISTING: Parquet support
├── index.rs                  # NEW: Index implementation
└── mod.rs

scripts/
├── pg_tpch_setup.sql         # EXISTING: DDL source
├── tpch_comparison.py        # EXISTING: comparison script
└── dbgen/                    # NEW: TPC-H data generation
    └── (dbgen tool or pre-generated .tbl files)

queries/
├── q1.sql ~ q22.sql          # EXISTING: standard TPC-H queries
└── (use existing files)

docs/plans/
├── 2025-05-03-tpch-real-data-testing-design.md  # THIS FILE
└── (additional implementation docs)
```

---

## 7. Dependencies

### 7.1 New Crates Required

```toml
# crates/bench-cli/Cargo.toml (additions)
[dependencies]
sqlparser = "0.50"  # For DDL parsing (if not already available)
```

### 7.2 Existing Infrastructure

- `crates/storage/src/columnar/storage.rs` - ColumnarStorage API
- `crates/storage/src/columnar/parquet.rs` - Parquet persistence
- `scripts/pg_tpch_setup.sql` - DDL source
- `queries/q1.sql ~ q22.sql` - Query source

---

## 8. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| DDL parser incomplete | Start with simple parser, extend incrementally |
| .tbl file encoding issues | Assume UTF-8, handle BOM if present |
| Memory pressure with SF=1 | Process in batches, not all in memory |
| Parquet serialization slow | Use existing parquet.rs, benchmark bottlenecks |
| Query execution failures | Capture errors, continue with remaining queries |

---

## 9. Success Criteria

### 9.1 Phase 1 Complete (Data Import)
- [ ] `tpch-import` successfully imports all 8 tables for SF=0.1
- [ ] Row counts match source .tbl files (verification passes)
- [ ] Parquet files created in output directory
- [ ] Indexes created and persisted

### 9.2 Phase 2 Complete (Benchmark)
- [ ] `tpch-bench` loads persisted data successfully
- [ ] Q1-Q22 all execute without crashes
- [ ] Results JSON produced with all metrics
- [ ] Human-readable summary displayed

### 9.3 Phase 3 Complete (Validation)
- [ ] SF=0.1: Query results validated
- [ ] SF=1: Full benchmark completes
- [ ] Performance metrics collected and documented

---

## 10. Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| Week 1 | Phase 1.1 | DDL parser, basic table creation |
| Week 1 | Phase 1.2 | .tbl parser, data import |
| Week 2 | Phase 1.3 | Parquet persistence, indexing |
| Week 2 | Phase 1.4 | Verification (row count, sampling) |
| Week 3 | Phase 2.1 | tpch-bench command |
| Week 3 | Phase 2.2 | Query execution, metrics collection |
| Week 3 | Phase 2.3 | Results output, summary display |
| Week 4 | Phase 3 | SF=0.1 validation, SF=1 testing |

---

## 11. References

- TPC-H Specification: http://www.tpc.org/tpch/
- SQLRustGo ColumnarStorage: `crates/storage/src/columnar/storage.rs`
- Existing TPC-H SQL: `queries/q*.sql`
- DDL Source: `scripts/pg_tpch_setup.sql`
- Previous fast importer: `crates/bench/examples/tpch_fast_importer.rs`

---

**Document Status**: Draft - Pending Implementation
**Next Action**: Begin Phase 1.1 - DDL Parser Implementation
