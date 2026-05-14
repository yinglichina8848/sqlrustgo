# SQLRustGo 测试体系改进建议

> **版本**: 1.0
> **日期**: 2026-05-14
> **基于**: /tmp/db_qa_research.md 数据库开发与测试体系研究报告

---

## 1. 概述

本文档基于 PostgreSQL、DuckDB、SQLite 的测试实践，为 SQLRustGo 提供测试体系改进路线图。

### 1.1 当前状态评估

根据 db_qa_research.md 研究：

| 维度 | PostgreSQL | DuckDB | SQLite | SQLRustGo (目标) |
|------|------------|--------|--------|------------------|
| 测试框架 | TAP, Isolation | sqllogictest | TH3, TCL, SLT, dbsqlfuzz | sqllogictest + |
| CI/CD | GitHub Actions | GitHub Actions | 手动+自动化 | GitHub Actions |
| 代码覆盖率 | 高 | 高 | 100% MC/DC | >80% |
| 模糊测试 | 有限 | 有限 | 极为完善 | 待建立 |
| 发布检查 | 标准化 | 标准化 | 详细清单 | 标准化 |

### 1.2 改进目标

```
当前状态          目标状态 (v3.2.0)
┌─────────────┐   ┌─────────────────────┐
│ 单元测试     │   │ 单元测试 + 属性测试  │
│ 集成测试     │   │ 集成测试 + Fuzzing  │
│ 基本覆盖率   │──▶│ 覆盖率 >80%         │
│ 手动性能测试  │   │ 自动化 TPC-H/C     │
│ 无形式化验证  │   │ TLA+ 核心协议验证  │
└─────────────┘   └─────────────────────┘
```

---

## 2. 测试金字塔改进

### 2.1 当前测试分层

```
        ┌──────────────┐
        │   系统测试    │  ← 手动/半自动
        ├──────────────┤
        │   集成测试    │  ← 部分自动化
        ├──────────────┤
        │   单元测试    │  ← 良好覆盖
        └──────────────┘
```

### 2.2 目标测试分层

```
              ┌─────────────────┐
              │   基准测试       │  TPC-C/H, pgbench
              ├─────────────────┤
              │   系统测试       │  sqllogictest, E2E
              ├─────────────────┤
              │   集成测试       │  模块间协作
              ├─────────────────┤
              │   属性测试       │  proptest, quickcheck
              ├─────────────────┤
              │   单元测试       │  函数/结构体级别
              ├─────────────────┤
              │   模糊测试       │  cargo-fuzz
              ├─────────────────┤
              │   形式化验证     │  TLA+, Model Checking
              └─────────────────┘
```

---

## 3. 改进路线图

### 3.1 第一阶段：基础增强 (v3.2.0)

**目标**: 提升基础测试覆盖率和自动化

#### 3.1.1 sqllogictest 全面集成

**现状**: 仅有基础测试
**目标**: 完整的 sqllogictest 框架

**实施步骤**:

1. 完善 sqllogictest 适配器:
```rust
// src/testing/sqllogictest_adapter.rs
pub struct SQLRustGoRunner {
    db: SQLRustGo,
    files: Vec<PathBuf>,
}

impl SQLRustGoRunner {
    pub fn new() -> Self {
        Self {
            db: SQLRustGo::new(),
            files: Vec::new(),
        }
    }

    pub fn add_test_file(&mut self, path: impl AsRef<Path>) {
        self.files.push(path.as_ref().to_path_buf());
    }

    pub fn run_all(&mut self) -> Result<(), TestError> {
        for file in &self.files {
            let runner = Runner::new(&self.db);
            runner.run_file(file)?;
        }
        Ok(())
    }
}
```

2. 创建测试文件结构:
```
tests/
├── sql/
│   ├── syntax/           # 语法测试
│   │   ├── select.test
│   │   ├── where.test
│   │   ├── join.test
│   │   └── subquery.test
│   ├── dml/              # DML 测试
│   │   ├── insert.test
│   │   ├── update.test
│   │   └── delete.test
│   ├── ddl/              # DDL 测试
│   │   ├── create_table.test
│   │   ├── index.test
│   │   └── alter.test
│   ├── transaction/      # 事务测试
│   │   ├── basic_txn.test
│   │   ├── mvcc_snapshot.test
│   │   └── wal_recovery.test
│   └── compatibility/    # PostgreSQL 兼容性
│       └── pg_*.test
```

3. 参考 DuckDB 的测试组织方式:
```sql
# tests/sql/transaction/mvcc_snapshot.test

# name: mvcc_snapshot.test
# description: MVCC snapshot isolation test
# group: [transaction]

statement ok
CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance INTEGER);

statement ok
INSERT INTO accounts VALUES (1, 100), (2, 200);

# Session 1: Start transaction with snapshot
query I
BEGIN;
----
ok

query I
SELECT * FROM accounts WHERE id = 1;
----
1
100

# Session 2: Concurrent update (should not be visible to Session 1)
statement ok
BEGIN;

statement ok
UPDATE accounts SET balance = 150 WHERE id = 1;

query I
SELECT * FROM accounts WHERE id = 1;
----
1
150

statement ok
COMMIT;

# Session 1: Should still see old value (snapshot isolation)
query I
SELECT * FROM accounts WHERE id = 1;
----
1
100

statement ok
COMMIT;
```

**预期成果**:
- 新增 200+ 测试用例
- 覆盖率提升 15%
- SQL 兼容性与 PostgreSQL 对齐

#### 3.1.2 Clippy 严格化

**现状**: 基础 Clippy 配置
**目标**: 生产级 lint 规则

**实施步骤**:

1. 创建 `clippy.toml`:
```toml
# 项目级 Clippy 配置
msrv = "1.80"
cognitive-complexity-threshold = 25
too-many-arguments-threshold = 6
type-complexity-threshold = 500
single-char-binding-names-threshold = 3

# 禁止特定模式
disallowed-names = ["foo", "bar", "baz", "quux"]

# 允许的 lint
enable.await-holding-reentrant-lock
enable/ deriva PartialEq - allow
```

2. 在 `Cargo.toml` 中配置:
```toml
[lints.rust]
unsafe_code = "deny"
unused = "deny"
```

3. CI 中添加 lint 检查:
```yaml
- name: Strict Clippy
  run: |
    cargo clippy --all-features -- -D warnings \
      -A clippy::doc_overly_long_string \
      -A clippy::field_reassign_with_default
```

**预期成果**:
- 代码质量提升
- 减少常见 Rust 代码异味
- 与 PostgreSQL/DuckDB 代码规范对齐

---

### 3.2 第二阶段：高级测试 (v3.3.0)

**目标**: 模糊测试、属性测试、性能测试

#### 3.2.1 Fuzzing 集成

**现状**: 无
**目标**: 集成 cargo-fuzz

**实施步骤**:

1. 初始化 fuzz 项目:
```bash
cargo fuzz init
```

2. 创建 SQL 解析器 fuzz target:
```rust
// fuzz_targets/fuzz_sql_parser.rs
#![no_main]
use libfuzzer_sys::fuzz;

fuzz!(|data: &[u8]| {
    if let Ok(sql) = std::str::from_utf8(data) {
        // 只测试解析器，忽略执行错误
        let parser = SQLParser::new(sql);
        let _ = parser.parse();
    }
});
```

3. 创建 SQL 执行器 fuzz target:
```rust
// fuzz_targets/fuzz_sql_executor.rs
#![no_main]
use libfuzzer_sys::fuzz;

fuzz!(|data: &[u8]| {
    if data.len() < 4 { return; }
    
    // 前 2 字节: 标志位
    // 后面的: SQL
    let (flags, sql) = data.split_at(2);
    let sql = match std::str::from_utf8(sql) {
        Ok(s) => s,
        Err(_) => return,
    };
    
    let db = unsafe { SQLRustGo::new_with_flags(u16::from_ne_bytes(flags.try_into().unwrap())) };
    
    // 执行并检测 panic
    let result = std::panic::catch_unwind(|| {
        db.execute(sql)
    });
    
    match result {
        Ok(_) => { /* 正常 */ }
        Err(e) => {
            // 发现 panic
            eprintln!("PANIC: {:?}", e);
        }
    }
});
```

4. 添加 dbsqlfuzz 风格的数据库变异:
```rust
// fuzz_targets/fuzz_db_mutation.rs
#![no_main]
use libfuzzer_sys::fuzz;

fuzz!(|data: &[u8]| {
    if data.len() < 10 { return; }
    
    // 同时变异数据库状态和 SQL
    let (db_data, sql) = data.split_at(data.len() - 100);
    let sql = match std::str::from_utf8(sql) {
        Ok(s) => s,
        Err(_) => return,
    };
    
    let db = unsafe { SQLRustGo::from_bytes(db_data) };
    let _ = db.execute(sql);
});
```

5. CI 配置:
```yaml
# .github/workflows/fuzz.yml
name: Fuzz

on:
  schedule:
    - cron: '0 2 * * *'  # 每日凌晨
  push:
    branches: [main]

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: rustfmt
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      - name: Build fuzz_sql_parser
        run: cargo fuzz build fuzz_sql_parser
      - name: Run fuzz_sql_parser (1 hour)
        run: timeout 3600 cargo fuzz run fuzz_sql_parser || true
      - name: Run fuzz_sql_executor (1 hour)
        run: timeout 3600 cargo fuzz run fuzz_sql_executor || true
```

**预期成果**:
- 发现解析器边界条件 bug
- 发现执行器 panic
- 对标 SQLite dbsqlfuzz

#### 3.2.2 Property-Based Testing

**现状**: 无
**目标**: 核心算法的属性测试

**实施步骤**:

1. 添加依赖:
```toml
[dev-dependencies]
proptest = "1.1"
```

2. SQL 解析器属性测试:
```rust
// tests/property/sql_parser.rs

proptest! {
    #[test]
    fn test_parser_determinism(sql in "SELECT .*") {
        let ast1 = parse_sql(&sql).unwrap();
        let ast2 = parse_sql(&sql).unwrap();
        prop_assert_eq!(ast1, ast2);
    }

    #[test]
    fn test_parser_roundtrip(sql in valid_sql_generator()) {
        let ast = parse_sql(&sql).unwrap();
        let regenerated = ast.to_sql().unwrap();
        let ast2 = parse_sql(&regenerated).unwrap();
        prop_assert_eq!(ast, ast2);
    }

    #[test]
    fn test_expression_evaluation(expr in complex_expression()) {
        let expected = eval_direct(&expr);
        let optimized = optimize(&expr);
        let result = eval_optimized(&optimized);
        prop_assert_eq!(expected, result);
    }
}

fn valid_sql_generator() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("1".to_string()),
        Just("'hello'".to_string()),
        any::<i32>().prop_map(|n| n.to_string()),
    ];
    
    leaf.prop_recursive(
        8,   // max depth
        256, // max size
        10,  // items per collection
        |inner| prop_oneof![
            format!("({})", inner.clone()),
            format!("{} + {}", inner.clone(), inner.clone()),
            format!("{} - {}", inner.clone(), inner.clone()),
            format!("{} * {}", inner.clone(), inner.clone()),
        ]
    )
}

fn complex_expression() -> impl Strategy<Value = Expr> {
    prop_oneof![
        any::<i32>().prop_map(|n| Expr::Literal(n)),
        any::<i32>().prop_map(|n| Expr::Column(n as i64)),
        (complex_expression(), complex_expression()).prop_map(|(l, r)| Expr::BinaryOp {
            left: Box::new(l),
            op: BinaryOp::Add,
            right: Box::new(r),
        }),
    ]
}
```

3. MVCC 属性测试:
```rust
// tests/property/mvcc.rs

proptest! {
    #[test]
    fn test_mvcc_snapshot_consistency(
        initial in arb_db_state(10),
        txns in vec(arb_transaction(), 1..20)
    ) {
        let db = SQLRustGo::new_with_state(initial);
        
        for txn in txns {
            let snapshot = db.snapshot();
            let results1 = db.execute_with_snapshot(&txn.query, snapshot);
            
            // 同一快照执行两次应得到相同结果
            let results2 = db.execute_with_snapshot(&txn.query, snapshot);
            prop_assert_eq!(results1, results2);
        }
    }

    #[test]
    fn test_concurrent_update_last_commit_wins(
        initial in arb_db_state(5),
        updates in vec(arb_update(1..100), 2..10)
    ) {
        let db = SQLRustGo::new_with_state(initial);
        
        // 并发执行更新
        let handles: Vec<_> = updates.iter().map(|u| {
            std::thread::spawn({
                let db = db.clone();
                let u = u.clone();
                move || db.update(u)
            })
        }).collect();
        
        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        // 只有一个提交应该成功
        let commit_count = results.iter().filter(|r| r.is_ok()).count();
        prop_assert!(commit_count <= 1, "Only one transaction should commit");
    }
}
```

**预期成果**:
- 解析器稳定性提升
- MVCC 正确性验证
- 边界条件覆盖率提升

#### 3.2.3 性能基准测试自动化

**现状**: 手动运行
**目标**: CI 自动化 + 回归检测

**实施步骤**:

1. 创建基准测试脚本 `scripts/benchmark/tpch_runner.sh`:
```bash
#!/bin/bash
set -e

SCALE=${1:-1}
RESULTS_DIR="perf_results/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "Running TPC-H benchmark (SF=$SCALE)..."

# 生成数据
./scripts/benchmark/generate_tpch_data.sh $SCALE

# 运行查询
for q in $(seq 1 22); do
    echo "Query $q..."
    START=$(date +%s%3N)
    cargo run --release --bin tpch_query -- --query $q > /dev/null 2>&1
    END=$(date +%s%3N)
    ELAPSED=$((END - START))
    echo "Q$q: ${ELAPSED}ms" >> "$RESULTS_DIR/results.txt"
done

# 与基线比较
if [ -f "perf_baselines/latest/tpch.json" ]; then
    echo "Comparing with baseline..."
    cargo run --release --bin tpch_compare \
        --current "$RESULTS_DIR/results.txt" \
        --baseline "perf_baselines/latest/tpch.json"
fi

echo "Results saved to $RESULTS_DIR"
```

2. 创建 TPC-C runner:
```bash
#!/bin/bash
set -e

WAREHOUSES=${1:-10}
DURATION=${2:-60}

echo "Running TPC-C benchmark ($WAREHOUSES warehouses, ${DURATION}s)..."

cargo build --release --bin tpcc_benchmark

./target/release/tpcc_benchmark \
    --warehouses $WAREHOUSES \
    --duration $DURATION \
    --output "perf_results/tpcc_$(date +%Y%m%d_%H%M%S).json"
```

3. CI 配置:
```yaml
# .github/workflows/benchmark.yml
name: Performance Benchmark

on:
  schedule:
    - cron: '0 3 * * *'  # 每日凌晨
  workflow_dispatch:
    inputs:
      tpch_scale:
        description: 'TPC-H Scale Factor'
        default: '1'
      tpcc_warehouses:
        description: 'TPC-C Warehouses'
        default: '10'

jobs:
  tpch:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run TPC-H
        run: scripts/benchmark/tpch_runner.sh ${{ github.event.inputs.tpch_scale || '1' }}

  tpcc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run TPC-C
        run: scripts/benchmark/tpcc_runner.sh ${{ github.event.inputs.tpcc_warehouses || '10' }}

  regression-check:
    needs: [tpch, tpcc]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check for regression
        run: scripts/benchmark/check_regression.sh
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**预期成果**:
- 每日性能基准测试
- 性能回归自动检测
- 趋势分析

---

### 3.3 第三阶段：形式化验证 (v3.4.0)

**目标**: 核心协议的形式化验证

#### 3.3.1 TLA+ 规格编写

**现状**: 无形式化验证
**目标**: 关键组件的 TLA+ 验证

**实施步骤**:

1. MVCC 快照隔离 TLA+ 规格:
```tla
----------------------------- MODULE MVCCSnapshot ----------------------------
EXTENDS Integers, FiniteSets, Sequences

CONSTANTS
    MaxTxn,
    MaxObject,
    NumWorkers

VARIABLES
    transactions,
    objects,
    txCounter

TypeOK ==
    /\ transactions \in [1..MaxTxn] -> [
        status: {"active", "committed", "aborted"},
        snapshot: 0..MaxTxn,
        reads: SUBSET (1..MaxObject),
        writes: SUBSET (1..MaxObject),
        values: [1..MaxObject -> Int]
    ]
    /\ objects \in [1..MaxObject] -> [
        value: Int,
        version: 0..MaxTxn
    ]
    /\ txCounter \in 0..MaxTxn

Init ==
    /\ transactions = [x \in 1..MaxTxn |-> NullTransaction]
    /\ objects = [x \in 1..MaxObject |-> [value |-> 0, version |-> 0]]
    /\ txCounter = 0

NullTransaction == [status |-> "inactive", snapshot |-> 0, reads |-> {}, writes |-> {}, values |-> {}]

BeginTxn(t) ==
    /\ transactions[t].status = "inactive"
    /\ txCounter < MaxTxn
    /\ transactions' = [transactions EXCEPT ![t] = [
        status |-> "active",
        snapshot |-> txCounter + 1,
        reads |-> {},
        writes |-> {},
        values |-> {}
    ]]
    /\ txCounter' = txCounter + 1

Read(t, oid) ==
    /\ transactions[t].status = "active"
    /\ oid \notin transactions[t].reads
    /\ \E obj \in DOMAIN objects :
        /\ obj.version <= transactions[t].snapshot
        /\ transactions' = [transactions EXCEPT ![t].reads = @ \cup {oid}]
    /\ UNCHANGED <<objects, txCounter>>

SnapshotIsolationCheck(t) ==
    \forall oid \in transactions[t].writes :
        \forall other \in DOMAIN transactions :
            other # t
            /\ transactions[other].status = "committed"
            /\ transactions[other].snapshot \in (transactions[t].snapshot, txCounter)
            => objects[oid].version \notin {transactions[t].snapshot}

Commit(t) ==
    /\ transactions[t].status = "active"
    /\ SnapshotIsolationCheck(t)
    /\ objects' = [oid \in DOMAIN objects |->
        IF oid \in transactions[t].writes
        THEN [value |-> transactions[t].values[oid], version |-> t]
        ELSE objects[oid]
    ]
    /\ transactions' = [transactions EXCEPT ![t].status = "committed"]

Abort(t) ==
    /\ transactions[t].status = "active"
    /\ transactions' = [transactions EXCEPT ![t].status = "aborted"]
    /\ UNCHANGED <<objects, txCounter>>

Next ==
    \E t \in 1..MaxTxn :
        \/ BeginTxn(t)
        \/ \E oid \in 1..MaxObject : Read(t, oid)
        \/ Commit(t)
        \/ Abort(t)

Spec == Init /\ [][Next]_<<transactions, objects, txCounter>>

THEOREM Spec => []TypeOK
============================================================================
```

2. WAL Recovery TLA+ 规格:
```tla
----------------------------- MODULE WALRecovery ----------------------------
EXTENDS Integers, Sequences, FiniteSets

VARIABLES
    log,
    disk,
    state,
    maxLSN

Init ==
    /\ log = <<>>
    /\ disk = [addr \in 1..10 |-> NullPage]
    /\ state = [data \in 1..10 |-> 0]
    /\ maxLSN = 0

NullPage == [addr |-> 0, data |-> 0, dirty |-> FALSE]

AppendLog(entry) ==
    /\ log' = Append(log, [lsn |-> maxLSN + 1] \o entry)
    /\ maxLSN' = maxLSN + 1

FlushPage(addr) ==
    /\ disk[addr].dirty = TRUE
    /\ disk' = [disk EXCEPT ![addr] = [disk[addr] EXCEPT !.dirty = FALSE]]

Recovery ==
    \forall entry \in log :
        entry.type = "COMMIT"
        => \forall mod \in entry.mods :
            disk' = [disk EXCEPT ![mod.addr] = [data |-> mod.value, dirty |-> FALSE]]
    /\ state' = [s \in 1..10 |-> disk[s].data]

Next ==
    \/ \E entry \in {AppendLog([type |-> "UPDATE", addr |-> 1, value |-> 1]),
                    AppendLog([type |-> "COMMIT", lsn |-> maxLSN + 1])}
    \/ FlushPage(1)
    \/ Recovery

Spec == Init /\ [][Next]_<<log, disk, state, maxLSN>>
============================================================================
```

3. 模型检验 CI:
```yaml
# .github/workflows/tla.yml
name: TLA+ Model Checking

on:
  push:
    paths:
      - 'specs/**/*.tla'
  schedule:
    - cron: '0 4 * * *'

jobs:
  model-check:
    runs-on: ubuntu-latest
    container: python:3.11-slim
    steps:
      - uses: actions/checkout@v4
      - name: Install TLA+ Toolbox
        run: |
          wget https://github.com/tlaplus/tlaplus/releases/download/v1.7.0/tla2tools.jar
          java -version
      - name: Check MVCC spec
        run: |
          java -cp tla2tools.jar tlc2.TLC specs/MVCCSnapshot.tla \
            -workers auto \
            -depth 100
      - name: Check WAL spec
        run: |
          java -cp tla2tools.jar tlc2.TLC specs/WALRecovery.tla \
            -workers auto \
            -depth 100
```

**预期成果**:
- MVCC 快照隔离正确性证明
- WAL 恢复协议验证
- 潜在 bug 在设计阶段发现

#### 3.3.2 Model Checking (SPIN)

**实施步骤**:

1. 锁协议验证:
```promela
// specs/lock_protocol.pml
mtype = { LOCK, UNLOCK, READ, WRITE };

chan lock_chan = [0] of { mtype, int };
int lock_holder = -1;
bool deadlock_detected = false;

proctype acquire_lock(int txn_id) {
    atomic {
        if
        :: lock_holder == -1 ->
            lock_holder = txn_id;
            printf("TXN %d acquired lock\n", txn_id)
        :: else ->
            printf("TXN %d blocked, lock held by %d\n", txn_id, lock_holder);
            deadlock_detected = true
        fi
    }
}

proctype release_lock(int txn_id) {
    atomic {
        if
        :: lock_holder == txn_id ->
            lock_holder = -1;
            printf("TXN %d released lock\n", txn_id)
        :: else ->
            printf("TXN %d cannot release, not lock holder\n", txn_id)
        fi
    }
}

init {
    atomic {
        run acquire_lock(1);
        run acquire_lock(2);
        run release_lock(1);
        run release_lock(2)
    }
}

ltl no_deadlock { <> (lock_holder == -1) }
```

2. 验证命令:
```bash
spin -a specs/lock_protocol.pml
gcc -O2 -o pan pan.c
./pan -m10000 -a
```

**预期成果**:
- 死锁检测
- 锁协议正确性验证

---

## 4. 测试基础设施

### 4.1 测试数据管理

**问题**: 测试数据分散，难以维护

**解决方案**:
```
test_data/
├── tpch/
│   ├── sf1/          # Scale Factor 1
│   ├── sf10/         # Scale Factor 10
│   └── expected/     # 预期结果
├── tpcc/
│   ├── w10/          # 10 warehouses
│   └── w100/         # 100 warehouses
├── sql/
│   ├── postgresql/   # PostgreSQL 兼容测试
│   ├── mysql/        # MySQL 兼容测试
│   └── sqlite/       # SQLite 兼容测试
└── fuzz/
    ├── corpora/      # 初始 fuzz 语料
    └── seeds/        # 手动添加的种子
```

### 4.2 测试环境隔离

**问题**: 测试之间相互影响

**解决方案**:
```rust
// src/testing/isolated_env.rs
pub struct IsolatedTestEnv {
    _temp_dir: TempDir,
    db: SQLRustGo,
}

impl IsolatedTestEnv {
    pub fn new() -> Self {
        let temp_dir = TempDir::new("sqlrustgo_test").unwrap();
        let db = SQLRustGo::new_with_path(temp_dir.path());
        Self { _temp_dir: temp_dir, db }
    }

    pub fn db(&self) -> &SQLRustGo {
        &self.db
    }
}

impl Drop for IsolatedTestEnv {
    fn drop(&mut self) {
        // 清理资源
        self.db.shutdown();
    }
}
```

### 4.3 测试报告

**改进报告格式**:
```json
{
  "test_report": {
    "timestamp": "2026-05-14T12:00:00Z",
    "total_tests": 1500,
    "passed": 1495,
    "failed": 5,
    "skipped": 0,
    "coverage": {
      "line": 85.2,
      "branch": 72.1,
      "function": 90.0
    },
    "duration_seconds": 342,
    "failures": [
      {
        "test": "test_mvcc_snapshot_isolation",
        "file": "tests/mvcc_test.rs",
        "line": 142,
        "message": "Snapshot value mismatch",
        "expected": "100",
        "actual": "150"
      }
    ]
  }
}
```

---

## 5. CI/CD 集成改进

### 5.1 多平台测试矩阵

```yaml
# .github/workflows/multi-platform.yml
name: Multi-Platform Test

on: [pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@${{ matrix.rust }}
      - name: Test
        run: cargo test --all-features
      - name: Clippy
        run: cargo clippy --all-features -- -D warnings
```

### 5.2 增量覆盖率检查

```yaml
# .github/workflows/coverage-gate.yml
name: Coverage Gate

on:
  pull_request:

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov
      - name: Generate coverage
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info
      - name: Check coverage
        run: |
          TOTAL=$(cargo llvm-cov report --lcov-only 2>/dev/null | grep Total | awk '{print $2}')
          echo "Total coverage: $TOTAL%"
          if (( $(echo "$TOTAL < 80" | bc -l) )); then
            echo "ERROR: Coverage below 80%"
            exit 1
          fi
```

---

## 6. 量化目标

### 6.1 测试覆盖率目标

| 阶段 | 版本 | 行覆盖率 | 分支覆盖率 |
|------|------|----------|------------|
| 当前 | v3.1.0 | ~60% | ~45% |
| Phase 1 | v3.2.0 | 75% | 60% |
| Phase 2 | v3.3.0 | 85% | 70% |
| Phase 3 | v3.4.0 | 90% | 80% |

### 6.2 测试用例数量目标

| 类型 | 当前 | v3.2.0 | v3.3.0 | v3.4.0 |
|------|------|--------|--------|--------|
| 单元测试 | 500 | 600 | 700 | 800 |
| sqllogictest | 100 | 300 | 500 | 700 |
| 属性测试 | 0 | 50 | 100 | 150 |
| 模糊测试 | 0 | 2 targets | 5 targets | 10 targets |
| 基准测试 | 22 TPC-H | +TPC-C | +YCSB | +LinkBench |

### 6.3 缺陷逃逸率目标

| 指标 | 当前 | 目标 |
|------|------|------|
| 缺陷逃逸率 | ~15% | <5% |
| 测试发现缺陷 | 85% | 95% |
| CI 检测率 | 90% | 99% |

---

## 7. 实施检查清单

### Phase 1 (v3.2.0)
```
□ sqllogictest 适配器完善
□ 新增 200+ sqllogictest 测试用例
□ Clippy 严格化配置
□ 测试数据管理框架
□ 隔离测试环境实现
□ 覆盖率 >75%
```

### Phase 2 (v3.3.0)
```
□ cargo-fuzz 集成
□ 至少 3 个 fuzz target
□ 属性测试框架
□ 至少 50 个属性测试
□ TPC-C 自动化
□ 每日性能基准测试
□ 覆盖率 >85%
```

### Phase 3 (v3.4.0)
```
□ TLA+ MVCC 规格
□ TLA+ WAL Recovery 规格
□ Model Checking (SPIN)
□ 锁协议验证
□ 形式化验证 CI
□ 覆盖率 >90%
```

---

## 8. 参考资源

- PostgreSQL Regression Tests: https://www.postgresql.org/docs/devel/regress-init.html
- DuckDB Testing: https://github.com/duckdb/duckdb/tree/main/test
- SQLite Test Methodology: https://www.sqlite.org/testing.html
- TLA+ Example: https://github.com/tlaplus/Examples
- SPIN Documentation: https://spinroot.com/spin/Man/Manual.html
- Property-Based Testing: https:// propify.rs/

---

*文档版本: 1.0*
*创建日期: 2026-05-14*
*基于研究: /tmp/db_qa_research.md*
