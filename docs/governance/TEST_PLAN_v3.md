# SQLRustGo v3.0.0 测试计划（综合版）

> **版本**: 1.0
> **日期**: 2026-05-08
> **综合自**: v1.9.0 / v2.7.0 / v2.9.0 / v3.0.0 历史测试文档
> **目的**: 建立分阶段、分层级的测试体系，重点解决"覆盖率指标好看但不知道该补什么"的问题

---

## 一、核心问题诊断

### 1.1 历史测试计划的问题

| 问题 | 现象 | 根因 |
|------|------|------|
| **覆盖率高但不知道该补什么** | workspace 72% 但 optimizer 0%、planner <1% | 只看整体数字，未分析模块级差距 |
| **测试通过但场景遗漏** | 294 tests 集中在 5-6 个文件，边缘路径未测 | 测试用例聚集在主路径 |
| **B2 全量测试超时** | `cargo test --all-features` 挂起 | heavy crates（mysql-server/bench/distributed）拖慢 |
| **覆盖率工具不统一** | 有用 tarpaulin 有用 llvm-cov | 未固定主工具 |
| **测试计划与门禁脱节** | TEST_PLAN 有 160 项但门禁只检查 15 项 | 未建立映射关系 |

### 1.2 本文档解决方案

```
┌─────────────────────────────────────────────────────┐
│              覆盖率弱项分析引擎                       │
│                                                     │
│  覆盖率数据 → 差距计算 → 风险评级 → 补测优先级        │
│                                                     │
│  不是"覆盖率低了多写测试"                             │
│  而是"这个模块缺了什么场景，为什么危险"                │
└─────────────────────────────────────────────────────┘
```

---

## 二、测试分层体系（L0~L3）

### 2.1 四层测试定义

| 层级 | 名称 | 执行时机 | 目标时间 | 覆盖范围 |
|------|------|----------|----------|----------|
| **L0** | 冒烟 | 每次 commit/PR | <5min | 编译+格式+核心路径 |
| **L1** | 模块回归 | 每次合并前 | <30min | 核心 crate lib tests |
| **L2** | 集成回归 | 发布前/大PR | <90min | 跨模块+协议+SQL Corpus |
| **L3** | 深度验证 | 夜间/发布前 | >90min | 性能+稳定性+混沌 |

### 2.2 L0 冒烟测试

**目的**: 快速判断分支是否可用，5 分钟内完成

```
L0 检查项
=============
[ ] cargo build --release --workspace          # 编译成功
[ ] cargo fmt --all -- --check                 # 格式正确
[ ] cargo clippy --all-features -- -D warnings # 无警告
[ ] cargo test -p sqlrustgo-types --lib        # types 冒烟
[ ] cargo test -p sqlrustgo-parser --lib      # parser 冒烟
```

**命令**: `bash scripts/gate/check_l0_smoke.sh`

### 2.3 L1 模块回归

**目的**: 验证核心 crate 行为正确，30 分钟内完成

```
L1 范围（排除 heavy crates）
==============================
核心 crates:
  - sqlrustgo-types
  - sqlrustgo-parser
  - sqlrustgo-planner
  - sqlrustgo-optimizer
  - sqlrustgo-executor (lib only, skip integration tests)
  - sqlrustgo-storage (lib only, skip integration tests)
  - sqlrustgo-transaction
  - sqlrustgo-catalog

禁止在 L1 中运行:
  - sqlrustgo-mysql-server (需要网络监听)
  - sqlrustgo-bench (heavy 性能测试)
  - sqlrustgo-distributed (需要多节点)
  - sqlancer (fuzz 测试)
  - sqlrustgo-server (需要 TCP 端口)

命令:
  cargo test -p sqlrustgo-types \
             -p sqlrustgo-parser \
             -p sqlrustgo-planner \
             -p sqlrustgo-optimizer \
             -p sqlrustgo-executor \
             -p sqlrustgo-storage \
             -p sqlrustgo-transaction \
             -p sqlrustgo-catalog \
             --lib \
             -- --test-threads=8
```

### 2.4 L2 集成回归

**目的**: 跨模块、跨引擎、跨协议验证，90 分钟内完成

```
L2 范围
========
集成测试（按重要性排序）:
  1. sqlrustgo-sql-corpus (SQL 兼容性)
  2. wal_integration_test (WAL + 崩溃恢复)
  3. crash_recovery_test (崩溃恢复)
  4. concurrency_stress_test (并发压力)
  5. transaction 相关 integration tests
  6. optimizer/planner integration tests

命令:
  cargo test -p sqlrustgo-sql-corpus -- --test-threads=4
  cargo test --test wal_integration_test
  cargo test --test crash_recovery_test
  cargo test --test concurrency_stress_test
  cargo test -p sqlrustgo-optimizer --lib -- --test-threads=4
  cargo test -p sqlrustgo-planner --lib -- --test-threads=4
```

### 2.5 L3 深度验证

**目的**: 发布级稳定性与性能，夜间/发布前执行

```
L3 范围
========
性能测试:
  - TPC-H SF=0.1 / SF=1 (bash scripts/gate/check_tpch.sh)
  - Sysbench (bash scripts/gate/check_sysbench.sh)
  - Point/UPDATE/DELETE QPS (cargo bench)

稳定性测试:
  - long_run_stability_test (长时间运行)
  - network_tcp_smoke_test (网络稳定性)

覆盖率:
  - cargo llvm-cov --all-features --json (完整覆盖率)

混沌工程:
  - crash_injection_test (崩溃注入)
  - chaos_test (混沌测试)

命令:
  bash scripts/gate/check_tpch.sh sf=1
  bash scripts/gate/check_sysbench.sh
  cargo llvm-cov --all-features --json --output-path /tmp/cov-v300.json
  cargo test --test long_run_stability_test
  cargo test --test chaos_test
```

---

## 三、阶段门禁与测试映射

### 3.1 四阶段门禁定义

```
Alpha (A-Gate) → Beta (B-Gate) → RC (R-Gate) → GA (G-Gate)
   功能开发完成    功能冻结        发布候选       正式发布
```

### 3.2 各阶段测试要求

| 阶段 | L0 | L1 | L2 | L3 | 覆盖率目标 | 特殊要求 |
|------|----|----|----|----|-----------|----------|
| **Alpha** | ✅ 每次提交 | ✅ 每次PR | 关键路径 | — | ≥50% | 性能基线建立 |
| **Beta** | ✅ | ✅ | ✅ 完整 | 关键项 | ≥75% | TPC-H SF=0.1 22/22 |
| **RC** | ✅ | ✅ | ✅ | ✅ 完整 | ≥85% | TPC-H SF=1 22/22 |
| **GA** | ✅ | ✅ | ✅ | ✅ | ≥85% | Point Select ≥10K QPS |

### 3.3 Alpha 阶段测试要求

**入口条件**: Phase 0-4 开发任务完成

```
A-Gate 检查清单
================
A1. 编译: cargo build --all-features --workspace          [无错误]
A2. 单元测试: cargo test --all-features --workspace        [≥80%]
A3. Clippy: cargo clippy --all-features -- -D warnings     [零警告]
A4. 格式: cargo fmt --all -- --check                       [无错误]
A5. 文档链接: bash scripts/gate/check_docs_links.sh        [无死链]
A6. 覆盖率: cargo llvm-cov --all-features                   [≥50%]
A7. 安全: cargo audit                                      [无高危]

覆盖率模块级要求 (Alpha):
  executor  ≥45%    optimizer ≥40%    storage ≥15%
  catalog   ≥50%    parser    ≥50%     整体   ≥50%
```

### 3.4 Beta 阶段测试要求

**入口条件**: A-Gate 已通过 + TPC-H SF=0.1 可运行

```
B-Gate 检查清单
================
B1. 编译: cargo build --release --workspace
B2. 全量测试: cargo test --all-features        [≥90%]  ⚠️ 可能超时
B3. Clippy: cargo clippy --all-features -- -D warnings
B4. 格式: cargo fmt --all -- --check
B5. 覆盖率: cargo llvm-cov --all-features       [≥75%]
B6. 安全: cargo audit
B7. 文档链接: bash scripts/gate/check_docs_links.sh
B8. TPC-H: scripts/gate/check_tpch.sh sf=0.1   [22/22]
B9. SQL Corpus: cargo test -p sqlrustgo-sql-corpus [≥85%]

稳定性测试 (B-S*):
  B-S1. concurrency_stress_test   [PASS]
  B-S2. crash_recovery_test       [PASS]
  B-S3. long_run_stability_test   [PASS]
  B-S4. wal_integration_test       [PASS]
  B-S5. network_tcp_smoke_test     [PASS]
  B-S10. test_sql_corpus_operations [≥20%]  [信息项]

覆盖率模块级要求 (Beta):
  executor  ≥60%    optimizer ≥50%    storage ≥20%
  catalog   ≥60%    parser    ≥60%    整体   ≥75%

⚠️ B2 注意事项:
  如果 cargo test --all-features 超时（>300s），
  使用 L1 核心 crate 测试替代，但必须在 CI 中记录
  实际运行的命令和超时原因。
```

### 3.5 RC 阶段测试要求

**入口条件**: B-Gate 已通过 + TPC-H SF=1 可运行

```
R-Gate 检查清单
================
R1. 编译: cargo build --release --workspace
R2. 全量测试: cargo test --all-features          [100%]
R3. Clippy: cargo clippy --all-features -- -D warnings
R4. 格式: cargo fmt --all -- --check
R5. 覆盖率: cargo llvm-cov --all-features         [≥85%]
R6. 安全: cargo audit
R7. 文档: check_docs_links.sh + 必选文档存在性 + 版本一致性
R8. SQL Compat: cargo test -p sqlrustgo-sql-corpus  [≥95%]
R9. TPC-H SF=1: scripts/gate/check_tpch.sh sf=1   [22/22]
R10. 性能基线: cargo bench + check_perf_baseline.sh [退化≤5%]
R11. Sysbench: scripts/gate/check_sysbench.sh
R12. MySQL Protocol: mysql:5.7 容器握手测试

覆盖率模块级要求 (RC):
  executor  ≥75%    optimizer ≥70%    storage ≥40%
  catalog   ≥70%    parser    ≥70%    整体   ≥85%
```

### 3.6 GA 阶段测试要求

**入口条件**: R-Gate 已通过 + QPS 达标

```
G-Gate 检查清单
================
G1.  编译: cargo build --release --workspace
G2.  全量测试: cargo test --all-features          [100%]
G3.  Clippy: cargo clippy --all-features -- -D warnings
G4.  格式: cargo fmt --all -- --check
G5.  覆盖率: cargo llvm-cov --all-features         [≥85%]
G6.  安全: cargo audit
G7.  Point SELECT QPS: cargo bench -- point_select [≥10,000 ops/s]
G8.  UPDATE QPS: cargo bench -- update_simple       [≥5,000 ops/s]
G9.  DELETE QPS: cargo bench -- delete_simple       [≥2,000 ops/s]
G10. TPC-H SF=1: scripts/gate/check_tpch.sh sf=1  [22/22]
G11. SQL Corpus: cargo test -p sqlrustgo-sql-corpus [≥98%]
G12. B-S 稳定性: B-S1~B-S5 全部 PASS
G13. MySQL Protocol: mysql:5.7 容器握手测试

覆盖率模块级要求 (GA):
  executor  ≥80%    optimizer ≥70%    storage ≥40%
  catalog   ≥75%    parser    ≥80%    整体   ≥85%
```

---

## 四、覆盖率弱项分析引擎

### 4.1 覆盖率分析框架

**不是"覆盖率低了多写测试"，而是"这个模块缺了什么场景，为什么危险"**

```
覆盖率分析流程
================

Step 1: 采集模块级覆盖率
  cargo llvm-cov --all-features --json --output-path /tmp/cov.json

Step 2: 分解模块差距
  ┌──────────────────────────────────────────────┐
  │ 模块        当前   目标   差距   风险评级    │
  │ optimizer   0%    50%    50%    🔴 CRITICAL  │
  │ planner     1%    50%    49%    🔴 CRITICAL  │
  │ storage     1%    40%    39%    🟡 HIGH      │
  │ types      30%    50%    20%    🟡 HIGH      │
  │ parser     20%    50%    30%    🟡 HIGH      │
  │ executor   72%    75%     3%    🟢 LOW       │
  └──────────────────────────────────────────────┘

Step 3: 风险评级标准
  🔴 CRITICAL: 模块差距 >30%，且该模块是核心路径
  🟡 HIGH:     模块差距 >20%，或非核心路径但影响大
  🟢 LOW:      模块差距 <10%

Step 4: 生成补测建议（不是"增加覆盖率"，而是"补什么场景"）
  optimizer 0% → 缺少物理计划生成测试、代价模型测试
  planner 1%  → 缺少逻辑计划到物理计划的转换测试
  storage 1%  → 缺少 B+Tree 并发测试、页缓存淘汰测试
```

### 4.2 模块级覆盖率目标（Beta 阶段）

| 模块 | 当前覆盖 | Beta 目标 | 差距 | 风险 | 缺失场景分析 |
|------|---------|-----------|------|------|-------------|
| optimizer | 0% | 50% | 50% | 🔴 CRITICAL | 无任何单元测试，物理计划生成完全未测 |
| planner | <1% | 50% | ~49% | 🔴 CRITICAL | 逻辑计划到物理计划转换未测 |
| storage | 1% | 20% | 19% | 🟡 HIGH | B+Tree 并发、页缓存淘汰路径未测 |
| types | 4% | 50% | 46% | 🔴 CRITICAL | Value 类型转换、边界条件未测 |
| parser | 20% | 60% | 40% | 🟡 HIGH | DDL 语法、复杂表达式未测 |
| executor | 72% | 75% | 3% | 🟢 LOW | 边缘路径（NULL、错误处理）可小幅补测 |
| transaction | ~0% | 50% | 50% | 🔴 CRITICAL | MVCC 可见性、死锁检测、WAL 路径未测 |
| catalog | 1% | 60% | 59% | 🔴 CRITICAL | Schema 操作、DDL 事务性未测 |

> **数据来源**: v2.9.0 TEST_STATUS_20260503.md §七

### 4.3 补测优先级决策树

```
发现模块覆盖率低于目标
         │
         ├── 差距 <5% 且模块非核心路径
         │         → 记录为 LOW，忽略，下版本处理
         │
         ├── 差距 5-15% 或模块是核心路径
         │         → 标记为 HIGH，优先补测
         │         → 补测场景：边界条件 + 错误路径
         │
         ├── 差距 >15% 且模块是核心路径
         │         → 标记为 CRITICAL，必须补测
         │         → 补测场景：主路径 + 边缘路径
         │
         └── 模块 = 0%（完全无测试）
                  → 标记为 CRITICAL，紧急补测
                  → 第一步：建立最小测试集（10-20 tests）
```

### 4.4 补测场景分析模板

每个模块补测前，必须回答以下问题：

```markdown
## {模块名} 补测场景分析

### 当前状态
- 当前覆盖率: {X}%
- 目标覆盖率: {Y}%
- 差距: {Z}%

### 核心路径覆盖情况
| 路径 | 是否已测 | 测试文件 |
|------|---------|---------|
| {路径1} | ✅/❌ | {文件} |
| {路径2} | ✅/❌ | {文件} |

### 缺失的高风险场景
| 场景 | 风险描述 | 可能导致的问题 | 补测建议 |
|------|---------|---------------|---------|
| {场景1} | {描述} | {后果} | {测试用例} |

### 补测计划
- 目标覆盖率: {Z}%
- 需要新增测试: {N} 个
- 预估时间: {X} 人天
```

### 4.5 覆盖率工具统一规定

**必须使用 `cargo-llvm-cov` 作为主工具**，禁止混用 tarpaulin：

```bash
# 安装（CI 环境）
cargo install cargo-llvm-cov

# 完整覆盖率
cargo llvm-cov --all-features --json --output-path /tmp/cov.json

# 模块级覆盖率
cargo llvm-cov -p sqlrustgo-optimizer --all-features --json

# HTML 报告
cargo llvm-cov --all-features --html --open
```

**输出格式规范**：
```
cargo llvm-cov --all-features --json 输出字段要求:
  - total_line_pct: 整体行覆盖率
  - per_crate: 每个 crate 的覆盖率
```

---

## 五、SQL Corpus 测试增强

### 5.1 SQL Corpus 测试结构

```
sqlrustgo-sql-corpus 测试分类
==============================
test_sql_corpus_all        # 全部 SQL 语法（485 cases）
test_sql_corpus_operations # DML 操作（55 cases） ← 当前最弱
test_sql_corpus_aggregates # 聚合函数
test_sql_corpus_joins     # JOIN 变体
test_sql_corpus_subqueries # 子查询
```

### 5.2 operations 类别弱项分析

**当前状态**: 20% (11/55) 通过，44 个语法不支持

| 不支持类别 | 数量 | 影响 |
|-----------|------|------|
| BACKUP | 1 | 运维能力缺失 |
| SAVEPOINT | 1 | 事务 Savepoint 缺失 |
| SET TRANSACTION ISOLATION LEVEL | 1 | 隔离级别控制缺失 |
| LIMIT/OFFSET | 1 | 分页缺失 |
| TRUNCATE | 1 | 表清空缺失 |
| REPLACE | 1 | UPSERT 缺失 |
| SHOW | ~10 | 元信息查询缺失 |
| EXPLAIN ANALYZE | 1 | 执行计划分析缺失 |
| TEMPORARY TABLE | 1 | 临时表缺失 |
| ALTER TABLE INPLACE | ~5 | 在线 DDL 缺失 |
| BATCH INSERT | ~10 | 批量插入缺失 |

**补测优先级**：

```
第一优先（影响大，实现难）:
  1. LIMIT/OFFSET → 分页是基本功能
  2. ALTER TABLE INPLACE → 在线 DDL 是 v3.0 亮点功能
  3. EXPLAIN ANALYZE → 性能调优必备

第二优先（影响中，实现中）:
  4. REPLACE → UPSERT 是常见模式
  5. SHOW → 元信息查询
  6. TEMPORARY TABLE → 临时表

第三优先（影响小，可豁免）:
  7. BACKUP → 运维功能，可延后
  8. SAVEPOINT → 高级事务功能
```

### 5.3 SQL Corpus 覆盖率追踪

```bash
# 追踪每个类别的通过率趋势
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_all -- --nocapture 2>&1 | \
  grep -E "test_sql_corpus|PASSED|FAILED|passed|failed"

# 分类输出
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_aggregates -- --nocapture
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_joins -- --nocapture
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_subqueries -- --nocapture
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_operations -- --nocapture
```

---

## 六、TPC-H 测试增强

### 6.1 TPC-H 测试分级

| 级别 | 触发条件 | 数据规模 | 执行时间 | 要求 |
|------|----------|----------|----------|------|
| **SF=0.1** | 每次 PR / Beta 门禁 | ~100KB | <30s | 22/22 |
| **SF=1** | RC 门禁 / 发布前 | ~1MB | <5min | 22/22 无 OOM |
| **SF=10** | 夜间 / 性能对比 | ~10MB | <30min | 22/22 记录 p99 |
| **SF=100** | 季度 / 硬件对比 | ~100MB | >1h | 可选 |

### 6.2 TPC-H 当前问题

**SF=0.1 Q1 性能退化**: Q1=49997ms > 10000ms（阈值）

```
Q1 慢查询分析
=============
症状: Q1 (Pricing Summary Report) 执行时间 49997ms，超过 10s 阈值

可能原因:
  1. 全表扫描 + 聚合计算效率低
  2. 内存中数据布局不利于 SIMD
  3. 日期过滤条件未使用索引

建议:
  1. 检查 lineitem 表是否有索引（l_shipdate）
  2. 分析 EXPLAIN 输出看是否走了索引扫描
  3. 对比 v2.9.0 的 Q1 执行时间，确认是回归还是已知问题

下一步:
  bash scripts/gate/check_tpch.sh sf=0.1 --verbose
  # 查看每个 query 的执行计划和耗时
```

### 6.3 TPC-H 测试增强建议

```markdown
## TPC-H 补测规划

### 22 个 Query 的风险分级

🔴 高风险（容易 OOM / 超时）:
  - Q17: Large Volume Orders (子查询)
  - Q18: Large Customer Orders (JOIN + GROUP BY)
  - Q20: Potential Part Promotion (LIKE 优化)
  - Q21: Suppliers Who Kept Orders Waiting (NOT IN)

🟡 中风险（执行时间不稳定）:
  - Q1: Pricing Summary Report
  - Q3: Shipping Priority
  - Q6: Forecast Revenue Change
  - Q9: Product Type Profit

🟢 低风险（稳定通过）:
  - Q2, Q4, Q5, Q7, Q8, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q19, Q22
```

---

## 七、测试执行节奏

### 7.1 日常开发节奏

```
日常开发（每次 commit/PR）
==========================
L0 冒烟:  <5min   自动在 CI 执行
L1 模块:  <30min  PR 合并前必须通过

合并前检查清单:
  [ ] L0 冒烟全部 PASS
  [ ] L1 核心 crate 测试 PASS
  [ ] 无新增 Clippy 警告
  [ ] 格式检查 PASS
```

### 7.2 发布节奏

```
Beta 发布前
============
执行: L0 + L1 + L2 + 关键 L3
验证: B-Gate 全部 PASS
时间: <3h

RC 发布前
=========
执行: L0 + L1 + L2 + 完整 L3
验证: R-Gate 全部 PASS
时间: <6h + 夜间 L3

GA 发布前
=========
执行: 完整 L0~L3 + 混沌测试
验证: G-Gate 全部 PASS
时间: <24h（含 L3 夜间）
```

### 7.3 覆盖率趋势追踪

```bash
# 覆盖率趋势记录（每次 L3 执行时）
timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
cargo llvm-cov --all-features --json --output-path /tmp/cov.json

# 提取数据
total_cov=$(cat /tmp/cov.json | jq '.tables.total[0].line_percent // 0')
optimizer_cov=$(cat /tmp/cov.json | jq '[.tables.crates[] | select(.name=="sqlrustgo-optimizer")] | .[0].line_percent // 0')
planner_cov=$(cat /tmp/cov.json | jq '[.tables.crates[] | select(.name=="sqlrustgo-planner")] | .[0].line_percent // 0')

echo "$timestamp,$total_cov,$optimizer_cov,$planner_cov" >> docs/releases/v3.0.0/coverage_trend.csv
```

---

## 八、测试增强规划能力

### 8.1 核心原则：测试是为了发现 Bug，不是为了覆盖率数字

```
❌ 错误认知:
   "我们的覆盖率是 65%，需要提升到 80%，多写测试"

✅ 正确认知:
   "optimizer 0% 覆盖率意味着物理计划生成完全没有测试，
    这是核心路径，一旦有 Bug 影响所有查询，必须立即补测"
```

### 8.2 弱项识别算法

```python
# 伪代码：弱项识别算法

def identify_weak_points(coverage_data, gate_requirements):
    """
    输入:
      coverage_data: {module: {current: float, target: float, is_core_path: bool}}
      gate_requirements: {module: {beta: float, rc: float, ga: float}}

    输出:
      weak_points: [{module, gap, risk, suggested_scenarios}]
    """

    weak_points = []

    for module, data in coverage_data.items():
        current = data['current']
        target_rc = gate_requirements[module]['rc']
        gap = target_rc - current
        is_core = data['is_core_path']

        # 风险评级
        if gap > 30 and is_core:
            risk = "CRITICAL"
        elif gap > 20 or is_core:
            risk = "HIGH"
        else:
            risk = "LOW"

        # 生成补测建议
        if risk in ("CRITICAL", "HIGH"):
            weak_points.append({
                'module': module,
                'current': current,
                'target': target_rc,
                'gap': gap,
                'risk': risk,
                'scenarios': generate_scenarios(module, gap)
            })

    # 按风险排序
    weak_points.sort(key=lambda x: RISK_ORDER[x['risk']])

    return weak_points

def generate_scenarios(module, gap):
    """
    根据模块和差距生成推荐补测场景
    不是随机增加测试，而是针对缺失的场景
    """
    scenarios = {
        'optimizer': [
            '物理计划生成: IndexScan vs SeqScan 选择',
            'JOIN 顺序优化',
            '代价模型计算',
            'LIMIT 下推优化'
        ],
        'planner': [
            '逻辑计划解析',
            '子查询去嵌套',
            'GROUP BY 聚合折叠'
        ],
        'storage': [
            'B+Tree 并发插入',
            '页缓存淘汰策略',
            'WAL 写入路径',
            '崩溃恢复完整性'
        ],
        'transaction': [
            'MVCC 可见性判断',
            '死锁检测与回滚',
            'Savepoint 嵌套',
            'RC/RR 隔离级别差异'
        ]
    }
    return scenarios.get(module, ['通用边界条件测试', '错误路径测试'])
```

### 8.3 补测效果验证

每次补测后，必须验证：

```
补测效果验证清单
================
[ ] 新增测试发现了 Bug？→ 如果是，证明补测有效
[ ] 新增测试只是增加了覆盖率数字？→ 如果是，重新评估补测方向
[ ] 模块覆盖率提升了吗？→ 提升了多少
[ ] 新增测试是否引入了新的不稳定性？→ 多次运行是否稳定
[ ] 新增测试的维护成本？→ 是否过于复杂/脆弱
```

---

## 九、测试体系改进建议

### 9.1 立即可改进项

| 改进项 | 当前问题 | 改进方案 | 优先级 |
|--------|----------|----------|--------|
| **B2 命令优化** | `cargo test --all-features` 超时挂起 | 排除 heavy crates，改为 L1 核心测试 | P0 |
| **optimizer 0%** | 完全没有单元测试 | 建立最小测试集（10-20 tests） | P0 |
| **planner <1%** | 物理计划转换未测 | 建立物理计划生成测试 | P1 |
| **覆盖率工具统一** | tarpaulin/llvm-cov 混用 | 统一使用 cargo-llvm-cov | P1 |
| **B5 超时** | `cargo llvm-cov --all-features` 超时 | 只测核心 crates | P2 |

### 9.2 中期改进项

| 改进项 | 目标 | 时间 |
|--------|------|------|
| SQL Fuzz | 发现边缘 SQL 解析 Bug | 1-2 周 |
| SQLite Differential Testing | 对比 SQL 语义正确性 | 1 周 |
| sqllogictest 集成 | 标准化 SQL 测试 | 1 周 |
| 覆盖率趋势可视化 | 追踪每个版本的覆盖率变化 | 2-3 天 |

### 9.3 长期改进项

| 改进项 | 目标 | 时间 |
|--------|------|------|
| Property-Based Testing | 用 rand 测试 SQL 执行器 | 2-3 周 |
| Model-Based Testing | TLA+ 模型驱动测试生成 | 持续 |
| Chaos Engineering | 自动化混沌测试 | 持续 |

---

## 十、相关文档

| 文档 | 作用 |
|------|------|
| `gate_spec_v300.md` | 门禁定义 SSOT |
| `gate_lifecycle_tracking.md` | 门禁失败项追踪 |
| `governance_self_improvement.md` | 治理自我进化机制 |
| `scripts/gate/check_beta_v300.sh` | Beta 门禁脚本 |
| `scripts/gate/check_tpch.sh` | TPC-H 测试脚本 |
| `scripts/gate/check_l0_smoke.sh` | L0 冒烟脚本（待创建） |

---

## 十一、附录：v2.9.0 覆盖率数据（基线）

> 来源: `docs/releases/v2.9.0/TEST_STATUS_20260503.md`

```
模块         Lines 覆盖   覆盖率    函数覆盖    风险
-------------------------------------------------------
executor     1436/6450   72.65%    78.99%      🟢 LOW
parser       3412/7723   20.85%    17.84%      🟡 HIGH
types         556/1137    4.30%     2.63%      🔴 CRITICAL
planner      1297/2607    0.99%     1.59%      🔴 CRITICAL
optimizer       0/6298    0.00%     0.00%      🔴 CRITICAL
storage      5054/10178   1.37%     2.21%      🔴 CRITICAL
catalog      2615/5280    1.88%     2.07%      🔴 CRITICAL
security     1609/3218    0.00%     0.00%      🔴 CRITICAL
gmp          3970/7940    0.00%     0.00%      🔴 CRITICAL
graph        3373/6746    0.00%     0.00%      🔴 CRITICAL
-------------------------------------------------------
Workspace    2185/3242   30.17%    26.16%      🔴
```

**关键发现**:
- executor 72% 是假象：294 tests 集中在 5-6 个文件
- optimizer 0%：完全没有单元测试
- planner <1%：物理计划生成没有测试
- storage 1%：虽然数据量看起来大，但占总行数只有 1.37%

---

*本文档综合自 v1.9.0 / v2.7.0 / v2.9.0 / v3.0.0 历史测试文档*
*最后更新: 2026-05-08*
