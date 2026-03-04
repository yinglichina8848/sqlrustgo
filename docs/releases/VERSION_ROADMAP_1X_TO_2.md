# SQLRustGo 1.x → 2.0 版本演进规划

> **版本**: 1.x 路线图
> **制定日期**: 2026-03-05
> **目标**: 从单机内核演进到可嵌入分布式数据库

---

## 版本总览

| 版本 | 代号 | 核心目标 | 状态 |
|------|------|----------|------|
| **1.2** | Architecture Stabilization | 架构接口化 | 🔄 当前 |
| **1.3** | Node Abstraction | 节点抽象 | 📋 规划 |
| **1.4** | WAL Abstraction | WAL 抽象 | 📋 规划 |
| **1.5** | Replicable Log | 可复制日志 | 📋 规划 |
| **2.0** | Distributed Core | 分布式数据库内核 | 📋 规划 |

---

## 一、1.2 架构接口化（详细版）

> 已在 [ARCHITECTURE_REFACTORING_PLAN.md](./ARCHITECTURE_REFACTORING_PLAN.md) 详细规划

### 核心目标

把"单机数据库内核"改造成"可扩展数据库平台内核"

### 必须完成的 6 大重构

1. Optimizer 框架层
2. Statistics 子系统
3. Catalog 系统
4. 执行层接口化
5. 错误域重构
6. Pipeline 执行模型

### 目标目录结构

```
src/
├── optimizer/           # 优化器框架
├── catalog/            # 元数据管理
├── statistics/         # 统计信息系统
├── error/              # 错误域分离
└── execution/
    └── pipeline.rs     # Pipeline 模型
```

### 完成标志

- [ ] Optimizer trait + Rule + CostModel
- [ ] StatisticsProvider trait
- [ ] Catalog trait + 可持久化
- [ ] Executor trait
- [ ] StorageEngine trait
- [ ] 错误域分离

---

## 二、1.3 节点抽象（初步规划）

> **代号**: Node Abstraction
> **目标**: 引入节点概念，为分布式做准备

### 2.1 核心目标

1. 定义节点模型
2. 引入节点角色（Leader/Follower）
3. 实现节点通信基础

### 2.2 架构设计

#### 节点模型

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

#[derive(Clone, Debug)]
pub enum NodeRole {
    Leader,
    Follower,
    Candidate,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub role: NodeRole,
    pub address: SocketAddr,
    pub is_local: bool,
}
```

#### 节点注册表

```rust
trait NodeRegistry: Send + Sync {
    fn register(&self, node: Node) -> Result<()>;
    fn unregister(&self, node_id: NodeId) -> Result<()>;
    fn get_node(&self, node_id: NodeId) -> Option<Node>;
    fn list_nodes(&self) -> Vec<Node>;
}
```

### 2.3 任务分解

| 任务 | 说明 | 优先级 |
|------|------|--------|
| N-001 | NodeId/Node/NodeRole 定义 | P0 |
| N-002 | NodeRegistry trait | P0 |
| N-003 | LocalNodeRegistry 实现 | P0 |
| N-004 | 节点健康检查 | P1 |
| N-005 | 节点心跳机制 | P1 |

### 2.4 目录结构

```
src/
├── cluster/             # ⭐ 新增
│   ├── mod.rs
│   ├── node.rs         # 节点模型
│   ├── registry.rs     # 节点注册
│   └── health.rs       # 健康检查
```

### 2.5 依赖

- 依赖 1.2 完成的 Executor trait
- 依赖 1.2 完成的 StorageEngine trait

---

## 三、1.4 WAL 抽象（初步规划）

> **代号**: WAL Abstraction
> **目标**: 统一日志接口，支持多种日志实现

### 3.1 核心目标

1. 抽象 WAL 接口
2. 支持多种日志格式
3. 支持日志压缩和归档

### 3.2 架构设计

#### WAL Trait

```rust
pub trait WAL: Send + Sync {
    fn append(&self, entry: WALEntry) -> Result<Lsn>;
    fn read(&self, lsn: Lsn) -> Result<WALEntry>;
    fn flush(&self) -> Result<()>;
    fn truncate(&self, lsn: Lsn) -> Result<()>;
}

pub struct Lsn(pub u64);

pub enum WALEntry {
    Data(Vec<u8>),
    Commit { tx_id: u64, lsn: Lsn },
    Abort { tx_id: u64 },
}
```

#### WAL 实现

```rust
pub struct FileWAL {
    path: PathBuf,
    // ...
}

impl WAL for FileWAL {
    // 文件 WAL 实现
}
```

### 3.3 任务分解

| 任务 | 说明 | 优先级 |
|------|------|--------|
| W-001 | WAL trait 定义 | P0 |
| W-002 | FileWAL 实现 | P0 |
| W-003 | WAL 格式化 | P1 |
| W-004 | WAL 压缩 | P2 |
| W-005 | WAL 归档 | P2 |

### 3.4 目录结构

```
src/
├── wal/                # ⭐ 新增
│   ├── mod.rs
│   ├── trait.rs        # WAL trait
│   ├── file_wal.rs     # 文件 WAL
│   ├── entry.rs        # 日志条目
│   └── error.rs
```

### 3.5 依赖

- 依赖 1.2 完成的 error 分类
- 依赖 1.2 完成的 StorageEngine trait

---

## 四、1.5 可复制日志（初步规划）

> **代号**: Replicable Log
> **目标**: 日志复制接口，为 Raft 做准备

### 4.1 核心目标

1. 抽象日志复制接口
2. 支持日志同步/异步复制
3. 定义复制协议

### 4.2 架构设计

#### Log Replication Trait

```rust
pub trait LogReplication: Send + Sync {
    fn replicate(&self, entry: WALEntry) -> Result<ReplicateResult>;
    fn get_commit_index(&self) -> Lsn;
    fn get_last_index(&self) -> Lsn;
}

pub struct ReplicateResult {
    pub success: bool,
    pub commit_index: Lsn,
    pub node_id: NodeId,
}

pub enum ReplicationMode {
    Sync,    // 同步复制
    Async,   // 异步复制
    Quorum,  // 多数派复制
}
```

#### 日志领导者

```rust
trait LogLeader: Send + Sync {
    fn replicate_to(&self, node: NodeId, entry: WALEntry) -> Result<()>;
    fn wait_for_acks(&self, quorum: usize, timeout: Duration) -> Result<Lsn>;
}
```

### 4.3 任务分解

| 任务 | 说明 | 优先级 |
|------|------|--------|
| L-001 | LogReplication trait | P0 |
| L-002 | ReplicationMode 枚举 | P0 |
| L-003 | SimpleReplicator 实现 | P1 |
| L-004 | 多数派复制实现 | P1 |

### 4.4 目录结构

```
src/
├── replication/        # ⭐ 新增
│   ├── mod.rs
│   ├── trait.rs       # 复制 trait
│   ├── simple.rs      # 简单复制
│   └── quorum.rs      # 多数派复制
```

### 4.5 依赖

- 依赖 1.3 完成的 Node 模型
- 依赖 1.4 完成的 WAL trait

---

## 五、版本依赖关系

```
1.2 架构接口化
  │
  ├──→ Optimizer trait
  ├──→ Statistics trait
  ├──→ Catalog trait
  ├──→ Executor trait
  ├──→ StorageEngine trait
  └──→ Error 分类
           │
           ↓
1.3 节点抽象
  │
  ├──→ NodeId/Node/NodeRole
  ├──→ NodeRegistry
  └──→ 节点健康检查
           │
           ↓
1.4 WAL 抽象
  │
  ├──→ WAL trait
  ├──→ FileWAL 实现
  └──→ 日志格式化
           │
           ↓
1.5 可复制日志
  │
  ├──→ LogReplication trait
  ├──→ ReplicationMode
  └──→ SimpleReplicator
           │
           ↓
2.0 分布式数据库内核
    │
    ├──→ Raft 共识
    ├──→ 分布式执行
    └──→ 全局优化
```

---

## 六、五年演进路线图（简化版）

### Year 1: 单机内核成熟

| 版本 | 时间 | 目标 |
|------|------|------|
| 1.2 | Month 1-2 | 架构接口化 |
| 1.3 | Month 3-4 | 节点抽象 |
| 1.4 | Month 5-6 | WAL 抽象 |
| 1.5 | Month 7-8 | 可复制日志 |

**成果**: 可嵌入数据库内核，具备分布式基础

### Year 2: 分布式数据库

| 版本 | 目标 |
|------|------|
| 2.0 | 分布式数据库内核 |

---

## 七、安全线检查

| 版本 | 必须完成的接口 | 2.0 必需 |
|------|---------------|----------|
| 1.2 | Executor trait | ✅ |
| 1.2 | StorageEngine trait | ✅ |
| 1.2 | StatisticsProvider | ✅ |
| 1.2 | Catalog | ✅ |
| 1.3 | Node 模型 | ✅ |
| 1.4 | WAL trait | ✅ |
| 1.5 | LogReplication | ✅ |

---

## 八、下一步

1. **完成 1.2 开发**（当前任务）
2. **启动 1.3 详细规划**（节点抽象）
3. 依次推进 1.4、1.5

---

## 相关文档

- [ARCHITECTURE_REFACTORING_PLAN.md](./ARCHITECTURE_REFACTORING_PLAN.md) - 1.2 详细规划
- [VERSION_PLAN.md](./VERSION_PLAN.md) - 1.2 版本计划

---

*制定日期: 2026-03-05*
