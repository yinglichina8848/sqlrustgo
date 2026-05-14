# SQLRustGo 工具集成指南

> **版本**: 1.0
> **日期**: 2026-05-14
> **基于**: /tmp/db_qa_research.md 数据库开发与测试体系研究报告

---

## 1. 概述

本文档为 SQLRustGo 提供全面的工具集成指南，涵盖静态分析、测试框架、基准测试、形式化验证和 CI/CD 流水线。

### 1.1 工具分类总览

| 类别 | 工具 | 优先级 | 适用场景 |
|------|------|--------|----------|
| 静态分析 | Clippy | P0 | 日常开发、CI |
| 静态分析 | Miri | P1 | UB 检测、内存安全 |
| 静态分析 | rust-analyzer | P0 | IDE 支持 |
| 静态分析 | Sanitizers | P1 | 运行时检测 |
| 测试框架 | sqllogictest | P0 | SQL 正确性测试 |
| 测试框架 | Fuzzing | P1 | 异常输入测试 |
| 测试框架 | Property-based | P2 | 边界条件测试 |
| 基准测试 | TPC-C/H/DS | P0 | OLTP/OLAP 性能 |
| 基准测试 | pgbench | P1 | PostgreSQL 兼容性 |
| 基准测试 | YCSB/LinkBench | P2 | 云场景/社交图谱 |
| 形式化验证 | TLA+ | P1 | 协议设计验证 |
| 形式化验证 | Model Checking | P2 | 并发验证 |

---

## 2. 静态分析工具集成

### 2.1 Clippy

**工具信息**
- **名称**: Clippy
- **版本**: Rust 1.80+ 内置
- **类型**: Linter/Static Analyzer
- **官网**: https://github.com/rust-lang/rust-clippy

**集成步骤**

1. 确保 Rust 1.80+ 已安装:
```bash
rustup update
rustc --version  # >= 1.80.0
```

2. 运行 Clippy:
```bash
cargo clippy --all-features --all-targets -- -D warnings
```

3. CI 集成 - `.github/workflows/clippy.yml`:
```yaml
name: Clippy Lint
on: [pull_request, push]
jobs:
  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --all-features -- -D warnings
```

**配置示例**

在 `clippy.toml` 中添加项目特定规则:
```toml
# 允许特定 lint 组
msrv = "1.80"
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 8
```

在 `Cargo.toml` 中调整 lint 级别:
```toml
[lints.rust]
unsafe_code = "deny"
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P0 | 必需工具，发现 SQL 解析器中的常见错误 |
| 集成难度 | 低 | 原生支持，开箱即用 |
| 误报率 | 低 | Rust 生态最成熟的 linter |
| 性能影响 | 无 | 仅静态分析 |

**SQLRustGo 特定规则**:
```bash
# 针对 SQL 解析/执行器的额外检查
cargo clippy --all-features -- \
  -A clippy::cognitive_complexity \
  -D clippy::db_macro_padding
```

---

### 2.2 Miri

**工具信息**
- **名称**: Miri (Mid-level IR Interpreter)
- **版本**: nightly-only，需要 `miri` component
- **类型**: Undefined Behavior Detector
- **官网**: https://github.com/rust-lang/miri

**集成步骤**

1. 安装 Miri:
```bash
rustup +nightly component add miri
```

2. 运行 Miri (仅支持 x86_64-linux):
```bash
cargo +nightly miri test
cargo +nightly miri test --lib
```

3. 检测数据竞争:
```bash
MIRI_Soanitize=thread cargo +nightly miri test
```

**配置示例**

`.cargo/config.toml`:
```toml
[build]
rustflags = ["-Zmiri-disable-isolation"]
```

测试文件级别配置 `tests/miri_test.rs`:
```rust
#![miri::cfg(miri)]
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P1 | 对内存安全要求高的存储引擎必须 |
| 集成难度 | 中 | 仅支持 Linux x86_64 |
| 覆盖范围 | 高 | 检测 UB、内存泄漏、数据竞争 |
| 限制 | 仅测试代码 | 不能测试 FFI 绑定 |

**推荐使用场景**:
- MVCC 快照管理代码
- WAL 写入/恢复逻辑
- 内存分配器实现
- 并发控制原语

---

### 2.3 Sanitizers

**工具信息**
- **支持版本**: Rust 1.80+
- **类型**: Runtime Memory/Behavior Sanitizer

**Sanitizer 类型**

| Sanitizer | 用途 | 标志 | 适用场景 |
|-----------|------|------|----------|
| AddressSanitizer (ASan) | 内存错误 | `-Z sanitizer=address` | 堆溢出、use-after-free |
| UndefinedBehaviorSanitizer (UBSan) | 未定义行为 | `-Z sanitizer=undefined` | 整数溢出、空指针 |
| ThreadSanitizer (TSan) | 数据竞争 | `-Z sanitizer=thread` | 多线程并发 |
| LeakSanitizer (LSan) | 内存泄漏 | `-Z sanitizer=leak` | 资源泄漏 |

**集成步骤**

1. AddressSanitizer:
```bash
RUSTFLAGS="-Z sanitizer=address" cargo +nightly build --tests
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test
```

2. UndefinedBehaviorSanitizer:
```bash
RUSTFLAGS="-Z sanitizer=undefined" cargo +nightly build --tests
RUSTFLAGS="-Z sanitizer=undefined" cargo +nightly test
```

3. ThreadSanitizer:
```bash
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly build --tests
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test
```

**CI 集成 - `.github/workflows/sanitizers.yml`:
```yaml
name: Sanitizers
on: [push, pull_request]
jobs:
  asan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - name: Run ASan
        env:
          RUSTFLAGS: "-Z sanitizer=address"
        run: cargo test -Z sanitizer=address

  ubsan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - name: Run UBSan
        env:
          RUSTFLAGS: "-Z sanitizer=undefined"
        run: cargo test -Z sanitizer=undefined
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P1 | 数据库存储引擎必须检测内存安全 |
| 集成难度 | 中 | 需要 nightly Rust |
| 性能影响 | 2-5x 降低 | 需在专用 CI 跑 |
| 兼容性 | 仅 Linux | Windows/macOS 支持有限 |

---

### 2.4 rust-analyzer

**工具信息**
- **名称**: rust-analyzer
- **版本**: 稳定版
- **类型**: Language Server Protocol (LSP)
- **官网**: https://rust-analyzer.github.io/

**集成步骤**

1. VS Code: 安装 `rust-lang.rust-analyzer` 扩展
2. Neovim: 使用 `nvim-lspconfig` 配置
3. Emacs: 使用 `eglot` 或 `corfu`

**配置示例**

`.vscode/settings.json`:
```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.procMacro.enable": true
}
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P0 | 开发效率工具，必需 |
| 集成难度 | 无 | 即装即用 |
| 功能 | 完整 | 补全、跳转、类型推断、诊断 |

---

## 3. 测试框架集成

### 3.1 sqllogictest

**工具信息**
- **名称**: sqllogictest
- **版本**: 0.9+ (DuckDB/PostgreSQL 兼容格式)
- **类型**: SQL 回归测试框架
- **参考**: DuckDB 使用此框架

**集成步骤**

1. 添加依赖 `Cargo.toml`:
```toml
[dev-dependencies]
sqllogictest = "0.9"
```

2. 创建测试文件 `tests/sql/syntax.test`:
```sql
# test: basic SELECT
query I
SELECT 1;
----
1

# test: WHERE clause
query I
SELECT * FROM (VALUES (1), (2), (3)) AS t(x) WHERE x > 1;
----
2
3
```

3. 编写测试代码:
```rust
use sqllogictest::{ColumnType, DBOutput, Runner};

fn main() {
    let mut runner = Runner::new(MyDB::new());
    runner.run_file("tests/sql/syntax.test");
}

impl sqllogictest::DB for MyDB {
    fn run(&self, sql: &str) -> DBOutput {
        // 执行 SQL 并返回结果
        match self.execute(sql) {
            Ok(rows) => DBOutput::Rows {
                columns: vec![ColumnType::Any],
                rows: rows.into_iter().map(|r| r.values).collect(),
            },
            Err(e) => DBOutput::Error(e.to_string()),
        }
    }
}
```

**SQLRustGo 适配层示例**:
```rust
// src/testing/sqllogictest_adapter.rs
use sqllogictest::{mysql, postgres};

pub struct SQLRustGoDB {
    // SQLRustGo 连接
}

impl sqllogictest::DB for SQLRustGoDB {
    fn run(&self, sql: &str) -> DBOutput {
        match self.query(sql) {
            Ok(result) => result.into(),
            Err(e) => DBOutput::Error(format!("{}", e)),
        }
    }

    fn engine_name(&self) -> &str {
        "sqlrustgo"
    }
}
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P0 | 核心测试框架，参考 DuckDB |
| 兼容性 | 高 | 兼容 PostgreSQL 测试用例 |
| 学习曲线 | 低 | 简单文本格式 |
| 覆盖率 | SQL 逻辑 | 不覆盖网络/并发 |

---

### 3.2 Fuzzing (模糊测试)

**工具信息**
- **推荐工具**: cargo-fuzz, libFuzzer
- **版本**: Rust 1.80+ 内置 support
- **类型**: 随机输入测试

**集成步骤**

1. 设置 libFuzzer 兼容的 Rust:
```bash
# 安装 cargo-fuzz
cargo install cargo-fuzz
```

2. 创建 fuzz target:
```bash
cargo fuzz init
```

3. 编写 fuzz target `fuzz_targets/fuzz_sql.rs`:
```rust
#![no_main]
use libfuzzer_sys::fuzz;

fuzz!(|data: &[u8]| {
    if let Ok(sql) = std::str::from_utf8(data) {
        let db = SQLRustGo::new();
        // 忽略语法错误，只检测 panic/UB
        let _ = db.execute(sql);
    }
});
```

4. 运行模糊测试:
```bash
cargo fuzz run fuzz_sql
```

5. SQLite dbsqlfuzz 风格 - 同时变异 SQL 和数据库文件:
```rust
fuzz!(|data: &[u8]| {
    if data.len() < 10 { return; }
    
    // 前 4 字节: 数据库操作
    // 剩余: SQL 语句
    let (db_op, sql) = data.split_at(4);
    
    let db = unsafe { SQLRustGo::from_bytes(db_op) };
    let _ = db.execute(std::str::from_utf8(sql).unwrap_or(""));
});
```

**CI 集成 - `.github/workflows/fuzz.yml`:
```yaml
name: Fuzz
on:
  schedule:
    - cron: '0 2 * * *'  # Nightly
  push:
    branches: [main]
jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - name: Run fuzz_sql
        run: |
          cargo install cargo-fuzz
          cargo fuzz build fuzz_sql
          # 持续 1 小时
          timeout 3600 cargo fuzz run fuzz_sql || true
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P1 | 发现边界条件和异常输入 |
| 覆盖范围 | 语法/执行器 | 检测 panic 和 UB |
| 性能影响 | 高 | 建议仅在 nightly CI 运行 |

---

### 3.3 Property-Based Testing (属性测试)

**工具信息**
- **推荐工具**: proptest, quickcheck
- **版本**: proptest 1.1+, quickcheck 1.0+
- **类型**: 基于属性的测试

**集成步骤**

1. 添加依赖 `Cargo.toml`:
```toml
[dev-dependencies]
proptest = "1.1"
```

2. 编写属性测试 `tests/property_tests.rs`:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_sql_parser_commutative(sql in "\\PC+") {
        // 解析两次应该产生相同结果
        let ast1 = parse(&sql);
        let ast2 = parse(&sql);
        assert_eq!(ast1, ast2);
    }

    #[test]
    fn test_query_result_determinism(queries in any::<Vec<String>>()) {
        let db = SQLRustGo::new();
        for q in &queries {
            let result1 = db.query(q);
            let result2 = db.query(q);
            assert_eq!(result1, result2, "Query: {}", q);
        }
    }
}
```

3. SQL 生成器示例:
```rust
use proptest::string::{string_regex, StringRegex};

prop_compose! {
    fn arb_valid_sql()
        (depth in 1..10i32)
        (sql in gen_sql(depth)) -> String {
            sql
        }
}

fn gen_sql(depth: i32) -> impl Strategy<Value = String> {
    if depth <= 0 {
        Just("SELECT 1".to_string())
    } else {
        prop_oneof![
            gen_simple_select(),
            format!("({}) UNION ({})", gen_sql(depth-1), gen_sql(depth-1)),
            format!("({}) JOIN ({}) ON 1=1", gen_sql(depth-1), gen_sql(depth-1)),
        ]
    }
}
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P2 | 补充边界条件测试 |
| 复杂度 | 中 | 需要定义正确的属性 |
| 适用场景 | 解析器、执行器 | 验证代数性质 |

---

## 4. 基准测试工具集成

### 4.1 TPC-C

**工具信息**
- **名称**: TPC-C
- **版本**: TPC-C 5.x
- **类型**: OLTP 事务处理基准
- **规范**: https://www.tpc.org/tpcc/

**集成步骤**

1. 准备 TPC-C 数据生成器 (推荐使用 HammerDB 或 pg必备工具):
```bash
# 使用 HammerDB CLI
wget https://github.com/TPC-Council/HammerDB/releases/download/v4.8/HammerDB-4.8-Linux.tar.gz
tar -xzf HammerDB-4.8-Linux.tar.gz
```

2. SQLRustGo TPC-C 适配:
```rust
// src/benchmark/tpcc.rs
pub struct TPCCTables {
    pub warehouse: Vec<Warehouse>,
    pub district: Vec<District>,
    pub customer: Vec<Customer>,
    pub orders: Vec<Order>,
    pub order_line: Vec<OrderLine>,
    pub item: Vec<Item>,
    pub stock: Vec<Stock>,
    pub new_orders: Vec<NewOrder>,
}
```

3. 运行 TPC-C:
```bash
# 1. 创建表
psql -f scripts/benchmarks/tpcc/schema.sql

# 2. 加载数据 (100 warehouses)
./hammerdbcli auto scripts/benchmarks/tpcc/tpcc_run.tcl

# 3. 运行基准测试
./hammerdbcli auto scripts/benchmarks/tpcc/tpcc_test.tcl
```

**TPC-C SQL Schema (简化)**:
```sql
CREATE TABLE warehouse (
    w_id INTEGER PRIMARY KEY,
    w_name VARCHAR(10),
    w_street_1 VARCHAR(20),
    w_city VARCHAR(20),
    w_state CHAR(2),
    w_zip CHAR(9),
    w_ytd DECIMAL(12,2)
);

CREATE TABLE district (
    d_id INTEGER,
    d_w_id INTEGER,
    d_name VARCHAR(10),
    d_street_1 VARCHAR(20),
    d_city VARCHAR(20),
    d_state CHAR(2),
    d_zip CHAR(9),
    d_ytd DECIMAL(12,2),
    d_next_o_id INTEGER,
    PRIMARY KEY (d_w_id, d_id)
);
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P0 | 核心 OLTP 性能指标 |
| 复杂度 | 高 | 需要完整事务支持 |
| 数据量 | TB 级 | 需要足够大的测试数据 |

---

### 4.2 TPC-H

**工具信息**
- **名称**: TPC-H
- **版本**: TPC-H 3.0
- **类型**: Decision Support / Ad-hoc 查询
- **规范**: https://www.tpc.org/tpch/

**集成步骤**

1. 安装 TPC-H 数据生成工具 dbgen:
```bash
git clone https://github.com/electrum/tpch-dbgen.git
cd tpch-dbgen
make
```

2. 生成测试数据 (SF=1):
```bash
./dbgen -s 1 -f  # 生成 ~1GB 数据
```

3. SQLRustGo TPC-H 查询适配 - 参考 `docs/TPC-H-TEST-GUIDE.md`

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P0 | 分析型查询性能基准 |
| 复杂度 | 中 | 22 条标准查询 |
| SQL 覆盖 | 复杂查询 | JOIN、聚合、子查询 |

---

### 4.3 pgbench

**工具信息**
- **名称**: pgbench (PostgreSQL Bench)
- **版本**: PostgreSQL 16+
- **类型**: TPC-B 风格基准
- **特点**: 简单易用，内置 PostgreSQL

**集成步骤**

1. 安装 PostgreSQL:
```bash
# macOS
brew install postgresql@16

# Ubuntu
apt install postgresql-16
```

2. 初始化 pgbench:
```bash
pgbench -i -s 50 sqlrustgo  # 50 scale factor
```

3. 运行基准测试:
```bash
# 基本测试
pgbench -c 10 -j 2 -T 60 sqlrustgo

# 高级测试
pgbench -c 10 -j 2 -T 60 -M prepared -S -n sqlrustgo
```

4. 自定义测试脚本 `scripts/benchmarks/pgbench/custom.sql`:
```sql
-- TPC-B style transaction
\set aid random(1, 100000 * :scale)
\set bid random(1, 1 * :scale)
\timing on
BEGIN;
UPDATE pgbench_accounts SET abalance = abalance + 1 WHERE aid = :aid;
SELECT abalance FROM pgbench_accounts WHERE aid = :aid;
UPDATE pgbench_tellers SET tbalance = tbalance + 1 WHERE tid = :tid;
UPDATE pgbench_branches SET bbalance = bbalance + 1 WHERE bid = :bid;
INSERT INTO pgbench_history (tid, bid, aid, delta, mtime) VALUES (:tid, :bid, :aid, 1, NOW());
COMMIT;
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P1 | PostgreSQL 兼容性测试 |
| 复杂度 | 低 | 工具成熟，使用简单 |
| 兼容性 | 原生 | 测试 PostgreSQL wire protocol |

---

### 4.4 YCSB (Yahoo! Cloud Serving Benchmark)

**工具信息**
- **名称**: YCSB
- **版本**: 0.17+
- **类型**: 云服务基准
- **GitHub**: https://github.com/brianfrankcooper/YCSB

**集成步骤**

1. 安装 YCSB:
```bash
git clone https://github.com/brianfrankcooper/YCSB.git
cd YCSB
mvn -pl site.ycsb:postgresql -am clean package
```

2. SQLRustGo JDBC/PostgreSQL 连接配置:
```properties
db.driver=org.postgresql.Driver
db.url=jdbc:postgresql://localhost:5432/sqlrustgo
db.user=benchmark
db.passwd=benchmark

recordcount=1000000
operationcount=500000

workload=site.ycsb.workloads.CoreWorkload

readproportion=0.5
updateproportion=0.5
scanproportion=0
insertproportion=0
```

3. 运行 YCSB:
```bash
./bin/ycsb load postgresql -P workloads/workloada -p recordcount=1000000
./bin/ycsb run postgresql -P workloads/workloada -p operationcount=500000
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P2 | 云场景参考 |
| 适用性 | 中 | 主要测试 KV 场景 |
| 价值 | 中 | 可对比 Redis/Cassandra |

---

### 4.5 LinkBench

**工具信息**
- **名称**: LinkBench
- **开发者**: Facebook
- **类型**: 社交图谱基准
- **GitHub**: https://github.com/facebookarchive/linkbench

**集成步骤**

1. 编译 LinkBench:
```bash
git clone https://github.com/facebookarchive/linkbench.git
cd linkbench
make
```

2. 配置 `config/LinkConfig.properties`:
```properties
username=benchmark
password=benchmark
dbid=1
servers=localhost
port=5432

# 图数据规模
maxid1=100000000  # 节点数
maxid2=1000000000 # 边数
```

3. 运行基准测试:
```bash
./bin/linkbench_load config/LinkConfig.properties
./bin/linkbench config/LinkConfig.properties -l phases=load,request
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P2 | 特定社交图谱场景 |
| 复杂度 | 高 | 图查询实现复杂 |
| 价值 | 中 | 补充 OLTP/OLAP 场景 |

---

## 5. 形式化验证工具集成

### 5.1 TLA+

**工具信息**
- **名称**: TLA+ (Temporal Logic of Actions)
- **版本**: TLA+ Toolbox 1.7+
- **类型**: 规格说明语言 + 模型检验
- **官网**: https://lamport.azurewebsites.net/tla/tla.html

**集成步骤**

1. 安装 TLA+ Toolbox:
```bash
# 下载 https://github.com/tlaplus/tlaplus/releases
java -jar tla2tools.jar
```

2. 创建 TLA+ 规格 `specs/MVCC.tla`:
```tla
----------------------------- MODULE MVCC ----------------------------
EXTENDS Integers, Sequences, FiniteSets

CONSTANT 
    MaxTxn,
    MaxObject,
    NumWorkers

VARIABLES
    transactions,
    objects,
    aborted,
    committed

TypeOK ==
    /\ transactions \in [1..MaxTxn] -> 
        [status: {"active", "committed", "aborted"}, 
         snapshot: 0..MaxTxn,
         writes: SUBSET (1..MaxObject)]
    /\ objects \in [1..MaxObject] -> 
        [value: Int, version: 0..MaxTxn]
    /\ aborted \subseteq 1..MaxTxn
    /\ committed \subseteq 1..MaxTxn

SnapshotIsolation(tx) ==
    /\ transactions[tx].status = "active"
    /\ \forall oid \in transactions[tx].writes :
        objects[oid].version = tx
    /\ \forall other_tx \in DOMAIN transactions :
        /\ other_tx # tx
        /\ transactions[other_tx].status = "committed"
        /\ transactions[other_tx].snapshot < tx
        /\ \exists oid \in transactions[tx].writes :
            objects[oid].version \notin {tx, transactions[other_tx].snapshot}
    => abort(tx)

Commit(tx) ==
    /\ transactions[tx].status = "active"
    /\ \forall oid \in transactions[tx].writes :
        objects' = [objects EXCEPT ![oid] = 
            [value |-> transactions[tx].value, version |-> tx]]
    /\ committed' = committed \cup {tx}
    /\ transactions' = [transactions EXCEPT ![tx].status = "committed"]

Abort(tx) ==
    /\ transactions[tx].status = "active"
    /\ aborted' = aborted \cup {tx}
    /\ transactions' = [transactions EXCEPT ![tx].status = "aborted"]

Next ==
    \E tx \in 1..MaxTxn :
        \/ SnapshotIsolation(tx)
        \/ Commit(tx)
        \/ Abort(tx)

Spec == Init /\ [][Next]_<<transactions, objects, aborted, committed>>
========================================================================
```

3. 模型检验:
```bash
# 使用 TLC 模型检验器
java -cp tla2tools.jar tlc2.TLC specs/MVCC.tla -model
```

**TLA+ 规格示例 - WAL Recovery**:
```tla
----------------------------- MODULE WAL -----------------------------
EXTENDS Integers, Sequences

VARIABLES
    log,
    disk,
    state

WriteAheadLog(txn_id, operation) ==
    /\ log' = Append(log, [txn_id |-> txn_id, op |-> operation])
    /\ state' = Apply(operation, state)

Recovery ==
    \forall entry \in log :
        entry.op.type = "commit"
        => state' = Apply(entry.op, state)

Spec == Init /\ [][Next]_<<log, disk, state>>
========================================================================
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P1 | 核心协议验证必需 |
| 复杂度 | 高 | 需要 TLA+ 专业知识 |
| 适用场景 | MVCC/WAL/事务 | 协议设计阶段 |
| 参考 | PostgreSQL/DuckDB | 均有使用 TLA+ 验证 |

---

### 5.2 Model Checking (SPIN)

**工具信息**
- **名称**: SPIN (Simple Promela Interpreter)
- **版本**: SPIN 6+
- **类型**: 并发系统模型检验
- **官网**: https://spinroot.com/

**集成步骤**

1. 安装 SPIN:
```bash
# macOS
brew install spin

# Ubuntu
apt install spin
```

2. 创建 Promela 模型 `specs/concurrent_access.pml`:
```promela
mtype = { READ, WRITE, COMMIT, ABORT };

chan log = [0] of { mtype, int, int };
int object_value = 0;
int lock_holder = -1;

proctype Transaction(int txn_id) {
    do
    ::  log!WRITE(txn_id, object_value);
        if
        ::  lock_holder == -1 ->
            lock_holder = txn_id;
            object_value = object_value + 1;
            log!COMMIT(txn_id, 0);
            lock_holder = -1;
            break
        ::  else ->
            log!ABORT(txn_id, 0);
            break
        fi
    od
}

init {
    atomic {
        run Transaction(1);
        run Transaction(2);
        run Transaction(3);
    }
}
```

3. 运行验证:
```bash
spin -a concurrent_access.pml
gcc -O2 -o pan pan.c
./pan -m10000  # 验证深度 10000
```

**对 SQLRustGo 的适用性分析**

| 维度 | 评分 | 说明 |
|------|------|------|
| 必要性 | P2 | 补充 TLA+ |
| 复杂度 | 高 | 需要 Promela 语言 |
| 适用场景 | 锁协议 | 死锁检测 |

---

## 6. CI/CD 集成

### 6.1 GitHub Actions

**基础工作流 - `.github/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: [main, develop/*]
  pull_request:
    branches: [main, develop/*]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # R1: Build
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --all-features --verbose
      - name: Build tests
        run: cargo build --tests --all-features

  # R2: Test
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Test
        run: cargo test --all-features -- --nocapture
      - name: Test with coverage
        run: cargo test --all-features --coverage

  # R3: Clippy
  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - name: Run Clippy
        run: cargo clippy --all-features -- -D warnings

  # R4: Format
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - name: Check formatting
        run: cargo fmt --all -- --check

  # R5: Coverage
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov
      - name: Generate coverage
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info
      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: true
          threshold: 80%

  # R6: Security
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Audit
        run: cargo audit

  # R7: Docs
  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check docs build
        run: cargo doc --no-deps --all-features
        env:
          RUSTDOCFLAGS: "-D warnings"
```

**高级工作流 - `.github/workflows/r-gate.yml`:
```yaml
name: R-Gate Verification

on:
  pull_request:
    branches: [main, develop/*]

jobs:
  # R1-R7: Core Gates (并行)
  core-gates:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        gate: [build, test, clippy, fmt, coverage, security, docs]
    steps:
      - uses: actions/checkout@v4
      - name: Run Gate ${{ matrix.gate }}
        run: |
          case ${{ matrix.gate }} in
            build) cargo build --all-features ;;
            test) cargo test --all-features ;;
            clippy) cargo clippy --all-features -- -D warnings ;;
            fmt) cargo fmt --all -- --check ;;
            security) cargo audit ;;
            docs) cargo doc --no-deps --all-features ;;
          esac

  # R8: SQL Compatibility
  sql-compat:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check SQL Corpus
        run: |
          SQL_CORPUS=$(find tests/sql -name "*.test" | wc -l)
          echo "SQL Corpus: $SQL_CORPUS files"
          if [ $SQL_CORPUS -lt 100 ]; then
            echo "ERROR: SQL Corpus below 100 files"
            exit 1
          fi

  # R9: Performance
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run TPC-H
        run: cargo test --test tpch
      - name: Check baseline
        run: |
          ./scripts/benchmark/check_baseline.sh
          if [ $? -ne 0 ]; then
            echo "WARNING: Performance regression detected"
          fi

  # R10: Formal Proof
  formal-proof:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check Proof Files
        run: |
          PROOF_COUNT=$(find docs/proof -name "PROOF-*.md" | wc -l)
          echo "Proof files: $PROOF_COUNT"
          if [ $PROOF_COUNT -lt 10 ]; then
            echo "ERROR: Need at least 10 proof files"
            exit 1
          fi

  # 合并门控
  merge-gate:
    needs: [core-gates, sql-compat, performance, formal-proof]
    runs-on: ubuntu-latest
    steps:
      - name: All gates passed
        run: echo "All R-Gates passed, merge enabled"
```

### 6.2 Gitea Actions

**工作流配置 - `.gitea/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: [main, develop/*]
  pull_request:

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Build
        run: cargo build --all-features --verbose

      - name: Test
        run: cargo test --all-features

      - name: Clippy
        run: cargo clippy --all-features -- -D warnings

      - name: Format
        run: cargo fmt --all -- --check
```

---

## 7. 代码覆盖率

### 7.1 cargo-llvm-cov

**工具信息**
- **名称**: cargo-llvm-cov
- **版本**: 0.6+
- **类型**: LLVM 覆盖率工具
- **官网**: https://github.com/taiki-e/cargo-llvm-cov

**集成步骤**

1. 安装:
```bash
cargo install cargo-llvm-cov
```

2. 生成覆盖率报告:
```bash
# 文本报告
cargo llvm-cov --all-features

# HTML 报告
cargo llvm-cov --all-features --html --open

# LCOV 格式 (用于 Codecov)
cargo llvm-cov --all-features --lcov --output-path lcov.info
```

3. CI 集成:
```yaml
- name: Install cargo-llvm-cov
  uses: taiki-e/install-action@cargo-llvm-cov

- name: Generate coverage
  run: cargo llvm-cov --all-features --lcov --output-path lcov.info

- name: Upload to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: lcov.info
    fail_ci_if_error: true
```

**Codecov 配置 - `codecov.yml`:
```yaml
coverage:
  precision: 2
  round: down
  range: "70...100"

  status:
    project:
      default:
        target: 80%
        threshold: 1%
    patch:
      default:
        target: 80%
```

### 7.2 grcov

**工具信息**
- **名称**: grcov
- **版本**: 0.8+
- **类型**: GCC/Rust 覆盖率收集器
- **官网**: https://github.com/mozilla/grcov

**集成步骤**

1. 安装:
```bash
cargo install grcov
```

2. 收集覆盖率:
```bash
# 使用 LLVM
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="coverage-%p.profraw"

cargo build --tests
cargo test
grcov . --binary-path ./target/debug/deps/ \
  -s . -t html --branch --keep-only "src/*" \
  -o ./target/coverage/html/
```

3. CI 集成:
```yaml
- name: Run tests with coverage
  env:
    CARGO_INCREMENTAL: 0
    RUSTFLAGS: "-C instrument-coverage"
  run: |
    cargo test
    grcov . --binary-path ./target/debug/deps/ \
      -s . -t lcov --branch \
      -o ./target/lcov.info

- name: Upload coverage
  uses: codecov/codecov-action@v4
  with:
    files: target/lcov.info
```

**对 SQLRustGo 的适用性分析**

| 工具 | 评分 | 说明 |
|------|------|------|
| cargo-llvm-cov | P0 | 推荐首选，集成简单 |
| grcov | P1 | 备选，支持更多格式 |

---

## 8. 工具清单汇总

### 8.1 优先级矩阵

| 优先级 | 工具 | 类型 | 集成状态 |
|--------|------|------|----------|
| P0 | Clippy | 静态分析 | 必须 |
| P0 | rust-analyzer | IDE | 必须 |
| P0 | sqllogictest | 测试 | 必须 |
| P0 | cargo-llvm-cov | 覆盖率 | 必须 |
| P0 | GitHub Actions | CI/CD | 必须 |
| P1 | Miri | UB 检测 | 推荐 |
| P1 | Sanitizers | 运行时检测 | 推荐 |
| P1 | pgbench | 基准测试 | 推荐 |
| P1 | TPC-C/H | 基准测试 | 推荐 |
| P1 | TLA+ | 形式化验证 | 推荐 |
| P2 | Fuzzing | 模糊测试 | 可选 |
| P2 | Property-based | 属性测试 | 可选 |
| P2 | YCSB | 基准测试 | 场景可选 |
| P2 | LinkBench | 基准测试 | 场景可选 |
| P2 | SPIN | 模型检验 | 高级可选 |

### 8.2 集成检查清单

```
□ Clippy 配置完成
□ rust-analyzer 配置完成
□ sqllogictest 适配器实现
□ cargo-llvm-cov CI 集成
□ GitHub Actions 流水线配置
□ Sanitizers CI 配置 (可选)
□ TPC-H 测试用例导入
□ pgbench 脚本准备
□ TLA+ 规格文件 (MVCC, WAL)
□ Fuzzing target 编写 (可选)
□ Property-based 测试编写 (可选)
```

---

## 9. 参考资源

- PostgreSQL Testing: https://www.postgresql.org/docs/devel/
- DuckDB Contributing: https://github.com/duckdb/duckdb/blob/main/CONTRIBUTING.md
- SQLite Testing: https://www.sqlite.org/testing.html
- TPC Benchmarks: https://www.tpc.org/
- Rust Clippy: https://rust-lang.github.io/rust-clippy/
- Miri: https://github.com/rust-lang/miri
- TLA+ toolbox: https://github.com/tlaplus/tlaplus

---

*文档版本: 1.0*
*创建日期: 2026-05-14*
*基于研究: /tmp/db_qa_research.md*
