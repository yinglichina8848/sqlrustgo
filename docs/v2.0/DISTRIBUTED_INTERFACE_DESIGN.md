# 3.0 分布式接口预留设计

> **版本**: 3.0 预留设计  
> **状态**: 规划中  
> **目标**: 2.0 单机 → 3.0 可安全升级为分布式执行  
> **更新日期**: 2026-03-05

---

## 目录

1. [设计目标](#一设计目标)
2. [参考项目](#二参考项目)
3. [ExecutionContext 设计](#三executioncontext-设计)
4. [Executor Trait](#四executor-trait)
5. [Plan Fragment](#五plan-fragment)
6. [DataExchange 接口](#六dataexchange-接口)
7. [Storage Trait](#七storage-trait)
8. [分布式架构预览](#八分布式架构预览)

---

## 一、设计目标

### 1.1 核心目标

> **2.0 单机 → 3.0 可安全升级为分布式执行**

### 1.2 设计原则

| 原则 | 说明 |
|------|------|
| 接口稳定 | 2.0 接口在 3.0 不变 |
| 向后兼容 | 2.0 代码在 3.0 可运行 |
| 渐进扩展 | 通过新增接口支持分布式 |
| 最小侵入 | 分布式逻辑不污染核心代码 |

---

## 二、参考项目

### 2.1 TiDB

| 设计点 | 参考内容 |
|--------|----------|
| 分布式执行 | Coprocessor 模型 |
| 数据交换 | Chunk 流式传输 |
| 调度模型 | DAG 调度 |

### 2.2 Cockroach Labs

| 设计点 | 参考内容 |
|--------|----------|
| 分布式事务 | MVCC + 2PC |
| 数据复制 | Raft 共识 |
| 节点通信 | gRPC |

### 2.3 Apache Spark

| 设计点 | 参考内容 |
|--------|----------|
| 执行模型 | Stage/Task 划分 |
| 数据交换 | Shuffle 机制 |
| 容错机制 | Lineage 重算 |

---

## 三、ExecutionContext 设计

### 3.1 NodeId 定义

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn coordinator() -> Self {
        Self(0)
    }
    
    pub fn is_coordinator(&self) -> bool {
        self.0 == 0
    }
}
```

### 3.2 ExecutionMode

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    Local,
    Distributed,
}
```

### 3.3 ExecutionContext Trait

```rust
pub trait ExecutionContext: Send + Sync {
    fn node_id(&self) -> NodeId;
    
    fn mode(&self) -> ExecutionMode;
    
    fn is_distributed(&self) -> bool {
        matches!(self.mode(), ExecutionMode::Distributed)
    }
    
    fn is_coordinator(&self) -> bool {
        self.node_id().is_coordinator()
    }
    
    fn data_exchange(&self) -> Option<&dyn DataExchange> {
        None
    }
}
```

### 3.4 LocalExecutionContext (2.0 实现)

```rust
pub struct LocalExecutionContext {
    node_id: NodeId,
}

impl LocalExecutionContext {
    pub fn new() -> Self {
        Self {
            node_id: NodeId::coordinator(),
        }
    }
    
    pub fn with_node_id(node_id: NodeId) -> Self {
        Self { node_id }
    }
}

impl Default for LocalExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionContext for LocalExecutionContext {
    fn node_id(&self) -> NodeId {
        self.node_id.clone()
    }
    
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Local
    }
}
```

### 3.5 DistributedExecutionContext (3.0 实现)

```rust
pub struct DistributedExecutionContext {
    node_id: NodeId,
    data_exchange: Arc<dyn DataExchange>,
    cluster_info: ClusterInfo,
}

impl DistributedExecutionContext {
    pub fn new(
        node_id: NodeId,
        data_exchange: Arc<dyn DataExchange>,
        cluster_info: ClusterInfo,
    ) -> Self {
        Self {
            node_id,
            data_exchange,
            cluster_info,
        }
    }
}

impl ExecutionContext for DistributedExecutionContext {
    fn node_id(&self) -> NodeId {
        self.node_id.clone()
    }
    
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Distributed
    }
    
    fn data_exchange(&self) -> Option<&dyn DataExchange> {
        Some(self.data_exchange.as_ref())
    }
}
```

---

## 四、Executor Trait

### 4.1 Trait 定义

```rust
pub trait Executor: Send {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    
    fn next(&mut self, ctx: &dyn ExecutionContext) 
        -> Result<Option<RecordBatch>>;
    
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    
    fn schema(&self) -> &Schema;
    
    fn statistics(&self) -> ExecutorStatistics {
        ExecutorStatistics::default()
    }
}
```

### 4.2 设计要点

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Executor 设计要点                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. 所有算子必须接收 ExecutionContext                                      │
│      └── 支持本地/分布式执行切换                                             │
│                                                                              │
│   2. 返回 RecordBatch 而非单行                                              │
│      └── 支持向量化执行                                                      │
│      └── 支持网络批量传输                                                   │
│                                                                              │
│   3. open/next/close 生命周期                                               │
│      └── open: 初始化资源                                                   │
│      └── next: 拉取数据                                                     │
│      └── close: 释放资源                                                    │
│                                                                              │
│   4. 统计信息收集                                                            │
│      └── 支持性能分析                                                       │
│      └── 支持 CBO 优化                                                      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.3 示例实现

```rust
pub struct ScanExecutor {
    table: String,
    storage: Arc<dyn StorageEngine>,
    schema: Schema,
    scanner: Option<Box<dyn Iterator<Item = RecordBatch>>>,
}

impl ScanExecutor {
    pub fn new(table: String, storage: Arc<dyn StorageEngine>) -> Result<Self> {
        let schema = storage.schema(&table)?;
        Ok(Self {
            table,
            storage,
            schema,
            scanner: None,
        })
    }
}

impl Executor for ScanExecutor {
    fn open(&mut self, _ctx: &dyn ExecutionContext) -> Result<()> {
        self.scanner = Some(self.storage.scan(&self.table)?);
        Ok(())
    }
    
    fn next(&mut self, _ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>> {
        self.scanner
            .as_mut()
            .ok_or_else(|| SqlError::ExecutorNotOpen)?
            .next()
            .transpose()
    }
    
    fn close(&mut self, _ctx: &dyn ExecutionContext) -> Result<()> {
        self.scanner = None;
        Ok(())
    }
    
    fn schema(&self) -> &Schema {
        &self.schema
    }
}
```

---

## 五、Plan Fragment

### 5.1 定义

```rust
#[derive(Clone, Debug)]
pub struct PlanFragment {
    pub fragment_id: u32,
    pub root: Arc<dyn PhysicalPlan>,
    pub partition: Option<PartitionInfo>,
}

#[derive(Clone, Debug)]
pub struct PartitionInfo {
    pub partition_count: usize,
    pub partition_column: Option<String>,
}
```

### 5.2 Fragment 生成

```rust
pub struct PlanFragmenter {
    next_fragment_id: u32,
}

impl PlanFragmenter {
    pub fn new() -> Self {
        Self { next_fragment_id: 0 }
    }
    
    pub fn fragment(&mut self, plan: Arc<dyn PhysicalPlan>) -> Vec<PlanFragment> {
        let mut fragments = Vec::new();
        self.collect_fragments(plan, &mut fragments);
        fragments
    }
    
    fn collect_fragments(
        &mut self, 
        plan: Arc<dyn PhysicalPlan>, 
        fragments: &mut Vec<PlanFragment>
    ) {
        // 在 Exchange 算子处切分
        if plan.as_exchange().is_some() {
            fragments.push(PlanFragment {
                fragment_id: self.next_fragment_id,
                root: plan,
                partition: None,
            });
            self.next_fragment_id += 1;
        } else {
            for child in plan.children() {
                self.collect_fragments(child, fragments);
            }
        }
    }
}
```

### 5.3 流程示意

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Plan Fragment 流程                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   协调节点                                                                   │
│        │                                                                     │
│        ▼                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     LogicalPlan                                      │   │
│   │  Project                                                             │   │
│   │    └── Filter                                                        │   │
│   │          └── Join                                                    │   │
│   │                ├── Scan (table_a)                                    │   │
│   │                └── Scan (table_b)                                    │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│        │                                                                     │
│        ▼                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     PhysicalPlan                                     │   │
│   │  ProjectExec                                                         │   │
│   │    └── FilterExec                                                    │   │
│   │          └── HashJoinExec                                            │   │
│   │                ├── ScanExec (table_a)                                │   │
│   │                └── Exchange ──────────────────────┐ (切分点)         │   │
│   │                                                    │                  │   │
│   └────────────────────────────────────────────────────┼─────────────────┘   │
│                                                        │                     │
│                                                        ▼                     │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     Fragment 0 (Worker Node 1)                       │   │
│   │  ScanExec (table_b, partition 0)                                    │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│        │                                                                     │
│        ▼                                                                     │
│   发送 Fragment 给工作节点                                                   │
│        │                                                                     │
│        ▼                                                                     │
│   执行并返回结果                                                             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 六、DataExchange 接口

### 6.1 Trait 定义

```rust
pub trait DataExchange: Send + Sync {
    fn send_batch(&self, target: NodeId, batch: RecordBatch) -> Result<()>;
    
    fn receive_batch(&self, source: NodeId) -> Result<Option<RecordBatch>>;
    
    fn send_eof(&self, target: NodeId) -> Result<()>;
    
    fn close(&self) -> Result<()>;
}
```

### 6.2 NoopExchange (2.0 实现)

```rust
pub struct NoopExchange;

impl NoopExchange {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopExchange {
    fn default() -> Self {
        Self::new()
    }
}

impl DataExchange for NoopExchange {
    fn send_batch(&self, _target: NodeId, _batch: RecordBatch) -> Result<()> {
        Ok(())
    }
    
    fn receive_batch(&self, _source: NodeId) -> Result<Option<RecordBatch>> {
        Ok(None)
    }
    
    fn send_eof(&self, _target: NodeId) -> Result<()> {
        Ok(())
    }
    
    fn close(&self) -> Result<()> {
        Ok(())
    }
}
```

### 6.3 NetworkExchange (3.0 实现)

```rust
pub struct NetworkExchange {
    node_id: NodeId,
    connections: HashMap<NodeId, ExchangeConnection>,
}

struct ExchangeConnection {
    sender: mpsc::Sender<ExchangeMessage>,
    receiver: mpsc::Receiver<ExchangeMessage>,
}

enum ExchangeMessage {
    Batch(RecordBatch),
    Eof,
    Error(String),
}

impl DataExchange for NetworkExchange {
    fn send_batch(&self, target: NodeId, batch: RecordBatch) -> Result<()> {
        let conn = self.connections.get(&target)
            .ok_or_else(|| SqlError::NodeNotFound(target))?;
        conn.sender.send(ExchangeMessage::Batch(batch))
            .map_err(|e| SqlError::ExchangeError(e.to_string()))?;
        Ok(())
    }
    
    fn receive_batch(&self, source: NodeId) -> Result<Option<RecordBatch>> {
        let conn = self.connections.get(&source)
            .ok_or_else(|| SqlError::NodeNotFound(source))?;
        match conn.receiver.recv() {
            Ok(ExchangeMessage::Batch(batch)) => Ok(Some(batch)),
            Ok(ExchangeMessage::Eof) => Ok(None),
            Ok(ExchangeMessage::Error(e)) => Err(SqlError::ExchangeError(e)),
            Err(e) => Err(SqlError::ExchangeError(e.to_string())),
        }
    }
    
    fn send_eof(&self, target: NodeId) -> Result<()> {
        let conn = self.connections.get(&target)
            .ok_or_else(|| SqlError::NodeNotFound(target))?;
        conn.sender.send(ExchangeMessage::Eof)
            .map_err(|e| SqlError::ExchangeError(e.to_string()))?;
        Ok(())
    }
    
    fn close(&self) -> Result<()> {
        for conn in self.connections.values() {
            let _ = conn.sender.send(ExchangeMessage::Eof);
        }
        Ok(())
    }
}
```

---

## 七、Storage Trait

### 7.1 Trait 定义

```rust
pub trait StorageEngine: Send + Sync {
    fn schema(&self, table: &str) -> Result<Schema>;
    
    fn scan(&self, table: &str) -> Result<Box<dyn Iterator<Item = RecordBatch>>>;
    
    fn scan_partition(
        &self, 
        table: &str, 
        partition: usize
    ) -> Result<Box<dyn Iterator<Item = RecordBatch>>> {
        if partition == 0 {
            self.scan(table)
        } else {
            Err(SqlError::PartitionNotFound(partition))
        }
    }
    
    fn insert(&self, table: &str, batch: RecordBatch) -> Result<()>;
    
    fn partition_count(&self, table: &str) -> Result<usize> {
        Ok(1)
    }
}
```

### 7.2 实现类型

| 实现 | 版本 | 说明 |
|------|------|------|
| MemoryStorage | 2.0 | 内存存储 |
| FileStorage | 2.0 | 文件存储 |
| RocksDBStorage | 2.0+ | RocksDB 存储 |
| RemoteStorage | 3.0 | 远程存储 |
| DistributedKVStorage | 3.0 | 分布式 KV 存储 |

---

## 八、分布式架构预览

### 8.1 架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          3.0 分布式架构                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                       Coordinator Node                               │   │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │   │
│   │  │   Parser    │  │  Optimizer  │  │  Scheduler  │                  │   │
│   │  └─────────────┘  └─────────────┘  └─────────────┘                  │   │
│   │                         │                                            │   │
│   │                         ▼                                            │   │
│   │              ┌─────────────────────┐                                 │   │
│   │              │   Plan Fragmenter   │                                 │   │
│   │              └─────────────────────┘                                 │   │
│   │                         │                                            │   │
│   │                         ▼                                            │   │
│   │              ┌─────────────────────┐                                 │   │
│   │              │   Task Dispatcher   │                                 │   │
│   │              └─────────────────────┘                                 │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                    ┌───────────────┼───────────────┐                        │
│                    ▼               ▼               ▼                        │
│   ┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐  │
│   │    Worker Node 1    │ │    Worker Node 2    │ │    Worker Node N    │  │
│   │  ┌───────────────┐  │ │  ┌───────────────┐  │ │  ┌───────────────┐  │  │
│   │  │ LocalExecutor │  │ │  │ LocalExecutor │  │ │  │ LocalExecutor │  │  │
│   │  └───────────────┘  │ │  └───────────────┘  │ │  └───────────────┘  │  │
│   │  ┌───────────────┐  │ │  ┌───────────────┐  │ │  ┌───────────────┐  │  │
│   │  │ Storage Engine│  │ │  │ Storage Engine│  │ │  │ Storage Engine│  │  │
│   │  └───────────────┘  │ │  └───────────────┘  │ │  └───────────────┘  │  │
│   └─────────────────────┘ └─────────────────────┘ └─────────────────────┘  │
│                                                                              │
│   数据交换层:                                                                │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                      NetworkExchange                                 │   │
│   │  • gRPC / TCP 通信                                                   │   │
│   │  • RecordBatch 序列化                                                │   │
│   │  • 流式传输                                                          │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 8.2 执行流程

```
1. Client 发送 SQL
        │
        ▼
2. Coordinator 解析 SQL → LogicalPlan
        │
        ▼
3. Optimizer 优化 → PhysicalPlan
        │
        ▼
4. Fragmenter 切分 → PlanFragments
        │
        ▼
5. Dispatcher 分发 Fragments 到 Workers
        │
        ▼
6. Workers 执行 LocalExecutor
        │
        ▼
7. 数据通过 DataExchange 传输
        │
        ▼
8. Coordinator 合并结果
        │
        ▼
9. 返回给 Client
```

---

## 九、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-05 | 初始版本 |
