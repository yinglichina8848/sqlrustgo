# v3.1.0 测试计划

> **版本**: 3.1.0  
> **日期**: 2026-05-11  
> **状态**: 开发中

---

## 测试概述

### 测试策略

```
v3.1.0 测试金字塔
────────────────────────────────────────────
                    ▲
                   /E2E\         集成测试 (100+)
                  /─────\
                 / smoke\       冒烟测试 (50+)
                /───────\
               /  unit  \     单元测试 (500+)
              /───────────\
             / coverage  \    覆盖率驱动测试
            └─────────────┘
```

### 测试类别

| 类别 | v3.0.0 | v3.1.0 目标 |
|----------|---------|---------------|
| 单元测试 | ~400 | ≥500 |
| 集成测试 | ~50 | ≥100 |
| 端到端测试 | ~10 | ≥20 |
| 混沌/崩溃测试 | 5 | 10 |
| 并发测试 | 20 | 50 |
| **合计** | **~485** | **≥700** |

---

## Alpha 阶段测试 (2026-06-01)

### INFORMATION_SCHEMA 测试

| 测试 | 文件 | 覆盖率目标 |
|------|------|---------------|
| SCHEMATA 表 | `tests/is_schemata_test.rs` | 100% |
| TABLES 表 | `tests/is_tables_test.rs` | 100% |
| COLUMNS 表 | `tests/is_columns_test.rs` | 100% |
| STATISTICS 表 | `tests/is_statistics_test.rs` | 100% |

### SQL 操作测试

| 测试 | 文件 | 目标 |
|------|------|--------|
| SAVEPOINT | `tests/savepoint_test.rs` | 100% |
| SET TRANSACTION | `tests/set_transaction_test.rs` | 100% |
| TRUNCATE | `tests/truncate_test.rs` | 100% |
| REPLACE INTO | `tests/replace_test.rs` | 100% |
| SHOW 变体 | `tests/show_statement_test.rs` | 100% |

### MERGE 语句测试

| 测试 | 文件 | 目标 |
|------|------|--------|
| MERGE matched UPDATE | `tests/merge_test.rs` | 100% |
| MERGE NOT MATCHED INSERT | `tests/merge_test.rs` | 100% |
| MERGE 多表 | `tests/merge_test.rs` | 100% |

---

## Beta 阶段测试 (2026-07-01)

### Performance Schema 测试

| 测试 | 文件 | 目标 |
|------|------|--------|
| setup_actors | `tests/ps_setup_actors_test.rs` | 100% |
| setup_instruments | `tests/ps_setup_instruments_test.rs` | 100% |
| events_statements_summary | `tests/ps_events_statements_test.rs` | 100% |

### CBO 测试

| 测试 | 文件 | 目标 |
|------|------|--------|
| 索引选择 | `tests/cbo_index_selection_test.rs` | 100% |
| 连接排序 | `tests/cbo_join_order_test.rs` | 100% |
| 成本估算 | `tests/cbo_cost_estimation_test.rs` | 100% |

### TPC-H 测试

| 测试 | 文件 | 目标 |
|------|------|--------|
| SF=0.1 全部 22 个 | `tests/tpch_sf01_test.rs` | 22/22 |
| SF=1 全部 22 个 | `tests/tpch_sf1_test.rs` | 22/22 |

---

## RC 阶段测试 (2026-08-01)

### GMP 合规测试

#### 审计链测试 (BP2-1~BP2-6)

| 测试 | 场景 | 目标 |
|------|---------|--------|
| `chaos_crash_wal_before` | S1: 崩溃前 WAL 写入 | 0 数据丢失 |
| `chaos_crash_wal_after` | S2: 崩溃后 WAL 写入, 未提交 | 一致性 |
| `chaos_crash_precommit` | S3: 预提交崩溃 | 可恢复 |
| `chaos_crash_checkpoint` | S4: 检查点崩溃 | 链完整 |
| `chaos_crash_torn_page` | S5: 撕裂页 | 无损坏 |
| `audit_hash_chain_verify` | SHA-256 链验证 | 0 中断 |
| `audit_concurrent_write` | 100 并发审计写入 | 0 丢失 |

#### 间隙锁测试

| 测试 | 场景 | 目标 |
|------|---------|--------|
| `gap_lock_phantom_read` | 并发 INSERT + SELECT 范围 | 0 幻读 |
| `gap_lock_range_update` | 并发 UPDATE + SELECT | 一致性 |
| `gap_lock_deadlock` | SSI 死锁检测 | <100ms |
| `serializable_phantom_read` | SERIALIZABLE 隔离级别 | 0 幻读 |

#### 加密测试

| 测试 | 目标 |
|------|--------|
| `encryption_page_roundtrip` | 加密/解密 = 原始值 |
| `encryption_key_rotation` | 服务不中断 |
| `encryption_wrong_key` | 启动失败（非静默） |

#### 聚簇索引测试

| 测试 | 目标 |
|------|--------|
| `clustered_pk_lookup` | 1 次 I/O（非 2 次） |
| `clustered_secondary_index` | 正确的 PK 引用 |
| `clustered_hidden_pk` | UUID 自动生成 |

### 全文搜索测试

| 测试 | 目标 |
|------|--------|
| `fts_english_tokenize` | 停用词已移除 |
| `fts_chinese_tokenize` | Jieba 分词 |
| `fts_match_against` | MATCH 返回相关文档 |
| `fts_boolean_mode` | +word -word 过滤 |

### 事件调度器测试

| 测试 | 目标 |
|------|--------|
| `event_create` | CREATE EVENT 语法 |
| `event_schedule_at` | AT 时间戳执行 |
| `event_schedule_every` | EVERY 间隔执行 |
| `event_persist_recovery` | 事件在重启后存活 |

---

## 稳定性测试 (B-S1~B-S5)

| 测试 | v3.0.0 | v3.1.0 目标 |
|------|---------|---------------|
| `concurrency_stress_test` | 9/9 | 9/9 |
| `crash_recovery_test` | 8/8 | 9/9 (+1 新增) |
| `long_run_stability_test` | 10/10 #[ignore] | **10/10 通过** |
| `wal_integration_test` | 16/16 | 16/16 |
| `network_tcp_smoke_test` | 6/6 | 8/8 (+2 新增) |

---

## 覆盖率目标

| 阶段 | 日期 | 总体 | parser | executor | planner | optimizer |
|-------|------|---------|--------|----------|---------|-----------|
| Alpha | 2026-06-01 | ≥40% | 65% | 55% | 50% | 40% |
| Beta | 2026-07-01 | ≥50% | 65% | 55% | 50% | 40% |
| RC | 2026-08-01 | ≥60% | 70% | 65% | 55% | 50% |
| GA | 2026-09-01 | **≥65%** | **75%** | **70%** | **60%** | **60%** |

---

## 测试执行

### 本地

```bash
# 完整测试套件
cargo test --all-features

# 覆盖率
cargo llvm-cov --all-features --html
open target/llvm-cov/html/index.html

# 特定测试
cargo test --test chaos_crash_wal_before
cargo test --test merge_test
```

### CI/CD

```bash
# Nomad CI runner (Gitea Actions)
# 每次 PR 自动运行
bash scripts/gate/check_beta_v310.sh   # Alpha
bash scripts/gate/check_rc_v310.sh     # RC
bash scripts/gate/check_ga_v310.sh     # GA
```

---

## 测试基础设施

### 测试数据库

| 数据库 | 用途 |
|----------|---------|
| `sqlrustgo_test` | 单元/集成测试 |
| `sqlrustgo_tpch` | TPC-H 基准测试 |
| `sqlrustgo_gmp` | GMP 合规测试 |

### 测试 fixtures

```
tests/fixtures/
├── schema/
│   ├── standard.sql      # 标准测试表
│   ├── gmp.sql           # GMP 合规模式
│   └── tpch.sql          # TPC-H 表定义
├── data/
│   ├── sample_1k.csv      # 1K 行
│   ├── sample_100k.csv   # 100K 行
│   └── tpch_sf01/        # TPC-H SF=0.1 数据
└── expected/
    ├── merge_expected.json
    └── fts_expected.json
```
