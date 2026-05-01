# SQLRustGo → GMP 合规审核系统底层数据库 - 二次评估报告与开发计划

**报告日期**: 2026-04-12
**项目版本**: develop/v2.5.0 (已合并 PR #1373)
**评估视角**: 独立架构审查 + HermesAgent/ChatGPT 评估意见整合

---

## 一、总体结论（先说结果）

### 1.1 原始报告评价

| 维度 | 原报告评分 | 二次评估 |
|------|-----------|----------|
| SQL 引擎 | 4/5 | **2.5/5** |
| Vector | 3/5 | **2/5** |
| Graph | 2.5/5 | **1.5/5** |
| NL | 2/5 | **1/5** |
| API | 4/5 | **3/5** |
| 生产可用性 | 未评分 | **1.5/5** |

### 1.2 一句话定位

> **SQLRustGo 现在是 研究级数据库原型 + AI-native DB 架构实验平台，不是生产数据库**

### 1.3 但它是正确的演进方向

正确使用方式：

```
OpenClaw (Agent 编排层)
    ↓
HermesAgent (NL 推理层)
    ↓
SQLRustGo (多模型检索数据库 kernel)
    ↓
Hybrid Storage Engine
```

---

## 二、代码库深度审计结果

### 2.1 SQL 引擎真实状态

#### 事务系统 ⚠️ 有模块，未集成

**已实现** (`crates/transaction/`):
- `mvcc.rs`: MVCC 核心类型定义完整 (TxId, Transaction, Snapshot, RowVersion)
- `manager.rs`: TransactionManager 接口定义
- `lock.rs`: 锁管理器接口
- `coordinator.rs`: 分布式事务协调器

**问题**: 这些模块是 **骨架实现**，未与 executor/storage 集成。实际 SQL 执行不使用事务。

**验证**:
```rust
// crates/transaction/src/mvcc.rs - 有完整类型定义
pub struct MvccEngine { ... }
impl MvccEngine {
    pub fn begin_transaction(&mut self) -> TxId { ... }
    pub fn commit_transaction(&mut self, tx_id: TxId) -> Option<u64> { ... }
}
```

但 executor 未调用这些方法。

#### 完整性约束 ❌ 未实现

**FOREIGN KEY**: 仅在 `backup.rs` 和 `bench/examples/` 中作为字符串存在，**parser 和 executor 未实现约束检查**。

```rust
// 执行器中无 FOREIGN KEY 验证逻辑
// crates/executor/src/insert.rs - 直接插入，无约束检查
```

#### JOIN 实现 ⚠️ 基础

| JOIN 类型 | 状态 |
|----------|------|
| INNER JOIN | ✅ 基础实现 |
| LEFT JOIN | ❌ 未实现 |
| RIGHT JOIN | ❌ 未实现 |
| CROSS JOIN | ❌ 未实现 |

### 2.2 WAL 真实状态 ✅ 已实现

**好消息**: WAL 模块已完整实现 (1521 行)

```rust
// crates/storage/src/wal.rs
pub struct WalEntry { ... }
pub struct WalManager { ... }  // 完整的 write-ahead logging
```

**功能**:
- ✅ 事务日志 (Begin/Commit/Rollback)
- ✅ 崩溃恢复
- ✅ Checkpoint 支持
- ✅ 2PC Prepare

**问题**: 需要确认是否在正常 SQL 执行路径中启用。

### 2.3 Vector 引擎状态 ✅ 算法完整，❌ 嵌入伪实现

**已实现** (`crates/vector/`):
- `hnsw.rs`: HNSW 完整实现
- `ivf.rs`: IVF 完整实现
- `ivfpq.rs`: IVFPQ 完整实现
- `flat.rs`: 暴力搜索
- `metrics.rs`: 距离度量 (Cosine/Euclidean/DotProduct/Manhattan)
- `pq.rs`: Product Quantization

**问题**: 嵌入是 Hash-based，不是语义嵌入

```rust
// crates/gmp/src/embedding.rs
pub struct HashEmbeddingModel {
    dim: usize,  // 256 维
    // 基于 token hash 的伪随机嵌入
    // 无法捕获语义相似性
}
```

### 2.4 Graph 引擎状态 ⚠️ 内存图，非持久化

**已实现** (`crates/graph/`):
- Node/Edge/Property 数据模型
- BFS/DFS 遍历
- 邻接索引

**问题**: `InMemoryGraphStore` 纯内存，重启即丢失

```rust
// crates/graph/src/store/graph_store.rs
pub struct InMemoryGraphStore {
    nodes: NodeStore,      // DashMap<NodeId, Node> - 纯内存
    edges: EdgeStore,      // 重启即丢失
}
```

---

## 三、GMP 合规审核系统功能差距分析

### 3.1 必须具备的功能 (P0)

| 功能 | 当前状态 | 差距 | 工作量 |
|------|----------|------|--------|
| **事务 ACID** | 骨架未集成 | CRITICAL | 4-6 周 |
| **WAL 崩溃恢复** | 已实现 | 需启用验证 | 1 周 |
| **Graph 持久化** | 纯内存 | CRITICAL | 4-5 月 |
| **FOREIGN KEY 约束** | 未实现 | CRITICAL | 2 周 |
| **语义嵌入 API** | Hash 伪嵌入 | CRITICAL | 3 周 |

### 3.2 生产就绪功能 (P1)

| 功能 | 当前状态 | 差距 | 工作量 |
|------|----------|------|--------|
| LEFT/RIGHT JOIN | 未实现 | HIGH | 2 周 |
| 子查询优化 | 基础 | HIGH | 2 周 |
| Cypher 图查询语言 | 未实现 | HIGH | 6 周 |
| Cost-based Optimizer | 未实现 | HIGH | 6-10 周 |

### 3.3 高级特性 (P2)

| 功能 | 当前状态 | 差距 | 工作量 |
|------|----------|------|--------|
| 分布式图存储 | 未实现 | MEDIUM | 8 周 |
| 最短路径算法 | 未实现 | MEDIUM | 2 周 |
| 多模态向量 | 未实现 | LOW | 4 周 |
| MVCC 并发控制 | 骨架未集成 | HIGH | 4-6 周 |

---

## 四、GMP 合规审核系统开发计划

### 阶段 1: 核心就绪 (0-6 个月)

#### P0-A: 事务系统集成 (4-6 周)

**目标**: 让 SQL 执行启用事务支持

**任务**:
1. 集成 MvccEngine 到 executor
2. 集成 WalManager 到 storage engine
3. 实现 BEGIN/COMMIT/ROLLBACK SQL 命令
4. 验证崩溃恢复

**验收标准**:
```sql
BEGIN;
INSERT INTO deviation (...) VALUES (...);
COMMIT;
-- 崩溃后数据不丢失
```

#### P0-B: Graph 持久化 (4-5 月)

**目标**: 图数据可持久化存储

**任务**:
1. 实现 DiskGraphStore
2. 实现 Graph WAL
3. 实现节点/边的磁盘索引
4. 支持 restart-safe graph

**架构**:
```rust
pub struct DiskGraphStore {
    node_table: BPlusTree<NodeId, Node>,
    edge_table: BPlusTree<EdgeId, Edge>,
    adjacency_index: AdjacencyIndex,
    wal: WalManager,
}
```

#### P0-C: FOREIGN KEY 约束 (2 周)

**目标**: 启用引用完整性约束

**任务**:
1. Parser 支持 FOREIGN KEY 语法
2. Executor 实现约束检查
3. 级联删除/更新支持

#### P0-D: 语义嵌入 API (3 周)

**目标**: 替换 HashEmbedding 为真实语义嵌入

**任务**:
1. 定义 EmbeddingProvider trait
2. 实现 Ollama embedding provider
3. 实现 OpenAI embedding provider
4. 集成到 vector storage

**接口设计**:
```rust
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Vec<f32>;
    fn embed_batch(&self, texts: &[&str]) -> Vec<Vec<f32>>;
    fn dimension(&self) -> usize;
}
```

### 阶段 2: 生产候选 (6-12 个月)

#### P1-A: JOIN 完整实现 (2 周)

实现 LEFT/RIGHT/CROSS JOIN

#### P1-B: 子查询优化 (2 周)

实现 EXISTS/IN/ANY/ALL

#### P1-C: Cypher 子集 (6 周)

实现基本图查询语言

#### P1-D: Cost-based Optimizer (6-10 周)

实现统计信息和执行计划优化

### 阶段 3: 企业级 (12-24 个月)

#### P2-A: 分布式存储 (8 周)

图分区 + 副本

#### P2-B: Compliance Index Engine (6 周)

法规条款引用关系索引

#### P2-C: Evidence Trace Engine (8 周)

审计证据路径自动生成

---

## 五、GMP 系统推荐部署架构

### 5.1 当前最稳架构 (Phase 1-2)

```
PostgreSQL (主库 - 事务完整)
     ↓
SQLRustGo (检索增强层 - 向量+图)
     ↓
OpenClaw (Agent 层)
     ↓
HermesAgent (推理层)
```

### 5.2 目标架构 (Phase 3)

```
OpenClaw Cluster
     ↓
HermesAgent
     ↓
SQLRustGo Distributed Kernel
     ↓
Hybrid Storage Engine
     ↓
  ├─ SQL Engine (ACID)
  ├─ Vector Engine (HNSW/IVF)
  ├─ Graph Engine (Cypher)
  └─ Compliance Index
```

---

## 六、技术创新点 (可用于论文/项目申报)

### 创新点 1: AI-Native Multi-Model Compliance Database Architecture

区别于传统关系数据库，SQLRustGo 定位为 AI-native kernel。

### 创新点 2: Graph + Vector + SQL Fusion Retrieval Engine

统一检索接口，支持混合查询。

### 创新点 3: Evidence-Traceable Compliance Reasoning Index

自动生成审计证据链。

### 创新点 4: Agent-Driven Database Query Planning

LLM 驱动的执行计划优化。

---

## 七、结论与建议

### 7.1 当前状态

| 维度 | 评分 | 说明 |
|------|------|------|
| SQL 引擎 | 2.5/5 | 基础 SQL 支持，缺事务集成、FK、完整 JOIN |
| Vector 检索 | 2/5 | 算法完整，嵌入伪实现 |
| Graph 存储 | 1.5/5 | 纯内存，无持久化 |
| 生产可用性 | 1.5/5 | 研究原型，不能用于生产 |

### 7.2 建议

**立即行动** (0-3 个月):
1. 集成 MVCC + WAL 到 executor
2. 实现 Graph 持久化
3. 替换语义嵌入

**不要做**:
1. 不要在 SQLRustGo 内部集成 LLM
2. 不要做 "大一统" 系统

**正确路线**:
- SQLRustGo 作为 OpenClaw 的检索后端
- 与 PostgreSQL 配合使用
- 逐步演进为生产级数据库

---

## 八、ISSUE 发布计划

建议发布以下 ISSUE:

1. **[P0] 事务系统集成** - 集成 MVCC + WAL 到 SQL 执行路径
2. **[P0] Graph 持久化** - 实现 DiskGraphStore
3. **[P0] FOREIGN KEY 约束** - parser + executor 实现
4. **[P0] 语义嵌入 API** - 替换 HashEmbedding
5. **[P1] JOIN 完整实现** - LEFT/RIGHT/CROSS JOIN
6. **[P1] Cypher 查询语言** - 图查询子集
7. **[P2] 分布式存储** - ShardGraph/ShardVector
