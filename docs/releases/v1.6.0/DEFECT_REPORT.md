# SQLRustGo v1.6.0 缺陷检查报告

> **报告日期**: 2026-03-19
> **版本**: v1.6.0 Production Preview
> **状态**: 需要重新设计

---

## 一、执行摘要

本报告对 SQLRustGo v1.6.0 的设计文档与实际代码实现进行了全面审查，发现以下关键问题：

1. **文档状态过时**: 多个已完成的功能在文档中仍标记为待开发
2. **死锁检测未实现**: 设计文档描述的功能代码中不存在
3. **连接池架构缺陷**: 实现为 Session Pool 而非 Connection Pool，无法支持真正的并发压测

---

## 二、代码实现状态 vs 文档状态

### 2.1 功能完成状态对比

| 功能 | 设计文档状态 | 实际代码状态 | PR | 问题级别 |
|------|--------------|--------------|-----|----------|
| T-04 行级锁 | ✅ #633 | ✅ 已合并 | #633 | 无 |
| T-05 死锁检测 | ⏳ | ❌ 未实现 | #628 | **严重** |
| P-01 查询缓存 | ✅ #627 | ✅ 已合并 | #627 | 无 |
| P-02 连接池 | ⏳ | ⚠️ 有缺陷 | #636 | **严重** |
| P-03 TPC-H | ⏳ | ✅ 已合并 | #637 | **文档过时** |
| D-01 DATE | ✅ #624 | ✅ 已合并 | #624 | 无 |
| D-02 TIMESTAMP | ✅ #634 | ✅ 已合并 | #634 | 无 |

---

## 三、详细缺陷分析

### 3.1 T-05 死锁检测：完全未实现

#### 问题描述

设计文档 (`ARCHITECTURE_DESIGN.md`) 详细描述了死锁检测的架构：
- Wait-For Graph 数据结构
- DFS 环检测算法
- Victim 选择策略（等待时间最长）
- 异步回滚机制

#### 实际代码状态

```bash
$ grep -r "deadlock" crates/
crates/transaction/src/lock.rs:255:            LockError::Deadlock => write!(f, "deadlock detected"),
```

**发现**:
- 只有 `LockError::Deadlock` 枚举变体
- 无任何 `DeadlockDetector` 结构
- 无 Wait-For Graph 实现
- 无 DFS 检测逻辑

#### 影响

- 死锁无法自动检测
- 事务可能无限等待
- 并发安全性无法保证

---

### 3.2 P-02 连接池：架构设计缺陷

#### 设计预期

根据 `v1.6.0_design.md`:
```rust
struct ConnectionPool {
    max_size: usize,
    semaphore: Semaphore,
    connections: Mutex<Vec<Connection>>,
}
```

目标：支持 50+ 并发连接

#### 实际实现

`crates/server/src/connection_pool.rs`:
```rust
pub struct PooledSession {
    pub executor: LocalExecutor<'static>,
    pub storage: Arc<MemoryStorage>,  // 内存存储！
    pub transaction_id: Option<u64>,
    in_use: bool,
}
```

#### 问题分析

| 设计预期 | 实际实现 | 问题 |
|----------|----------|------|
| 真实数据库连接 | 内存存储 | 无法持久化数据 |
| 多连接复用 | 每 session 独立存储 | 无法共享数据 |
| 50+ 并发压测 | 单机内存模式 | 无法验证并发能力 |

#### 影响

- 无法进行真正的并发压测
- 与设计目标"可压测"不符
- Session 间数据隔离，无法测试跨连接事务

---

### 3.3 P-03 TPC-H：文档过时

#### 设计预期

`v1.6.0_design.md` 描述:
- Q1/Q6 执行
- SQLite 对比 (`benches/sqlite_compare.rs`)
- Benchmark 报告生成

#### 实际代码

- ✅ `benches/tpch_bench.rs` 存在
- ✅ Q1/Q6 执行 (PR #637)
- ❌ `sqlite_compare.rs` 不存在
- ❌ 无 SQLite 对比功能

---

## 四、文档一致性问题

### 4.1 状态标记不一致

| 文档 | P-02 连接池 | P-03 TPC-H |
|------|-------------|-------------|
| ARCHITECTURE_DESIGN.md | ⏳ | ⏳ |
| v1.6.0_task_checklist.md | ⏳ | ⏳ |
| v1.6.0_gate_check_spec.md | ⏳ | ⏳ |
| 实际代码 | ✅ 已合并 | ✅ 已合并 |

### 4.2 文件路径不一致

| 功能 | 文档路径 | 实际路径 |
|------|----------|----------|
| 连接池 | crates/pool/connection_pool.rs | crates/server/src/connection_pool.rs |
| TPC-H | benches/tpch_bench.rs | benches/tpch_bench.rs ✅ |

---

## 五、重新设计建议

### 5.1 T-05 死锁检测重新设计

#### 目标
实现完整的死锁检测和处理机制

#### 建议架构

```rust
// 1. Wait-For Graph
pub struct WaitForGraph {
    edges: HashMap<TxId, HashSet<TxId>>,  // tx -> blocked by
    wait_start: HashMap<TxId, Instant>,
}

// 2. 死锁检测器
pub struct DeadlockDetector {
    graph: RwLock<WaitForGraph>,
    max_depth: usize,  // 限制检测深度
}

// 3. 检测策略
enum DetectionStrategy {
    OnContention,      // 阻塞时检测（当前设计）
    Periodic,          // 定时检测
    Hybrid,            // 混合
}
```

#### 实施计划

| 阶段 | 任务 | 估计工作量 |
|------|------|-----------|
| 1 | Wait-For Graph 数据结构 | 100 行 |
| 2 | DFS 环检测算法 | 150 行 |
| 3 | Victim 选择策略 | 50 行 |
| 4 | 异步回滚机制 | 100 行 |
| 5 | 集成测试 | 100 行 |

---

### 5.2 P-02 连接池重新设计

#### 目标
实现真正的数据库连接池，支持 50+ 并发压测

#### 问题根因
当前设计混淆了 "Session Pool" 和 "Connection Pool"

#### 建议架构

```rust
// 方案 A: 基于单一物理连接的多会话
pub struct ConnectionPool {
    config: PoolConfig,
    semaphore: Arc<Semaphore>,
    connection: RwLock<Option<Arc<Database>>>,  // 单连接
    sessions: Mutex<HashMap<SessionId, SessionState>>,
}

// 方案 B: 独立连接池（推荐用于未来分布式）
pub struct ConnectionPool {
    config: PoolConfig,
    connections: Vec<PooledConnection>,  // 多连接
    semaphore: Arc<Semaphore>,
}
```

#### 最小可行实现（v1.6.1）

```rust
// 基于单一 MemoryStorage 的改进
pub struct SimpleConnectionPool {
    storage: Arc<RwLock<MemoryStorage>>,  // 共享存储
    semaphore: Arc<Semaphore>,
    max_connections: usize,
}

impl SimpleConnectionPool {
    pub async fn acquire(&self) -> PooledConnection {
        self.semaphore.acquire().await.unwrap();
        PooledConnection {
            storage: Arc::clone(&self.storage),
        }
    }
}
```

#### 实施计划

| 阶段 | 任务 | 估计工作量 |
|------|------|-----------|
| 1 | 重构为共享存储的连接池 | 150 行 |
| 2 | 添加并发测试 | 100 行 |
| 3 | 性能基准验证 | 50 行 |

---

### 5.3 文档更新

#### 建议

1. **统一状态源**: 使用 GitHub Projects 或单一 STATUS.md
2. **自动化同步**: PR 合并时自动更新文档状态
3. **路径验证**: 文档中的路径应与实际代码一致

---

## 六、影响评估

### 6.1 v1.6.0 发布质量

| 指标 | 当前状态 | 目标 | 风险 |
|------|----------|------|------|
| 死锁检测 | 未实现 | 可用 | 高 |
| 连接池 | 有缺陷 | 可压测 | 高 |
| 文档一致性 | 差 | 好 | 中 |

### 6.2 建议行动

1. **立即**: 更新文档状态，反映实际代码
2. **高优先级**: 实现 T-05 死锁检测
3. **高优先级**: 修复 P-02 连接池架构
4. **中优先级**: 添加 SQLite 对比功能

---

## 七、附录

### A. 代码位置汇总

| 功能 | 实际代码路径 |
|------|-------------|
| 行级锁 | crates/transaction/src/lock.rs |
| 查询缓存 | crates/executor/src/query_cache.rs |
| 连接池 | crates/server/src/connection_pool.rs |
| TPC-H | benches/tpch_bench.rs |
| DATE/TIMESTAMP | crates/types/src/value.rs |

### B. 合并的 PR

| PR | 功能 | 日期 |
|----|------|------|
| #633 | T-04 行级锁 | 2026-03 |
| #627 | P-01 查询缓存 | 2026-03 |
| #636 | P-02 连接池 | 2026-03 |
| #637 | P-03 TPC-H | 2026-03 |
| #624 | D-01 DATE | 2026-03 |
| #634 | D-02 TIMESTAMP | 2026-03 |

---

*本报告由 AI 辅助分析生成*
*分析日期: 2026-03-19*
