# C 组基础设施实施计划 — Infrastructure + Architecture Hardening

> **版本**: 1.0
> **日期**: 2026-05-06
> **基于**: v2.9.0 develop/v2.9.0
> **对应 v3.0.0 阶段**: Phase 3 + Phase 4
> **负责 Agent**: deepseek (yinglichina163/sqlrustgo)

---

## 一、总览

C 组负责基础设施完善和架构加固，确保 v3.0.0 达到生产就绪状态。

### 任务清单

| # | 任务 | 阶段 | 工时 | 依赖 |
|---|------|------|------|------|
| I-01 | INFORMATION_SCHEMA | Phase 3 | 3d | 无 |
| I-02 | EXPLAIN ANALYZE | Phase 3 | 3d | PP-01 |
| I-04 | 慢查询日志 | Phase 3 | 1d | 无 |
| I-05 | CI Gate 完善 | Phase 3 | 1d | 无 |
| A-01 | 模块边界审计 | Phase 4 | 2d | 无 |
| A-02 | API 版本化 | Phase 4 | 1d | 无 |
| A-03 | 升级兼容性验证 | Phase 4 | 1d | 无 |
| A-04 | 教学模式保持 | Phase 4 | 1d | 无 |

---

## 二、Phase 3: Infrastructure（2 周）

### I-01: INFORMATION_SCHEMA

**文件**: `crates/catalog/src/information_schema.rs`（新建）

| 工时 | 验收标准 |
|------|----------|
| 3d | `SHOW TABLES`/`SHOW COLUMNS` 等效查询通过 |

#### 实施步骤

**Step 1: TABLES 表（1d）**

```sql
SELECT TABLE_NAME, TABLE_TYPE, ENGINE, TABLE_ROWS
FROM INFORMATION_SCHEMA.TABLES
WHERE TABLE_SCHEMA = 'public';
```

**Step 2: COLUMNS 表（1d）**

```sql
SELECT TABLE_NAME, COLUMN_NAME, DATA_TYPE, IS_NULLABLE
FROM INFORMATION_SCHEMA.COLUMNS
WHERE TABLE_NAME = 'orders';
```

**Step 3: STATISTICS/FILES 表（1d）**

```sql
SELECT * FROM INFORMATION_SCHEMA.STATISTICS WHERE TABLE_NAME = 'lineitem';
SELECT * FROM INFORMATION_SCHEMA.FILES;
```

**验收测试**:

```bash
cargo test --test information_schema
```

---

### I-02: EXPLAIN ANALYZE

**文件**: `crates/executor/src/explain.rs`

| 工时 | 验收标准 |
|------|----------|
| 3d | TPC-H Q1 EXPLAIN 输出包含代价估算和行数预测 |

#### 实施步骤

**Step 1: EXPLAIN 输出格式（1d）**

```sql
EXPLAIN SELECT * FROM orders WHERE o_orderdate >= '1993-01-01';
```

输出格式:
```
Seq Scan on orders (cost=0.00..100.00 rows=1000)
  Filter: o_orderdate >= '1993-01-01'
```

**Step 2: ANALYZE 实际执行（1d）**

```sql
EXPLAIN ANALYZE SELECT * FROM orders WHERE o_orderdate >= '1993-01-01';
```

输出包含:
- 预计行数 vs 实际行数
- 执行时间
- 内存使用

**Step 3: JSON/Tree 格式（1d）**

```sql
EXPLAIN (FORMAT JSON) SELECT * FROM orders WHERE o_orderdate >= '1993-01-01';
```

**验收测试**:

```bash
cargo run --bin bench-cli -- tpch-bench --queries Q1 --explain-analyze
```

---

### I-04: 慢查询日志

**文件**: `crates/server/src/slow_query_log.rs`（新建）

| 工时 | 验收标准 |
|------|----------|
| 1d | 超阈值查询记录到日志文件 |

#### 实施步骤

**Step 1: 阈值配置（0.5d）**

```toml
[logging]
long_query_time = 1000  # ms
slow_query_log = "logs/slow.log"
```

**Step 2: 日志记录（0.5d）**

日志格式（兼容 MySQL）:
```
# Time: 2026-05-06T10:30:00.000Z
# Query_time: 1.234  Lock_time: 0.000 Rows_sent: 100  Rows_examined: 10000
SELECT * FROM orders WHERE o_orderdate >= '1993-01-01';
```

**验收测试**:

```bash
cargo test --test slow_query_log
```

---

### I-05: CI Gate 完善

**文件**: `.gitea/workflows/` 或 `.github/workflows/`

| 工时 | 验收标准 |
|------|----------|
| 1d | TPC-H/Sysbench/覆盖率趋势/MySQL 兼容测试进入 CI |

#### 实施步骤

**Step 1: 新建 CI Workflows（0.5d）**

```yaml
# .gitea/workflows/tpch-gate.yml
name: TPC-H Gate
on: [push, pull_request]
jobs:
  tpch:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run TPC-H SF=0.1
        run: |
          cargo run --bin bench-cli -- tpch-bench --queries all --sf 0.1
      - name: Check 18/22 pass
        run: python3 scripts/ci/check_tpch.py
```

**Step 2: 覆盖率趋势（0.5d）**

```yaml
# .gitea/workflows/coverage-trend.yml
name: Coverage Trend
on:
  schedule:
    - cron: '0 0 * * *'  # Daily
jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - name: Run coverage
        run: cargo llvm-cov --all-features --json
      - name: Upload to trend storage
        run: python3 scripts/ci/coverage_trend.py
```

**验收**: CI 页面白名单有新增 workflow

---

## 三、Phase 4: Architecture Hardening（1 周）

### A-01: 模块边界审计

**文件**: 全局

| 工时 | 验收标准 |
|------|----------|
| 2d | 画出 v3.0.0 模块依赖图，确认无环依赖；删除 ≥10% 不必要公开接口 |

#### 实施步骤

**Step 1: 依赖图生成（0.5d）**

```bash
# 使用 cargo modules 或自定义脚本
cargo modules --deps | dot -Tpng > module_deps.png
```

**Step 2: 环依赖检测（0.5d）**

```bash
# 验证无环
python3 scripts/arch/check_cycles.py
```

**Step 3: 接口精简（1d）**

- `pub` → `pub(crate)` 转换 ≥30 个

**验收**: `cargo doc --no-deps` 通过，无循环依赖警告

---

### A-02: API 版本化

**文件**: 所有对外 API

| 工时 | 验收标准 |
|------|----------|
| 1d | 所有对外 API 标注 `#[deprecated]` 或 `#[since = "3.0.0"]` |

#### 实施步骤

```rust
#[deprecated(since = "3.0.0", note = "Use ExecutionEngine::new_with_config instead")]
pub fn new() -> Self { ... }

#[since = "3.0.0"]
pub fn new_with_config(config: EngineConfig) -> Self { ... }
```

**验收**: `cargo clippy --all-features -- -D warnings` 通过

---

### A-03: 升级兼容性验证

**文件**: `docs/releases/v3.0.0/UPGRADE_GUIDE.md`（新建）

| 工时 | 验收标准 |
|------|----------|
| 1d | 产出 v2.9.0 → v3.0.0 迁移指南 |

#### 迁移指南内容

```markdown
## v2.9.0 → v3.0.0 迁移指南

### 配置变更
| 旧配置 | 新配置 | 迁移方式 |
|--------|--------|----------|
| [vector].hnsw_enable | [vector].index_type = "hnsw" | 自动兼容 |

### SQL 行为变更
| 行为 | v2.9.0 | v3.0.0 | 兼容性 |
|------|--------|--------|--------|
| CTE 语法 | 不支持 | 支持 | 兼容 |

### 存储格式
- 向下兼容：v2.9.0 数据文件可直接在 v3.0.0 使用
```

**验收**: 文档存在且完整

---

### A-04: 教学模式保持

**文件**: `crates/teaching/src/`（如果存在）

| 工时 | 验收标准 |
|------|----------|
| 1d | `SQLRUSTGO_TEACHING_MODE=1` 下运行教学 Lab 12 个实验，全部通过 |

#### 实施步骤

**Step 1: 教学模式验证（0.5d）**

```bash
SQLRUSTGO_TEACHING_MODE=1 cargo test --test teaching_labs
```

**Step 2: 新功能教学适配（0.5d）**

确保 CBO、窗口函数、CTE 在教学模式下可禁用/可见化。

---

## 四、时间线

```
Week 10-11 (Phase 3):
  [I-01 INFORMATION_SCHEMA ████████████████████]
  [I-02 EXPLAIN ANALYZE ████████████████████]
  [I-04 慢查询日志 ██████]
  [I-05 CI Gate ██████]

Week 12 (Phase 4):
  [A-01 模块边界审计 ████████████████████████]
  [A-02 API 版本化 ████████████]
  [A-03 升级兼容性 ████████████]
  [A-04 教学模式 ████████████]
```

---

## 五、验收检查清单

### Phase 3 验收

- [ ] `SHOW TABLES`/`SHOW COLUMNS` 等效查询通过
- [ ] TPC-H Q1 EXPLAIN 输出包含代价估算
- [ ] 慢查询日志正确记录超阈值查询
- [ ] CI 新增 tpch-gate, coverage-trend 等 workflow

### Phase 4 验收

- [ ] 模块依赖图无环依赖
- [ ] 不必要公开接口删除 ≥10%
- [ ] 所有对外 API 有版本标注
- [ ] v2.9.0 → v3.0.0 迁移指南存在且完整
- [ ] 教学模式 12 实验全部通过

---

## 六、风险缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 模块边界审计发现循环依赖 | 低 | 高 | 提前使用 `cargo modules` 自检 |
| CI Gate 超时 | 中 | 低 | 使用 Gitea CI 并行 job |
| 迁移指南遗漏重要变更 | 低 | 中 | 使用 checklist 逐项核对 |

---

## 七、依赖关系

```
I-02 (EXPLAIN) 依赖 PP-01 (CBO) — Phase 1 完成
A-03 (迁移指南) 依赖 Phase 3 所有任务完成
A-04 (教学模式) 依赖 F-01~F-05 完成
```

---

*版本 1.0 | 2026-05-06*
*C 组基础设施任务全部分配。关键路径 3 周。*
