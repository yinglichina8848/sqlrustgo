# DiskGraphStore 设计文档

## Issue #1378: Graph 持久化 - DiskGraphStore

**版本**: v2.5.0  
**状态**: 设计阶段  
**日期**: 2026-04-12

---

## 1. 背景与目标

### 1.1 问题陈述

当前 `graph` 模块只提供 `InMemoryGraphStore` 内存存储，节点和边在进程退出后丢失。需要实现 `DiskGraphStore` 将图数据持久化到磁盘。

### 1.2 设计目标

1. **WAL 集成**: 使用项目现有的 `WalManager` 实现事务安全和崩溃恢复
2. **组件化文件存储**: 分离存储 nodes、edges、adjacency、labels 到独立文件
3. **自动恢复**: 启动时从 WAL 重放操作重建图状态
4. **损坏处理**: 检测到文件损坏时删除并重建

---

## 2. 架构设计

### 2.1 装饰器模式

采用装饰器模式（Decorator Pattern），`DiskGraphStore` 包装 `InMemoryGraphStore`：

```
┌─────────────────────────────────────┐
│        DiskGraphStore               │
│  ┌───────────────────────────────┐  │
│  │   WalManager (WAL 日志)        │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │  InMemoryGraphStore (内存操作) │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

**优势**:
- 复用 `InMemoryGraphStore` 全部逻辑
- WAL 负责持久化和恢复
- 读写分离：读操作直接走内存，写操作透传到 WAL

### 2.2 文件结构

```
<base_path>/
├── nodes.bin      # NodeStore 序列化数据
├── edges.bin      # EdgeStore 序列化数据
├── adjacency.bin  # AdjacencyIndex 序列化数据
├── labels.bin     # LabelRegistry 序列化数据
└── meta.json      # 元数据: next_node_id, next_edge_id, version
```

### 2.3 数据流

```
写入操作:
  client.write() → wal.log_*() → inner.*() → persist_component() → wal.sync()

读取操作:
  client.read() → inner.*() (直接从内存)

崩溃恢复:
  startup → load_components() → replay_wal() → rebuild_indices()
```

---

## 3. 核心组件

### 3.1 DiskGraphStore 结构体

```rust
pub struct DiskGraphStore {
    inner: InMemoryGraphStore,
    wal: WalManager,
    path: PathBuf,
    wal_enabled: bool,
}
```

### 3.2 WAL 日志条目

```rust
pub enum GraphWalEntry {
    CreateNode { node_id: NodeId, label: LabelId, props: PropertyMap },
    UpdateNode { node_id: NodeId, props: PropertyMap },
    DeleteNode { node_id: NodeId },
    CreateEdge { edge_id: EdgeId, from: NodeId, to: NodeId, label: LabelId, props: PropertyMap },
    DeleteEdge { edge_id: EdgeId },
}
```

### 3.3 元数据格式 (meta.json)

```json
{
    "version": "1.0",
    "next_node_id": 1000,
    "next_edge_id": 500,
    "last_snapshot_ts": 1234567890
}
```

---

## 4. GraphStore Trait 实现

### 4.1 写操作

所有写操作遵循以下模式：

```rust
fn create_node(&mut self, label: &str, props: PropertyMap) -> NodeId {
    // 1. 分配 node_id
    let label_id = self.inner.labels.get_or_register(label);
    let node_id = self.next_node_id();
    
    // 2. 写入 WAL
    if self.wal_enabled {
        self.wal.log_create_node(node_id, label_id, props.clone())?;
    }
    
    // 3. 写入内存
    self.inner.create_node(label, props);
    
    // 4. 持久化组件
    self.persist_nodes()?;
    
    // 5. 同步 WAL
    if self.wal_enabled {
        self.wal.sync()?;
    }
    
    node_id
}
```

### 4.2 读操作

直接委托给 `inner`:

```rust
fn get_node(&self, id: NodeId) -> Option<Node> {
    self.inner.get_node(id)
}

fn nodes_by_label(&self, label: &str) -> Vec<NodeId> {
    self.inner.nodes_by_label(label)
}
```

---

## 5. 崩溃恢复流程

### 5.1 启动时恢复

```
1. 检查数据目录是否存在
   ├── 不存在 → 创建目录，初始化空图
   └── 存在 → 继续步骤 2

2. 加载组件文件
   ├── 全部存在 → 加载 nodes.bin, edges.bin, labels.bin
   └── 任何文件损坏 → 删除损坏文件，从 WAL 重建

3. 重放 WAL
   ├── WAL 存在 → 读取所有条目，按顺序应用到内存
   └── WAL 不存在 → 图加载完成

4. 重建索引
   └── 验证 adjaceny 和 label_index 与数据一致
```

### 5.2 损坏处理策略

当检测到文件损坏时：
1. 记录错误日志
2. 删除损坏的文件
3. 如果有 WAL → 从 WAL 重放重建
4. 如果没有 WAL → 放弃该组件，从空状态开始

---

## 6. 序列化方案

### 6.1 组件序列化

| 组件 | 序列化格式 | 说明 |
|------|-----------|------|
| NodeStore | bincode | 包含 DashMap<NodeId, Node> |
| EdgeStore | bincode | 包含 DashMap<EdgeId, Edge> |
| AdjacencyIndex | bincode | 复杂嵌套结构 |
| LabelRegistry | bincode | HashMap<String, LabelId> |
| meta.json | JSON | 版本和计数器信息 |

### 6.2 WAL 序列化

WAL 使用 bincode 序列化 `GraphWalEntry` 枚举。

---

## 7. 配置与 API

### 7.1 构造函数

```rust
impl DiskGraphStore {
    /// 创建新的持久化图存储
    pub fn new(base_path: PathBuf) -> Result<Self, GraphError>;
    
    /// 创建时不启用 WAL（更快但不安全）
    pub fn new_without_wal(base_path: PathBuf) -> Result<Self, GraphError>;
    
    /// 从现有数据加载
    pub fn load(base_path: PathBuf) -> Result<Self, GraphError>;
}
```

### 7.2 管理接口

```rust
impl DiskGraphStore {
    /// 刷新所有数据到磁盘
    pub fn flush(&mut self) -> Result<(), GraphError>;
    
    /// 获取 WAL 恢复后的未执行条目
    pub fn recover(&self) -> Result<Vec<GraphWalEntry>, GraphError>;
    
    /// 手动 checkpoint（截断 WAL）
    pub fn checkpoint(&mut self) -> Result<(), GraphError>;
}
```

---

## 8. 测试计划

### 8.1 单元测试

| 测试 | 描述 |
|------|------|
| `test_disk_store_crud` | 基本 CRUD 操作后数据正确持久化 |
| `test_disk_store_reload` | 重启后数据完整加载 |
| `test_wal_replay` | 模拟崩溃后 WAL 重放正确 |
| `test_concurrent_write` | 多线程写入安全性 |
| `test_corruption_recovery` | 文件损坏时的恢复行为 |

### 8.2 集成测试

| 测试 | 描述 |
|------|------|
| `test_graph_with_existing_wal` | 与现有 WAL 系统集成 |
| `test_large_graph_persistence` | 大量节点的持久化性能 |

---

## 9. 依赖关系

### 9.1 内部依赖

- `crates/graph/src/store/graph_store.rs` - GraphStore trait
- `crates/graph/src/model/` - Node, Edge, PropertyMap 等模型
- `crates/storage/src/wal/` - WalManager

### 9.2 外部依赖

- `bincode` - 二进制序列化（已存在于项目中）

---

## 10. 实现计划

### Phase 1: 基础结构
- [ ] 创建 `disk_graph_store.rs` 文件
- [ ] 实现 `DiskGraphStore` 结构体和构造函数
- [ ] 实现组件序列化/反序列化

### Phase 2: WAL 集成
- [ ] 定义 `GraphWalEntry` 枚举
- [ ] 实现 WAL 日志写入
- [ ] 实现 WAL 重放

### Phase 3: GraphStore Trait
- [ ] 实现所有写操作（带 WAL 和持久化）
- [ ] 实现所有读操作（委托给 inner）
- [ ] 实现遍历操作（BFS/DFS）

### Phase 4: 恢复与测试
- [ ] 实现崩溃恢复流程
- [ ] 添加损坏检测与恢复
- [ ] 编写完整测试套件

---

## 11. 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| 并发写入冲突 | 使用 DashMap 提供线程安全 |
| WAL 膨胀 | 定期 checkpoint 截断日志 |
| 大图加载慢 | 考虑懒加载或快照机制 |
| 序列化版本升级 | meta.json 中存储 version 字段 |
