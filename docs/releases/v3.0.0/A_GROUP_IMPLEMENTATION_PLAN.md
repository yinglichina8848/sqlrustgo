# A 组性能核心实施计划 — Performance Pocket v1

> **版本**: 1.0
> **日期**: 2026-05-06
> **基于**: v2.9.0 develop/v2.9.0
> **对应 v3.0.0 阶段**: Phase 0 + Phase 1 + Phase 3 (I-03)
> **负责 Agent**: opencode (openheart/sqlrustgo)

---

## 一、总览

A 组负责性能核心任务，是 v3.0.0 "**从能用到好用**" 的关键驱动力。

### 量化目标

| 指标 | v2.9.0 基线 | Phase 1 目标 | 最终目标 |
|------|-------------|--------------|----------|
| Point Select QPS | ~2,000 | ≥8,000 | ≥20,000 |
| UPDATE QPS | ~950 | ≥4,000 | ≥10,000 |
| DELETE QPS | ~206 | ≥1,000 | ≥5,000 |
| TPC-H SF=0.1 Q1 | 基线待测 | 减少 ≥30% | 减少 ≥50% |
| 批量写入吞吐 | 基线待测 | 提升 ≥2x | 提升 ≥3x |

### 关键路径

```
D-01 CBO 实现 ──→ PP-01 CBO 完善 ──→ PP-02 连接池 ──→ I-03 SSL/TLS
                    │
                    └─→ PP-03 查询缓存 ──→ PP-04 Group Commit
                    │
                    └─→ PP-05 批量 Insert
```

---

## 二、Phase 0 债务偿还（2 周）

### D-01: CBO 规则实现

**源码证据**: `optimizer/src/rules.rs:67,95,122` — 3 个 `// TODO: Implement`

| 工时 | 验收标准 |
|------|----------|
| 4d | 3 个 TODO 清零，规则返回 true（有实际变换） |

**实施步骤**:

1. **规则 1 (rules.rs:67)** — Predicate Pushdown
   - 文件: `crates/optimizer/src/rules.rs`
   - 实现: 将 WHERE 条件下推到存储层
   - 验证: `cargo test --package sqlrustgo-optimizer -- predicate`

2. **规则 2 (rules.rs:95)** — Projection Pruning
   - 实现: 只读取需要的列
   - 验证: `cargo test --package sqlrustgo-optimizer -- projection`

3. **规则 3 (rules.rs:122)** — Constant Folding
   - 实现: 编译期计算常量表达式
   - 验证: `cargo test --package sqlrustgo-optimizer -- constant`

**依赖**: 无

---

## 三、Phase 1: Performance Pocket v1（4 周）

### PP-01: CBO 完善（Index Selection + Join Reordering + 代价模型）

**文件**: `crates/optimizer/src/`

| 工时 | 验收标准 |
|------|----------|
| 7d | TPC-H Q1 执行时间减少 ≥50% |

#### 实施步骤

**Step 1: Index Selection（2d）**

```bash
# 验收测试
cargo run --bin bench-cli -- tpch-bench --queries Q2,Q16 --iterations 3 --sf 0.1
```

- 基于代价选择最优索引
- 多索引时合并结果
- 文件: `crates/optimizer/src/cost.rs`

**Step 2: Join Reordering（3d）**

- 实现多表 JOIN 顺序优化
- 基于卡片式估计（cardinality estimation）
- 文件: `crates/planner/src/join_order.rs`

**Step 3: 代价模型校准（2d）**

- I/O 代价 vs CPU 代价权重调优
- 行数估计准确性提升
- 验证: TPC-H Q1-Q6 全部通过

---

### PP-02: 连接池

**文件**: `crates/server/src/` 或 `crates/mysql-server/src/`

| 工时 | 验收标准 |
|------|----------|
| 5d | 100 并发 point_select 无连接失败 |

#### 实施步骤

**Step 1: 连接复用（2d）**

- 实现连接复用，避免每次查询重新打开文件
- 文件: `crates/server/src/connection_pool.rs`（新建）

**Step 2: 配置项（1d）**

```toml
[performance]
max_connections = 256
connection_pool_size = 16
```

**Step 3: 压力测试（2d）**

```bash
# 验收: 100 并发连接
cargo test --test connection_pool_stress
```

---

### PP-03: 查询缓存

**文件**: `crates/executor/src/query_cache.rs`

| 工时 | 验收标准 |
|------|----------|
| 3d | INSERT 后同条件 SELECT 返回新数据 |

#### 实施步骤

**Step 1: DML 自动失效（1.5d）**

- DML 后自动失效相关缓存条目
- LRU 缓存容量可配置

**Step 2: 缓存统计（0.5d）**

- 缓存命中/未命中计数
- 缓存大小监控

**Step 3: 集成测试（1d）**

```bash
# 验收: INSERT 后 SELECT 返回新数据
cargo test --test query_cache_invalidation
```

---

### PP-04: Group Commit

**文件**: `crates/transaction/src/wal.rs` 或 `crates/storage/src/wal.rs`

| 工时 | 验收标准 |
|------|----------|
| 3d | 批量写入 QPS 提升 ≥2x |

#### 实施步骤

**Step 1: WAL 批量 fsync（2d）**

- WAL 批量 fsync，减少 sync 次数
- 配置项: `group_commit_batch_size`, `group_commit_timeout_ms`

**Step 2: 性能验证（1d）**

```bash
# 验收: 批量写入 QPS 提升 ≥2x
cargo run --bin bench-cli -- tpch-bench --queries Q1 --iterations 10 --sf 0.1
```

---

### PP-05: 批量 Insert 优化

**文件**: `crates/executor/src/insert.rs`

| 工时 | 验收标准 |
|------|----------|
| 2d | 1000 行批量插入 <100ms |

#### 实施步骤

**Step 1: Batch Insert 实现（1d）**

- 批量 Insert 优化
- 减少单行插入的函数调用开销

**Step 2: 性能验证（1d）**

```bash
# 验收: 1000 行批量插入 <100ms
cargo test --test bulk_insert_perf
```

---

## 四、Phase 3: Infrastructure（I-03 SSL/TLS）

### I-03: SSL/TLS 支持

**文件**: `crates/network/src/tls.rs`

| 工时 | 验收标准 |
|------|----------|
| 2d | MySQL 客户端 `--ssl-mode=REQUIRED` 握手成功 |

#### 实施步骤

**Step 1: TLS 握手（1d）**

- 依赖 PP-02 连接池
- 实现 TLS 1.2/1.3 支持
- 文件: `crates/network/src/tls.rs`

**Step 2: 集成测试（1d）**

```bash
# 验收: MySQL 客户端 SSL 连接
mysql -h 127.0.0.1 -P 3306 --ssl-mode=REQUIRED -u root -e "SELECT 1"
```

---

## 五、时间线

```
Week 1-2 (Phase 0):  D-01 CBO 规则实现
                      [D-01################███████████████]

Week 3-4 (Phase 1):  PP-01 CBO 完善 (Index Selection + Join Reordering)
                      [PP-01 Index█████████]
                      [PP-01 Join███████]

Week 5 (Phase 1):     PP-02 连接池 + PP-03 查询缓存
                      [PP-02 连接池█████████████]
                      [PP-03 查询缓存███████████]

Week 6 (Phase 1):     PP-04 Group Commit + PP-05 批量 Insert
                      [PP-04 GroupCommit████████████]
                      [PP-05 批量Insert██████]

Week 10-11 (Phase 3): I-03 SSL/TLS
                      [I-03 SSL/TLS████████████████]
```

---

## 六、验收检查清单

### Phase 0 验收

- [ ] `cargo clippy --all-features -- -D warnings` 通过
- [ ] CBO 3 个 TODO 清零
- [ ] 每个规则有实际变换输出

### Phase 1 验收

- [ ] TPC-H Q1 执行时间减少 ≥50%
- [ ] Point Select QPS ≥8,000
- [ ] 100 并发连接无失败
- [ ] INSERT 后 SELECT 返回新数据
- [ ] 批量写入 QPS 提升 ≥2x
- [ ] 1000 行批量插入 <100ms

### Phase 3 验收

- [ ] MySQL 客户端 `--ssl-mode=REQUIRED` 握手成功

---

## 七、风险缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| CBO 实现后 QPS 未达 8K | 中 | 高 | 预留 1 周调优 buffer |
| 连接池与现有网络栈冲突 | 低 | 中 | 先做模块化接口设计 |
| Group Commit 引入数据风险 | 低 | 高 | 先写崩溃恢复测试 |

---

## 八、依赖关系

```
PP-01 (CBO) 依赖 D-01 (Phase 0)
I-03 (SSL) 依赖 PP-02 (连接池)
PP-03 (缓存) 依赖 PP-01 (CBO)
PP-04 (Group Commit) 依赖 PP-01 (CBO)
PP-05 (批量 Insert) 无依赖
```

---

*版本 1.0 | 2026-05-06*
*A 组性能核心任务全部分配。关键路径 8 周。*
