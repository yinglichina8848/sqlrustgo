# 测试体系工程化 — 全项目可复制覆盖率提升方案

> **For agentic workers:** REQUIRED: Use subagent-driven-development to implement. Steps use checkbox (`- [ ]`) syntax.

**Goal:** 构建全项目可复制的测试分层体系 + 覆盖率门禁（80%），以 v2.8.0 P0 模块（parser 54.70%, executor 60.71%, mysql-server 35.59%, network 0%）为重点突破口。

**Architecture:** 三阶段推进——Phase 1 补齐 P0 模块测试；Phase 2 标准化分层；Phase 3 自动化门禁。

**Tech Stack:** cargo-tarpaulin, cargo-llvm-cov, Rust 2021

---

## 当前状态

| 模块 | 当前覆盖率 | 目标 | 缺口 |
|------|----------|------|------|
| parser | 54.70% | 85% | -30.3% |
| executor | 60.71% | 80% | -19.3% |
| mysql-server | 35.59% | 70% | -34.4% |
| network | 0% | 70% | -70% |
| optimizer | 90.99% | 85% | ✅ 已达标 |
| storage | 83.00% | 75% | ✅ 已达标 |

**CI 覆盖率门禁:** `scripts/gate/check_coverage.sh` — 阈值 **80%**

---

## 统一测试分层标准

```
Layer 1: 单元测试（函数 / struct / 独立逻辑）
Layer 2: 模块测试（Operator / Component / 内部交互）
Layer 3: 组合测试（Pipeline / 跨模块 Flow）
Layer 4: 错误路径测试（panic / error / invalid input）
Layer 5: 边界 / 状态测试（空 / 满 / 极值）
```

---

## Phase 1: P0 模块补测（parser + mysql-server + network）

### Task 1.1: Parser 覆盖率提升至 85%

- [ ] **Step 1: 审查现有测试**
  ```bash
  cargo test -p sqlrustgo-parser -- --list 2>&1 | grep -c "test"
  # 当前约 50 个测试，缺口约 200 行
  ```
  查看: `crates/parser/tests/parser_coverage_tests.rs`

- [ ] **Step 2: 扩展 error_tests（Layer 4）**
  创建: `crates/parser/tests/error_tests.rs`
  ```rust
  // 无效 SQL 解析错误
  #[test]
  #[should_panic(expected = "ParseError")]
  fn test_unterminated_string() { parse("SELECT 'unclosed"); }
  
  #[test]
  #[should_panic(expected = "ParseError")]
  fn test_invalid_operator() { parse("SELECT * FROM t WHERE a @@ b"); }
  ```

- [ ] **Step 3: 扩展 edge_tests（Layer 5）**
  创建: `crates/parser/tests/edge_tests.rs`
  ```rust
  #[test]
  fn test_empty_sql() { assert!(parse("").is_err()); }
  
  #[test]
  fn test_very_long_identifier() {
      let sql = format!("SELECT {}", "a".repeat(1000));
      let _ = parse(&sql);
  }
  ```

- [ ] **Step 4: 扩展 coverage_tests（Layer 2）**
  扩展: `crates/parser/tests/parser_coverage_tests.rs`
  - CREATE TABLE variants
  - INSERT/UPDATE/DELETE
  - JOIN 变体
  - SUBQUERY
  - UNION/INTERSECT

- [ ] **Step 5: 验证覆盖率**
  ```bash
  cargo tarpaulin -p sqlrustgo-parser --out Json --timeout 300
  # 目标: ≥85%
  ```

### Task 1.2: MySQL-Server 覆盖率提升至 70%

- [ ] **Step 1: 审查现有测试**
  查看: `crates/mysql-server/tests/mysql_server_tests.rs`

- [ ] **Step 2: 创建基本测试**
  创建: `crates/mysql-server/tests/packet_tests.rs`
  ```rust
  #[test]
  fn test_handshake_packet_parse() { }
  #[test]
  fn test_query_packet_parse() { }
  ```

- [ ] **Step 3: 创建错误路径测试**
  创建: `crates/mysql-server/tests/error_tests.rs`
  ```rust
  #[test]
  #[should_panic]
  fn test_invalid_packet_sequence() { }
  ```

### Task 1.3: Network 覆盖率从 0% 起步

- [ ] **Step 1: 探索 network 模块结构**
  ```bash
  ls crates/network/src/
  cargo test -p sqlrustgo-network -- --list
  ```

- [ ] **Step 2: 创建基本单元测试**
  创建: `crates/network/tests/unit_tests.rs`
  ```rust
  #[test]
  fn test_packet_encoding() { }
  #[test]
  fn test_packet_decoding() { }
  ```

- [ ] **Step 3: 创建错误路径测试**
  创建: `crates/network/tests/error_tests.rs`
  ```rust
  #[test]
  #[should_panic]
  fn test_invalid_packet_type() { }
  ```

---

## Phase 2: 标准化测试分层

### Task 2.1: 创建统一 common 工具库

- [ ] **Step 1: 创建 common 模块**
  创建: `tests/common/mod.rs`
  ```rust
  pub mod mock_storage;
  pub mod mock_parser;
  pub mod utils;
  ```

- [ ] **Step 2: 创建 MockStorage**
  创建: `tests/common/mock_storage.rs`
  ```rust
  pub struct MockStorage { pub data: Vec<Vec<sqlrustgo_types::Value>> }
  impl MockStorage {
      pub fn new() -> Self { Self { data: Vec::new() } }
      pub fn with_data(data: Vec<Vec<sqlrustgo_types::Value>>) -> Self { Self { data } }
  }
  ```

- [ ] **Step 3: 创建工具函数**
  创建: `tests/common/utils.rs`
  ```rust
  pub fn assert_err<T, E>(res: Result<T, E>) { assert!(res.is_err()); }
  pub fn assert_ok<T, E>(res: Result<T, E>) -> T { res.unwrap() }
  ```

### Task 2.2: 标准化存储模块测试

- [ ] **Step 1: 创建 page_tests**
  创建: `crates/storage/tests/page_tests.rs`
  ```rust
  use sqlrustgo_storage::{Page, StorageEngine};
  
  #[test]
  fn test_page_insert_read() {
      let mut page = Page::new();
      page.insert(1_i64.into());
      assert_eq!(page.read(0), Some(1_i64.into()));
  }
  ```

- [ ] **Step 2: 创建 boundary_tests**
  创建: `crates/storage/tests/boundary_tests.rs`
  ```rust
  #[test]
  fn test_empty_page() { let page = Page::new(); assert_eq!(page.len(), 0); }
  
  #[test]
  #[should_panic(expected = "OutOfBounds")]
  fn test_page_read_oob() { Page::new().read(999); }
  ```

- [ ] **Step 3: 创建 error_tests**
  创建: `crates/storage/tests/error_tests.rs`
  ```rust
  #[test]
  #[should_panic]
  fn test_invalid_page_id() { }
  ```

### Task 2.3: 标准化 Optimizer 测试（维持 85%+）

- [ ] **Step 1: 扩展 rule_tests**
  创建: `crates/optimizer/tests/rule_tests.rs`
  ```rust
  #[test]
  fn test_predicate_pushdown() { }
  #[test]
  fn test_projection_pushdown() { }
  ```

- [ ] **Step 2: 扩展 cost_tests**
  创建: `crates/optimizer/tests/cost_tests.rs`
  ```rust
  #[test]
  fn test_cost_comparison() { }
  ```

---

## Phase 3: CI 覆盖率门禁自动化

### Task 3.1: 更新 CI 覆盖率配置

- [ ] **Step 1: 检查现有 CI 配置**
  查看: `.gitea/workflows/ci.yml` (coverage job)

- [ ] **Step 2: 添加自动触发条件**
  修改: `.gitea/workflows/ci.yml`
  ```yaml
  coverage:
    # 改为 push 到 develop/* 和 PR 自动触发
    on:
      push:
        branches: [develop/v2.9.0, develop/v2.8.0]
      pull_request:
        branches: [develop/v2.9.0, develop/v2.8.0]
  ```

- [ ] **Step 3: 添加阈值检查**
  修改: `scripts/gate/check_coverage.sh`
  ```bash
  # 在现有检查后添加
  if [ "$COVERAGE" -lt 80 ]; then
      echo "Coverage $COVERAGE% below threshold 80%"
      exit 1
  fi
  ```

- [ ] **Step 4: 添加 per-module 报告**
  修改: `.gitea/workflows/ci.yml`
  ```yaml
  - name: Per-module coverage
    run: |
      cargo tarpaulin --all-features --out Json --timeout 600 | \
        python3 .agents/scripts/parse_coverage.py
  ```

### Task 3.2: 创建覆盖率解析脚本

- [ ] **Step 1: 创建解析脚本**
  创建: `.agents/scripts/parse_coverage.py`
  ```python
  #!/usr/bin/env python3
  import json, sys
  d = json.load(sys.stdin)
  for f in d.get('files', []):
      print(f"{f['name']}: {f['line_rate']*100:.1f}%")
  ```

---

## 验收标准

```bash
# Phase 1 完成后
cargo tarpaulin --all-features --out Json --timeout 300
# parser ≥ 85%
# mysql-server ≥ 70%
# network ≥ 50%

# Phase 2 完成后
cargo tarpaulin --all-features --out Json --timeout 300
# storage ≥ 75%
# optimizer ≥ 85%

# Phase 3 完成后
cargo test --all-features && bash scripts/gate/check_coverage.sh
# 全部测试通过
# 覆盖率 ≥ 80%
```

---

## 关键文件清单

| 操作 | 文件路径 |
|------|---------|
| 新增 | `crates/parser/tests/error_tests.rs` |
| 新增 | `crates/parser/tests/edge_tests.rs` |
| 扩展 | `crates/parser/tests/parser_coverage_tests.rs` |
| 新增 | `crates/mysql-server/tests/packet_tests.rs` |
| 新增 | `crates/mysql-server/tests/error_tests.rs` |
| 新增 | `crates/network/tests/unit_tests.rs` |
| 新增 | `crates/network/tests/error_tests.rs` |
| 新增 | `tests/common/mod.rs` |
| 新增 | `tests/common/mock_storage.rs` |
| 新增 | `tests/common/utils.rs` |
| 新增 | `crates/storage/tests/page_tests.rs` |
| 新增 | `crates/storage/tests/boundary_tests.rs` |
| 新增 | `crates/storage/tests/error_tests.rs` |
| 新增 | `crates/optimizer/tests/rule_tests.rs` |
| 新增 | `crates/optimizer/tests/cost_tests.rs` |
| 修改 | `.gitea/workflows/ci.yml` |
| 修改 | `scripts/gate/check_coverage.sh` |
| 新增 | `.agents/scripts/parse_coverage.py` |

---

## 里程碑

| 阶段 | 日期 | 目标 |
|------|------|------|
| Phase 1 | 2026-05-07 | parser ≥85%, mysql-server ≥70%, network ≥50% |
| Phase 2 | 2026-05-14 | storage ≥75%, optimizer ≥85%, 分层标准化 |
| Phase 3 | 2026-05-21 | CI 自动门禁, 全项目 ≥80% |

---

---

## Phase 4: 内核级测试体系（超越覆盖率）

> 目标：验证 SQL 语义正确性 + 引擎一致性 + 抗随机输入能力

### Layer K1: sqllogictest（SQL 语义验证）

**技术:** sqllogictest 是 SQLite/DuckDB/ClickHouse 都在用的标准 SQL 语义验证格式。

- [ ] **Step 1: 添加依赖**
  修改: `Cargo.toml`
  ```toml
  [dev-dependencies]
  sqllogictest = "0.17"
  ```

- [ ] **Step 2: 实现 sqllogictest DB 接口**
  创建: `tests/sql_logic/mod.rs`
  ```rust
  use sqllogictest::DB;

  pub struct SqlRustGoDB {
      engine: ExecutionEngine<MemoryStorage>,
  }

  impl DB for SqlRustGoDB {
      type Error = String;

      fn run(&mut self, sql: &str) -> Result<Vec<Vec<String>>, Self::Error> {
          let result = self.engine.execute(sql)
              .map_err(|e| e.to_string())?;
          Ok(result.rows.iter()
              .map(|row| row.iter()
                  .map(|v| v.to_string())
                  .collect())
              .collect())
      }
  }
  ```

- [ ] **Step 3: 创建基础 sqllogictest 文件**
  创建: `tests/sql_logic/basic.slt`
  ```sql
  statement ok
  CREATE TABLE t(a INT, b TEXT);

  statement ok
  INSERT INTO t VALUES (1, 'hello'), (2, 'world'), (3, NULL);

  query I
  SELECT COUNT(*) FROM t;
  ----
  3

  query I
  SELECT a FROM t WHERE a > 1 ORDER BY a;
  ----
  2
  3

  query I
  SELECT a FROM t WHERE b IS NULL;
  ----
  3
  ```

- [ ] **Step 4: 创建 JOIN sqllogictest**
  创建: `tests/sql_logic/join.slt`
  ```sql
  statement ok
  CREATE TABLE t1(a INT, b TEXT);

  statement ok
  CREATE TABLE t2(a INT, c TEXT);

  statement ok
  INSERT INTO t1 VALUES (1, 'x'), (2, 'y');

  statement ok
  INSERT INTO t2 VALUES (1, 'p'), (3, 'q');

  query I,I
  SELECT t1.a, t2.a FROM t1 INNER JOIN t2 ON t1.a = t2.a;
  ----
  1 1
  ```

- [ ] **Step 5: 创建 Aggregate sqllogictest**
  创建: `tests/sql_logic/aggregate.slt`
  ```sql
  statement ok
  CREATE TABLE orders(customer_id INT, amount INT);

  statement ok
  INSERT INTO orders VALUES (1, 100), (1, 200), (2, 150), (2, 50), (3, 300);

  query I
  SELECT COUNT(*) FROM orders;
  ----
  5

  query I
  SELECT SUM(amount) FROM orders WHERE customer_id = 1;
  ----
  300
  ```

### Layer K2: Differential Testing（对拍 SQLite）

**技术:** 同一条 SQL 在 SQLRustGo 和 SQLite 执行，结果必须一致。

- [ ] **Step 1: 添加 SQLite 依赖**
  修改: `Cargo.toml`
  ```toml
  [dev-dependencies]
  rusqlite = "0.31"
  ```

- [ ] **Step 2: 创建 SQLite Runner 封装**
  创建: `tests/differential/sqlite_runner.rs`
  ```rust
  use rusqlite::Connection;

  pub struct SqliteRunner {
      conn: Connection,
  }

  impl SqliteRunner {
      pub fn new() -> Self {
          let conn = Connection::open_in_memory().unwrap();
          Self { conn }
      }

      pub fn run(&self, sql: &str) -> Result<String, String> {
          let mut stmt = self.conn.prepare(sql).map_err(|e| e.to_string())?;
          let col_count = stmt.column_count();

          let rows: Vec<Vec<String>> = stmt.query_map([], |row| {
              (0..col_count).map(|i| {
                  row.get_ref(i)
                      .map(|v| match v {
                          rusqlite::types::ValueRef::Null => "NULL".to_string(),
                          rusqlite::types::ValueRef::Integer(i) => i.to_string(),
                          rusqlite::types::ValueRef::Real(f) => f.to_string(),
                          rusqlite::types::ValueRef::Text(s) => String::from_utf8_lossy(s).to_string(),
                          rusqlite::types::ValueRef::Blob(b) => format!("[blob {} bytes]", b.len()),
                      })
              }).collect()
          }).map_err(|e| e.to_string())?
          .map(|r| r.map_err(|e| e.to_string()).unwrap())
          .collect();

          Ok(format!("{:?}", rows))
      }
  }
  ```

- [ ] **Step 3: 创建 SQLRustGo Runner**
  创建: `tests/differential/sqlrustgo_runner.rs`
  ```rust
  use sqlrustgo::ExecutionEngine;
  use sqlrustgo_storage::MemoryStorage;
  use std::sync::{Arc, RwLock};

  pub struct SqlRustGoRunner {
      engine: ExecutionEngine<MemoryStorage>,
  }

  impl SqlRustGoRunner {
      pub fn new() -> Self {
          let storage = Arc::new(RwLock::new(MemoryStorage::new()));
          Self { engine: ExecutionEngine::new(storage) }
      }

      pub fn run(&mut self, sql: &str) -> Result<String, String> {
          let result = self.engine.execute(sql).map_err(|e| e.to_string())?;
          Ok(format!("{:?}", result.rows))
      }
  }
  ```

- [ ] **Step 4: 创建 SELECT 对拍测试**
  创建: `tests/differential/select_tests.rs`
  ```rust
  mod sqlite_runner;
  mod sqlrustgo_runner;

  use sqlite_runner::SqliteRunner;
  use sqlrustgo_runner::SqlRustGoRunner;

  fn assert_sql_eq(sql: &str) {
      let sqlite = SqliteRunner::new();
      let sqlite_result = sqlite.run(sql);

      let mut mine = SqlRustGoRunner::new();
      let my_result = mine.run(sql);

      if sqlite_result.is_err() && my_result.is_err() {
          return; // both failed, OK
      }
      assert_eq!(sqlite_result, my_result, "SQL mismatch: {}", sql);
  }

  #[test]
  fn test_select_basic() {
      assert_sql_eq("SELECT 1");
      assert_sql_eq("SELECT 1 + 2");
      assert_sql_eq("SELECT NULL");
      assert_sql_eq("SELECT 1 + 2 * 3");
  }

  #[test]
  fn test_select_with_null() {
      assert_sql_eq("SELECT NULL = NULL");
      assert_sql_eq("SELECT NULL IS NULL");
      assert_sql_eq("SELECT NULL IS NOT NULL");
  }

  #[test]
  fn test_select_aggregate() {
      assert_sql_eq("SELECT COUNT(*)");
      assert_sql_eq("SELECT COUNT(NULL)");
      assert_sql_eq("SELECT SUM(1)");
  }
  ```

- [ ] **Step 5: 创建 CREATE/INSERT 对拍测试**
  创建: `tests/differential/ddl_tests.rs`
  ```rust
  #[test]
  fn test_create_and_insert() {
      let sqls = [
          "CREATE TABLE t(a INT)",
          "INSERT INTO t VALUES (1)",
          "SELECT * FROM t",
          "CREATE TABLE t2(a TEXT, b INT)",
          "INSERT INTO t2 VALUES ('hello', 42)",
          "SELECT * FROM t2",
      ];
      for sql in sqls {
          assert_sql_eq(sql);
      }
  }
  ```

### Layer K3: SQL Fuzz（随机输入稳定性）

**技术:** 随机生成 SQL，输入引擎，确保不崩溃。

- [ ] **Step 1: 添加随机依赖**
  修改: `Cargo.toml`
  ```toml
  [dev-dependencies]
  rand = "0.8"
  ```

- [ ] **Step 2: 创建简单 SQL 生成器**
  创建: `tests/fuzz/sql_generator.rs`
  ```rust
  use rand::Rng;

  pub struct SqlGenerator { rng: rand::rngs::ThreadRng }

  impl SqlGenerator {
      pub fn new() -> Self { Self { rng: rand::thread_rng() } }

      pub fn generate(&mut self) -> String {
          match self.rng.gen_range(0..5) {
              0 => "SELECT 1".to_string(),
              1 => "SELECT NULL".to_string(),
              2 => "SELECT 1 + 2".to_string(),
              3 => "SELECT 'hello'".to_string(),
              _ => "SELECT 1 + 2 * 3".to_string(),
          }
      }
  }
  ```

- [ ] **Step 3: 创建 Fuzz 测试**
  创建: `tests/fuzz/sql_fuzz.rs`
  ```rust
  mod sql_generator;

  use sql_generator::SqlGenerator;
  use sqlrustgo_runner::SqlRustGoRunner;

  #[test]
  fn fuzz_sql_stability() {
      let mut runner = SqlRustGoRunner::new();
      let mut generator = SqlGenerator::new();

      for _ in 0..1000 {
          let sql = generator.generate();
          // 只确保不崩溃，不检查结果
          let _ = runner.run(&sql);
      }
  }

  #[test]
  fn fuzz_create_table_stability() {
      let mut runner = SqlRustGoRunner::new();

      let sqls = [
          "CREATE TABLE t(a INT)",
          "CREATE TABLE t(a TEXT)",
          "CREATE TABLE t(a INT, b TEXT)",
          "DROP TABLE IF EXISTS t",
      ];

      for sql in sqls {
          let _ = runner.run(sql);
      }
  }
  ```

---

## Phase 4 验收标准

```bash
# sqllogictest
cargo test --test sql_logic

# differential testing
cargo test --test differential

# fuzz testing
cargo test --test fuzz

# 全部通过
cargo test --all-features
```

---

## 里程碑（含 Phase 4）

| 阶段 | 日期 | 目标 |
|------|------|------|
| Phase 1 | 2026-05-07 | parser ≥85%, mysql-server ≥70%, network ≥50% |
| Phase 2 | 2026-05-14 | storage ≥75%, optimizer ≥85%, 分层标准化 |
| Phase 3 | 2026-05-21 | CI 自动门禁, 全项目 ≥80% |
| **Phase 4** | **2026-05-28** | **sqllogictest + SQLite 对拍 + Fuzz 通过** |

---

## Phase 4 关键文件清单

| 操作 | 文件路径 |
|------|---------|
| 新增 | `tests/sql_logic/mod.rs` |
| 新增 | `tests/sql_logic/basic.slt` |
| 新增 | `tests/sql_logic/join.slt` |
| 新增 | `tests/sql_logic/aggregate.slt` |
| 新增 | `tests/differential/sqlite_runner.rs` |
| 新增 | `tests/differential/sqlrustgo_runner.rs` |
| 新增 | `tests/differential/select_tests.rs` |
| 新增 | `tests/differential/ddl_tests.rs` |
| 新增 | `tests/fuzz/sql_generator.rs` |
| 新增 | `tests/fuzz/sql_fuzz.rs` |
| 修改 | `Cargo.toml` (添加依赖) |

---

## 相关文档

- Issue #213: 测试体系工程化
- Issue #173: Alpha 门禁验收报告
- `docs/releases/v2.8.0/COVERAGE_BASELINE.md`: 覆盖率基线
- `scripts/gate/check_coverage.sh`: 现有覆盖率门禁脚本
