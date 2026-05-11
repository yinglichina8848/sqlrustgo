# v3.1.0 Test Plan

> **Version**: 3.1.0  
> **Date**: 2026-05-11  
> **Status**: In Development

---

## Test Overview

### Testing Strategy

```
v3.1.0 Test Pyramid
────────────────────────────────────────────
                    ▲
                   /E2E\         Integration Tests (100+)
                  /─────\
                 / smoke\       Smoke Tests (50+)
                /───────\
               /  unit  \     Unit Tests (500+)
              /───────────\
             / coverage  \    Coverage-driven Tests
            └─────────────┘
```

### Test Categories

| Category | v3.0.0 | v3.1.0 Target |
|----------|---------|---------------|
| Unit tests | ~400 | ≥500 |
| Integration tests | ~50 | ≥100 |
| E2E tests | ~10 | ≥20 |
| Chaos/crash tests | 5 | 10 |
| Concurrency tests | 20 | 50 |
| **Total** | **~485** | **≥700** |

---

## Alpha Phase Tests (2026-06-01)

### INFORMATION_SCHEMA Tests

| Test | File | Coverage Target |
|------|------|---------------|
| SCHEMATA table | `tests/is_schemata_test.rs` | 100% |
| TABLES table | `tests/is_tables_test.rs` | 100% |
| COLUMNS table | `tests/is_columns_test.rs` | 100% |
| STATISTICS table | `tests/is_statistics_test.rs` | 100% |

### SQL Operations Tests

| Test | File | Target |
|------|------|--------|
| SAVEPOINT | `tests/savepoint_test.rs` | 100% |
| SET TRANSACTION | `tests/set_transaction_test.rs` | 100% |
| TRUNCATE | `tests/truncate_test.rs` | 100% |
| REPLACE INTO | `tests/replace_test.rs` | 100% |
| SHOW variants | `tests/show_statement_test.rs` | 100% |

### MERGE Statement Tests

| Test | File | Target |
|------|------|--------|
| MERGE matched UPDATE | `tests/merge_test.rs` | 100% |
| MERGE NOT MATCHED INSERT | `tests/merge_test.rs` | 100% |
| MERGE multi-table | `tests/merge_test.rs` | 100% |

---

## Beta Phase Tests (2026-07-01)

### Performance Schema Tests

| Test | File | Target |
|------|------|--------|
| setup_actors | `tests/ps_setup_actors_test.rs` | 100% |
| setup_instruments | `tests/ps_setup_instruments_test.rs` | 100% |
| events_statements_summary | `tests/ps_events_statements_test.rs` | 100% |

### CBO Tests

| Test | File | Target |
|------|------|--------|
| index selection | `tests/cbo_index_selection_test.rs` | 100% |
| join ordering | `tests/cbo_join_order_test.rs` | 100% |
| cost estimation | `tests/cbo_cost_estimation_test.rs` | 100% |

### TPC-H Tests

| Test | File | Target |
|------|------|--------|
| SF=0.1 all 22 | `tests/tpch_sf01_test.rs` | 22/22 |
| SF=1 all 22 | `tests/tpch_sf1_test.rs` | 22/22 |

---

## RC Phase Tests (2026-08-01)

### GMP Compliance Tests

#### Audit Chain Tests (BP2-1~BP2-6)

| Test | Scenario | Target |
|------|---------|--------|
| `chaos_crash_wal_before` | S1: WAL write before crash | 0 data loss |
| `chaos_crash_wal_after` | S2: WAL write after, uncommitted | Consistent |
| `chaos_crash_precommit` | S3: pre-commit crash | Recoverable |
| `chaos_crash_checkpoint` | S4: checkpoint crash | Chain intact |
| `chaos_crash_torn_page` | S5: torn page | No corruption |
| `audit_hash_chain_verify` | SHA-256 chain validation | 0 breaks |
| `audit_concurrent_write` | 100 concurrent audit writes | 0 lost |

#### Gap Locking Tests

| Test | Scenario | Target |
|------|---------|--------|
| `gap_lock_phantom_read` | Concurrent INSERT + SELECT range | 0 phantom |
| `gap_lock_range_update` | Concurrent UPDATE + SELECT | Consistent |
| `gap_lock_deadlock` | SSI deadlock detection | <100ms |
| `serializable_phantom_read` | SERIALIZABLE isolation | 0 phantom |

#### Encryption Tests

| Test | Target |
|------|--------|
| `encryption_page_roundtrip` | encrypt/decrypt = original |
| `encryption_key_rotation` | Service uninterrupted |
| `encryption_wrong_key` | Startup fails (not silent) |

#### Clustered Index Tests

| Test | Target |
|------|--------|
| `clustered_pk_lookup` | 1 I/O (not 2) |
| `clustered_secondary_index` | Correct PK reference |
| `clustered_hidden_pk` | UUID auto-generated |

### Full-Text Search Tests

| Test | Target |
|------|--------|
| `fts_english_tokenize` | Stop words removed |
| `fts_chinese_tokenize` | Jieba segmentation |
| `fts_match_against` | MATCH returns relevant docs |
| `fts_boolean_mode` | +word -word filtering |

### Event Scheduler Tests

| Test | Target |
|------|--------|
| `event_create` | CREATE EVENT syntax |
| `event_schedule_at` | AT timestamp execution |
| `event_schedule_every` | EVERY interval execution |
| `event_persist_recovery` | Events survive restart |

---

## Stability Tests (B-S1~B-S5)

| Test | v3.0.0 | v3.1.0 Target |
|------|---------|---------------|
| `concurrency_stress_test` | 9/9 | 9/9 |
| `crash_recovery_test` | 8/8 | 9/9 (+1 new) |
| `long_run_stability_test` | 10/10 #[ignore] | **10/10 PASS** |
| `wal_integration_test` | 16/16 | 16/16 |
| `network_tcp_smoke_test` | 6/6 | 8/8 (+2 new) |

---

## Coverage Targets

| Phase | Date | Overall | parser | executor | planner | optimizer |
|-------|------|---------|--------|----------|---------|-----------|
| Alpha | 2026-06-01 | ≥40% | 65% | 55% | 50% | 40% |
| Beta | 2026-07-01 | ≥50% | 65% | 55% | 50% | 40% |
| RC | 2026-08-01 | ≥60% | 70% | 65% | 55% | 50% |
| GA | 2026-09-01 | **≥65%** | **75%** | **70%** | **60%** | **60%** |

---

## Test Execution

### Local

```bash
# Full test suite
cargo test --all-features

# Coverage
cargo llvm-cov --all-features --html
open target/llvm-cov/html/index.html

# Specific test
cargo test --test chaos_crash_wal_before
cargo test --test merge_test
```

### CI/CD

```bash
# Nomad CI runner (Gitea Actions)
# Automatic on every PR
bash scripts/gate/check_beta_v310.sh   # Alpha
bash scripts/gate/check_rc_v310.sh     # RC
bash scripts/gate/check_ga_v310.sh      # GA
```

---

## Test Infrastructure

### Test Databases

| Database | Purpose |
|----------|---------|
| `sqlrustgo_test` | Unit/integration tests |
| `sqlrustgo_tpch` | TPC-H benchmarks |
| `sqlrustgo_gmp` | GMP compliance tests |

### Test Fixtures

```
tests/fixtures/
├── schema/
│   ├── standard.sql      # Standard test tables
│   ├── gmp.sql          # GMP compliance schemas
│   └── tpch.sql         # TPC-H table definitions
├── data/
│   ├── sample_1k.csv    # 1K rows
│   ├── sample_100k.csv  # 100K rows
│   └── tpch_sf01/       # TPC-H SF=0.1 data
└── expected/
    ├── merge_expected.json
    └── fts_expected.json
```
