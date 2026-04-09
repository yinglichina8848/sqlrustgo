# 统一存储层实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现统一 Document 表，支持 SQL + Vector + Graph 元数据存储

**Architecture:** 
- 扩展 `ColumnDefinition` 添加向量列支持
- 创建 `DocumentTable` 结构统一管理 SQL/Vector/Graph
- 扩展 `IndexRegistry` 为统一索引管理
- 提供 Document 与 Graph Node 的关联机制

**Tech Stack:** sqlrustgo-storage, sqlrustgo-vector, sqlrustgo-graph

---

## Task 1: 创建 unified_storage crate 结构

**Files:**
- Create: `crates/unified-storage/Cargo.toml`
- Create: `crates/unified-storage/src/lib.rs`
- Create: `crates/unified-storage/src/document_table.rs`
- Create: `crates/unified-storage/src/unified_index.rs`
- Create: `crates/unified-storage/src/graph_link.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "sqlrustgo-unified-storage"
version.workspace = true
edition.workspace = true

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true

sqlrustgo-storage.workspace = true
sqlrustgo-vector.workspace = true
sqlrustgo-graph.workspace = true
sqlrustgo-types.workspace = true

[dev-dependencies]
tokio.workspace = true
```

**Step 2: Create lib.rs**

```rust
pub mod document_table;
pub mod unified_index;
pub mod graph_link;

pub use document_table::{DocumentColumn, DocumentTable, VectorColumn};
pub use unified_index::{UnifiedIndex, UnifiedIndexType};
pub use graph_link::{GraphLink, NodeRef};
```

---

## Task 2: 实现 DocumentColumn 和 VectorColumn

**Files:**
- Modify: `crates/unified-storage/src/document_table.rs`

**Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vector_column_definition() {
        let col = VectorColumn::new("embedding", 384, DistanceMetric::Cosine);
        assert_eq!(col.dimension, 384);
        assert_eq!(col.metric, DistanceMetric::Cosine);
    }
    
    #[test]
    fn test_document_column_mixed() {
        let mut table = DocumentTable::new("products");
        table.add_sql_column("id", "INTEGER");
        table.add_sql_column("name", "TEXT");
        table.add_vector_column("embedding", 384, DistanceMetric::Cosine);
        
        assert_eq!(table.sql_columns().len(), 2);
        assert_eq!(table.vector_columns().len(), 1);
    }
}
```

**Step 2: Create document_table.rs with stub**

```rust
use serde::{Deserialize, Serialize};
use sqlrustgo_vector::metrics::DistanceMetric;

#[derive(Debug, Clone)]
pub struct VectorColumn {
    pub name: String,
    pub dimension: usize,
    pub metric: DistanceMetric,
}

impl VectorColumn {
    pub fn new(name: &str, dimension: usize, metric: DistanceMetric) -> Self {
        Self {
            name: name.to_string(),
            dimension,
            metric,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DocumentColumn {
    Sql {
        name: String,
        data_type: String,
    },
    Vector(VectorColumn),
}

pub struct DocumentTable {
    name: String,
    columns: Vec<DocumentColumn>,
    primary_key: Option<String>,
}

impl DocumentTable {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: Vec::new(),
            primary_key: None,
        }
    }
    
    pub fn add_sql_column(&mut self, name: &str, data_type: &str) {
        self.columns.push(DocumentColumn::Sql {
            name: name.to_string(),
            data_type: data_type.to_string(),
        });
    }
    
    pub fn add_vector_column(&mut self, name: &str, dimension: usize, metric: DistanceMetric) {
        self.columns.push(DocumentColumn::Vector(VectorColumn::new(name, dimension, metric)));
    }
    
    pub fn sql_columns(&self) -> Vec<&DocumentColumn> {
        self.columns.iter().filter(|c| matches!(c, DocumentColumn::Sql { .. })).collect()
    }
    
    pub fn vector_columns(&self) -> Vec<&VectorColumn> {
        self.columns.iter().filter_map(|c| match c {
            DocumentColumn::Vector(v) => Some(v),
            _ => None,
        }).collect()
    }
}
```

**Step 3: Run test to verify it passes**

```bash
cd /Users/liying/workspace/dev/heartopen/sqlrustgo
cargo test -p sqlrustgo-unified-storage
```

---

## Task 3: 实现 UnifiedIndex

**Files:**
- Modify: `crates/unified-storage/src/unified_index.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_unified_index_creation() {
    let mut index = UnifiedIndex::new("products");
    index.add_btree_index(vec!["category".to_string()]);
    index.add_vector_index("embedding", 384, VectorIndexType::Hnsw);
    index.add_graph_index("node_ref");
    
    assert_eq!(index.index_count(), 3);
}
```

**Step 2: Create unified_index.rs with implementation**

```rust
use serde::{Deserialize, Serialize};
use sqlrustgo_storage::IndexId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnifiedIndexType {
    BTree,
    Hash,
    CompositeBTree,
    VectorFlat,
    VectorHnsw,
    VectorIvf,
    Graph,
}

#[derive(Debug, Clone)]
pub struct UnifiedIndex {
    table_name: String,
    indexes: Vec<UnifiedIndexMeta>,
}

#[derive(Debug, Clone)]
pub struct UnifiedIndexMeta {
    pub id: IndexId,
    pub name: String,
    pub index_type: UnifiedIndexType,
    pub column_names: Vec<String>,
}

impl UnifiedIndex {
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            indexes: Vec::new(),
        }
    }
    
    pub fn add_btree_index(&mut self, columns: Vec<String>) {
        let id = IndexId(self.indexes.len() as u32);
        self.indexes.push(UnifiedIndexMeta {
            id,
            name: format!("idx_{}_{}", self.table_name, columns.join("_")),
            index_type: UnifiedIndexType::BTree,
            column_names: columns,
        });
    }
    
    pub fn add_vector_index(&mut self, column: &str, dimension: usize, index_type: sqlrustgo_vector::VectorIndexType) {
        let id = IndexId(self.indexes.len() as u32);
        let idx_type = match index_type {
            sqlrustgo_vector::VectorIndexType::Flat => UnifiedIndexType::VectorFlat,
            sqlrustgo_vector::VectorIndexType::Hnsw => UnifiedIndexType::VectorHnsw,
            sqlrustgo_vector::VectorIndexType::Ivf => UnifiedIndexType::VectorIvf,
            sqlrustgo_vector::VectorIndexType::ParallelKnn => UnifiedIndexType::VectorFlat,
        };
        self.indexes.push(UnifiedIndexMeta {
            id,
            name: format!("vec_idx_{}_{}", self.table_name, column),
            index_type: idx_type,
            column_names: vec![column.to_string()],
        });
    }
    
    pub fn add_graph_index(&mut self, column: &str) {
        let id = IndexId(self.indexes.len() as u32);
        self.indexes.push(UnifiedIndexMeta {
            id,
            name: format!("graph_idx_{}_{}", self.table_name, column),
            index_type: UnifiedIndexType::Graph,
            column_names: vec![column.to_string()],
        });
    }
    
    pub fn index_count(&self) -> usize {
        self.indexes.len()
    }
}
```

**Step 3: Run test to verify it passes**

```bash
cargo test -p sqlrustgo-unified-storage
```

---

## Task 4: 实现 GraphLink

**Files:**
- Modify: `crates/unified-storage/src/graph_link.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_graph_link() {
    let link = GraphLink::new("products", "id", "product_node");
    assert_eq!(link.table_name, "products");
    assert_eq!(link.link_column, "id");
    assert_eq!(link.graph_label, "product_node");
}
```

**Step 2: Create graph_link.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GraphLink {
    pub table_name: String,
    pub link_column: String,
    pub graph_label: String,
}

impl GraphLink {
    pub fn new(table: &str, column: &str, graph_label: &str) -> Self {
        Self {
            table_name: table.to_string(),
            link_column: column.to_string(),
            graph_label: graph_label.to_string(),
        }
    }
}
```

**Step 3: Run test to verify it passes**

```bash
cargo test -p sqlrustgo-unified-storage
```

---

## Task 5: 添加到 workspace

**Files:**
- Modify: `Cargo.toml` (workspace root)

**Step 1: Add to workspace members**

```toml
members = [
    # ... existing crates ...
    "crates/unified-storage",
]
```

---

## Task 6: 运行完整测试

**Step 1: Run all tests**

```bash
cargo test -p sqlrustgo-unified-storage
cargo build -p sqlrustgo-unified-storage
```

**Step 2: Verify output**

```
running 5 tests
test tests::test_vector_column_definition ... ok
test tests::test_document_column_mixed ... ok
test tests::test_unified_index_creation ... ok
test tests::test_graph_link ... ok

test result: ok. 4 passed; 0 failed
```

---

## Task 7: 提交代码

**Step 1: Create branch**

```bash
git checkout -b feature/unified-storage
```

**Step 2: Commit**

```bash
git add -A
git commit -m "feat(unified-storage): add unified storage layer for v2.5.0

- Add DocumentTable supporting SQL + Vector columns
- Add UnifiedIndex for unified index management
- Add GraphLink for document-graph association
- Closes #1338"
```

**Step 3: Push and create PR**

```bash
git push -u origin feature/unified-storage
gh pr create --title "feat(unified-storage): unified storage layer (Issue #1338)" --base develop/v2.5.0
```
