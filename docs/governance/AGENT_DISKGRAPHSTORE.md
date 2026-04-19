# DiskGraphStore Agent 执行指南

## Issue #1378: Graph 持久化 - DiskGraphStore

**Agent 角色**: 实现 DiskGraphStore 持久化存储  
**目标**: 为 graph 模块添加磁盘持久化能力，支持 WAL 崩溃恢复

---

## 1. 上下文

### 1.1 当前位置
- 分支: `develop/v2.5.0`
- 目录: `crates/graph/src/store/`

### 1.2 已有组件
- `InMemoryGraphStore` - 内存图存储（已实现）
- `GraphStore` trait - 图存储 API
- `WalManager` - 现有 WAL 日志管理器
- 模型: `Node`, `Edge`, `PropertyMap`, `NodeId`, `EdgeId`, `LabelId`

### 1.3 目标文件
```
crates/graph/src/store/disk_graph_store.rs  (新建)
crates/graph/src/store/mod.rs              (修改 - 添加导出)
crates/graph/tests/graph_persistence_tests.rs (新建 - 测试)
```

---

## 2. 执行步骤

### Phase 1: 基础结构

#### Step 1.1: 创建 disk_graph_store.rs

```rust
//! DiskGraphStore - 持久化图存储，支持 WAL 崩溃恢复

use crate::model::*;
use crate::store::*;
use crate::error::GraphError;
use std::path::PathBuf;
use bincode;

// WAL 导入
use crate::wal::{WalManager, WalEntry};

pub struct DiskGraphStore {
    inner: InMemoryGraphStore,
    wal: WalManager,
    path: PathBuf,
    wal_enabled: bool,
    next_node_id: NodeId,
    next_edge_id: EdgeId,
}

#[derive(Serialize, Deserialize)]
pub enum GraphWalEntry {
    CreateNode { node_id: NodeId, label_id: LabelId, props: PropertyMap },
    UpdateNode { node_id: NodeId, props: PropertyMap },
    DeleteNode { node_id: NodeId },
    CreateEdge { edge_id: EdgeId, from: NodeId, to: NodeId, label_id: LabelId, props: PropertyMap },
    DeleteEdge { edge_id: EdgeId },
}

impl DiskGraphStore {
    pub fn new(base_path: PathBuf) -> Result<Self, GraphError> {
        // 1. 创建目录结构
        // 2. 初始化 WAL
        // 3. 初始化空的 InMemoryGraphStore
        // 4. 初始化元数据
    }
    
    pub fn load(base_path: PathBuf) -> Result<Self, GraphError> {
        // 1. 加载组件文件
        // 2. 重放 WAL
        // 3. 重建索引
    }
    
    fn persist_nodes(&self) -> Result<(), GraphError> { ... }
    fn persist_edges(&self) -> Result<(), GraphError> { ... }
    fn persist_labels(&self) -> Result<(), GraphError> { ... }
    fn persist_adjacency(&self) -> Result<(), GraphError> { ... }
    fn persist_meta(&self) -> Result<(), GraphError> { ... }
}
```

#### Step 1.2: 实现 GraphStore Trait

所有写操作:
1. 写入 WAL (log_*)
2. 操作 inner (内存)
3. 持久化相关组件
4. 同步 WAL (sync)

所有读操作:
- 直接委托给 `self.inner.*`

#### Step 1.3: 修改 mod.rs

```rust
mod disk_graph_store;
pub use disk_graph_store::*;
```

### Phase 2: WAL 集成

#### Step 2.1: 定义 GraphWalEntry

```rust
#[derive(Serialize, Deserialize)]
pub enum GraphWalEntry {
    CreateNode { node_id: NodeId, label_id: LabelId, props: PropertyMap },
    UpdateNode { node_id: NodeId, props: PropertyMap },
    DeleteNode { node_id: NodeId },
    CreateEdge { edge_id: EdgeId, from: NodeId, to: NodeId, label_id: LabelId, props: PropertyMap },
    DeleteEdge { edge_id: EdgeId },
}
```

#### Step 2.2: WAL 写入方法

```rust
fn log_create_node(&self, node_id: NodeId, label_id: LabelId, props: PropertyMap) -> Result<(), GraphError> {
    let entry = GraphWalEntry::CreateNode { node_id, label_id, props };
    let bytes = bincode::serialize(&entry).map_err(|e| GraphError::StorageError(e.to_string()))?;
    self.wal.log(self.current_tx_id, bytes)?;
    Ok(())
}
```

### Phase 3: 崩溃恢复

#### Step 3.1: 加载组件

```rust
fn load_components(&mut self) -> Result<(), GraphError> {
    // 尝试加载每个组件文件
    // 如果文件不存在或损坏，返回错误
}
```

#### Step 3.2: WAL 重放

```rust
fn replay_wal(&mut self) -> Result<(), GraphError> {
    let entries = self.wal.recover()?;
    for entry in entries {
        match entry {
            GraphWalEntry::CreateNode { ... } => self.inner.create_node(...),
            // ... 处理所有变体
        }
    }
    Ok(())
}
```

### Phase 4: 测试

#### Step 4.1: 编写测试

```rust
#[cfg(test)]
mod persistence_tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_disk_store_crud() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();
        
        // 创建节点
        let node_id = store.create_node("Batch", PropertyMap::new());
        assert!(store.get_node(node_id).is_some());
        
        // 重新加载
        let store2 = DiskGraphStore::load(dir.path().to_path_buf()).unwrap();
        assert!(store2.get_node(node_id).is_some());
    }
    
    #[test]
    fn test_wal_replay() {
        // 模拟崩溃后恢复
    }
}
```

---

## 3. 验证清单

### 代码检查
- [ ] `cargo build -p sqlrustgo-graph` 通过
- [ ] `cargo clippy -p sqlrustgo-graph -- -D warnings` 无警告
- [ ] 所有新增代码有适当的注释

### 测试检查
- [ ] `cargo test -p sqlrustgo-graph` 全部通过
- [ ] 新增测试覆盖:
  - [ ] CRUD 操作持久化
  - [ ] 重新加载数据完整性
  - [ ] WAL 重放正确性
  - [ ] 并发写入安全

### 集成检查
- [ ] 与现有 `InMemoryGraphStore` 行为一致
- [ ] 与 `WalManager` 正确集成

---

## 4. 关键约束

1. **不破坏现有 API**: `GraphStore` trait 实现必须与 `InMemoryGraphStore` 行为一致
2. **线程安全**: 使用 `DashMap` 确保并发安全
3. **向后兼容**: `InMemoryGraphStore` 保持不变
4. **最小依赖**: 不引入新的外部 crate（bincode 已在使用）

---

## 5. 常见问题

### Q: 为什么不直接修改 InMemoryGraphStore？
A: 装饰器模式保持单一职责，`InMemoryGraphStore` 专注内存操作，`DiskGraphStore` 专注持久化。

### Q: WAL 如何与图的 CRUD 操作对应？
A: WAL 记录图操作（CreateNode, DeleteEdge 等），而不是底层存储操作。每个图操作映射到一个 WAL 条目。

### Q: 如何处理大图？
A: 当前实现全量加载。后续可优化为懒加载或快照机制。

---

## 6. 提交规范

```
feat(graph): add DiskGraphStore with WAL persistence

- Implement DiskGraphStore wrapping InMemoryGraphStore
- Add WAL logging for all write operations
- Add crash recovery with WAL replay
- Add persistence tests

Closes #1378
```

---

**Agent 指令结束**
