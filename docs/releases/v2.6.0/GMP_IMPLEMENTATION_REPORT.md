# SQLRustGo GMP (Graph Memory Processor) 实现报告

> **版本**: v2.6.0
> **日期**: 2026-04-22
> **作者**: Sisyphus (via OpenCode)
> **分支**: feat/gmp-cli-binary → develop/v2.6.0

---

## 一、概述

### 1.1 GMP 是什么

GMP (Graph Memory Processor) 是一个面向 GMP（良好生产规范）合规文档管理的向量检索与图查询系统。它结合了：

- **文档管理**: 版本化、类型化的 GMP 合规文档
- **向量搜索**: 基于文本嵌入的语义相似度检索
- **混合搜索**: 向量 + 全文 + 图关系的融合检索
- **图数据库**: 文档关系的图结构存储与查询
- **持久化**: SQLite 本地存储，支持双写和热重载

### 1.2 核心能力

| 能力 | 说明 |
|------|------|
| 文档导入 | 批量导入文档，自动生成向量嵌入 |
| 向量搜索 | 3584 维 hash-based 嵌入，cosine 相似度 |
| 混合搜索 | 60% 向量 + 40% 全文的 RRF 融合 |
| 图关系 | 节点/边的创建、查询、邻居发现 |
| 三路搜索 | 向量 + 图关系 + SQL 精确匹配 |
| 持久化 | SQLite 双写，服务重启后自动恢复 |

---

## 二、系统架构

### 2.1 组件结构

```
sqlrustgo-gmp (crate)
├── src/
│   ├── lib.rs              # 公共 API 导出
│   ├── cli.rs              # CLI 二进制入口 (sqlrustgo-gmp-cli)
│   ├── sql_api.rs          # GmpExecutor API (库模式)
│   ├── document.rs         # 文档管理
│   ├── embedding.rs        # 向量嵌入生成
│   ├── vector_search.rs    # 向量/混合搜索
│   ├── persist_sqlite.rs   # SQLite 持久化层
│   ├── audit.rs            # 审计日志
│   ├── compliance.rs        # 合规检查
│   ├── report.rs           # 报告生成
│   └── semantic_embedding.rs # 语义嵌入 (Ollama/OpenAI)
└── Cargo.toml
```

### 2.2 CLI 协议

CLI 使用 JSON Lines 协议，通过 stdin/stdout 通信：

```json
// 请求
{"cmd": "VectorSearch", "query": "文档内容", "top_k": 5}

// 响应
{"success": true, "data": [...], "time_ms": 12}
```

### 2.3 支持的命令

| 命令 | 功能 |
|------|------|
| `Ping` | 健康检查 |
| `Init` | 初始化表结构 |
| `VectorSearch` | 向量相似度搜索 |
| `HybridSearch` | 混合搜索 |
| `ImportDoc` | 导入文档 |
| `UpsertNode` | 创建/更新图节点 |
| `UpsertEdge` | 创建/更新图边 |
| `GraphNeighbors` | 查找邻居节点 |
| `ListNodes` | 列出节点 |
| `ListEdgeTypes` | 列出边类型 |
| `SqlSearch` | SQL 精确搜索 |
| `TripleSearch` | 三路 RRF 融合搜索 |

---

## 三、核心功能实现

### 3.1 文档管理 (document.rs)

```rust
// 文档状态
pub enum DocStatus {
    Draft,      // 草稿
    Active,     // 生效中
    Archived,   // 已归档
    Superseded, // 已废弃
}

// 文档表结构
CREATE TABLE gmp_documents (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    doc_type TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    effective_date INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'DRAFT'
);
```

**支持的查询**:
- 按类型查询 (`query_by_type`)
- 按状态查询 (`query_by_status`)
- 按生效日期查询 (`query_by_effective_date`)
- 按关键词全文搜索

### 3.2 向量嵌入 (embedding.rs)

```rust
// Hash-based 嵌入模型
pub struct HashEmbeddingModel { ... }

// 生成 3584 维向量
pub fn generate_embedding(text: &str) -> Vec<f32> {
    // 使用文本 hash + n-gram 生成固定维度向量
}

// 相似度计算
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32
```

### 3.3 向量搜索 (vector_search.rs)

**向量搜索**:
```rust
pub fn vector_search(
    storage: &dyn StorageEngine,
    query: &str,
    top_k: usize,
) -> SqlResult<Vec<SearchResult>>
```

**混合搜索** (60% 向量 + 40% 全文):
```rust
pub fn hybrid_search(
    storage: &dyn StorageEngine,
    query: &str,
    top_k: usize,
) -> SqlResult<Vec<SearchResult>>
```

### 3.4 图存储 (persist_sqlite.rs)

**图节点表**:
```sql
CREATE TABLE graph_nodes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    node_type TEXT NOT NULL,
    properties TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(name, node_type)
);
```

**图边表**:
```sql
CREATE TABLE graph_edges (
    id INTEGER PRIMARY KEY,
    src INTEGER NOT NULL,
    dst INTEGER NOT NULL,
    edge_type TEXT NOT NULL,
    properties TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(src) REFERENCES graph_nodes(id),
    FOREIGN KEY(dst) REFERENCES graph_nodes(id)
);
```

**关键设计**: 边关系在 GMP 图中是双向的，邻居查询同时返回出边和入边。

### 3.5 SQLite 持久化层 (persist_sqlite.rs)

**StorageBackend trait** (Stage 1→3 迁移契约):
```rust
pub trait StorageBackend: Send + Sync {
    fn load_documents(&self) -> Result<Vec<DocumentRecord>, String>;
    fn save_document(&self, doc: &DocumentRecord) -> Result<i64, String>;
    fn load_embeddings(&self) -> Result<Vec<EmbeddingRecord>, String>;
    fn save_embedding(&self, emb: &EmbeddingRecord) -> Result<(), String>;
    fn load_nodes(&self) -> Result<Vec<NodeRecord>, String>;
    fn save_node(&self, node: &NodeRecord) -> Result<i64, String>;
    fn load_edges(&self) -> Result<Vec<EdgeRecord>, String>;
    fn save_edge(&self, edge: &EdgeRecord) -> Result<i64, String>;
    fn upsert_edge_by_names(&self, edge: &EdgeUpsertRecord) -> Result<i64, String>;
}
```

**双写策略**:
1. 所有写入同时更新内存和 SQLite
2. 服务启动时从 SQLite 热重载到内存
3. 支持 `$SQLRUSTGO_HOME` 自定义数据目录

### 3.6 三路搜索 (TripleSearch)

RRF (Reciprocal Rank Fusion) 融合:
```
Score = Σ (1 / (k + rank_i)) for i in [vector, graph, sql]
k = 60 (常量)
```

---

## 四、GmpExecutor API

库模式下的高级 API：

```rust
use sqlrustgo_gmp::{GmpExecutor, create_gmp_tables};
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock}};

let storage = Arc::new(RwLock::new(MemoryStorage::new()));
let executor = GmpExecutor::new(storage);

// 初始化
executor.init()?;

// 导入文档
let doc_id = executor.import_document(
    "GMP 变更控制程序",
    "SOP",
    "本文件描述变更控制流程...",
    &["变更", "控制", "GMP"],
)?;

// 向量搜索
let results = executor.search("变更控制流程", 5)?;

// 混合搜索
let results = executor.hybrid_search("如何处理变更", 10)?;

// 批量导入
executor.bulk_import(&[
    ("文档1", "SOP", "内容1", vec!["key1"]),
    ("文档2", "SOP", "内容2", vec!["key2"]),
])?;
```

---

## 五、CLI 使用方式

### 5.1 构建

```bash
cargo build -p sqlrustgo-gmp
```

### 5.2 启动服务

```bash
# 设置数据目录 (可选)
export SQLRUSTGO_HOME=/path/to/data

# 启动 CLI (交互模式)
cargo run -p sqlrustgo-gmp-cli

# 或使用 JSON Lines 协议
echo '{"cmd":"Ping"}' | cargo run -p sqlrustgo-gmp-cli
```

### 5.3 命令示例

**导入文档**:
```json
{
  "cmd": "ImportDoc",
  "title": "变更控制程序 v2.0",
  "doc_type": "SOP",
  "content": "本文件描述变更控制流程...",
  "keywords": ["变更", "控制"],
  "properties": {"department": "QA", "chapter": "5"}
}
```

**向量搜索**:
```json
{"cmd": "VectorSearch", "query": "变更控制", "top_k": 5}
```

**图邻居查询**:
```json
{"cmd": "GraphNeighbors", "node_type": "SOP", "node_name": "变更控制程序"}
```

**三路搜索**:
```json
{"cmd": "TripleSearch", "query": "变更控制流程", "top_k": 10}
```

---

## 六、应用场景

### 6.1 GMP 合规文档管理

- **文档版本控制**: 每份文档带版本号和生效日期
- **状态流转**: Draft → Active → Archived/Superseded
- **审计日志**: 所有操作记录防篡改 checksum
- **合规检查**: 验证文档完整性、有效期、审批状态

### 6.2 语义搜索

- **研发知识库**: 代码文档、技术规范的语义检索
- **SOP 搜索**: 通过自然语言找到相关操作规程
- **问答系统**: 相似问题推荐

### 6.3 知识图谱

- **文档关联**: 通过图关系发现相关文档
- **依赖分析**: 变更影响的文档链路
- **邻居发现**: 某文档相关的所有内容

### 6.4 RAG (Retrieval-Augmented Generation)

```rust
// RAG 场景示例
let docs = executor.hybrid_search(user_query, 5)?;
// 将 top-k 文档内容注入 LLM prompt
```

---

## 七、改进工作 (本 PR)

### 7.1 修复的问题

| 问题 | 修复 |
|------|------|
| tonic 版本不一致 | `transaction/Cargo.toml` 使用 `tonic = "0.12"` 显式指定 |
| unused Result 警告 | `cli.rs:292` 添加 `let _ =` |

### 7.2 新增功能

| 功能 | 文件 | 说明 |
|------|------|------|
| GraphNeighbors 双向边 | `cli.rs` | 同时返回出边和入边邻居 |
| SqlSearch 章节过滤 | `cli.rs` | 支持 department/category/chapter 过滤 |
| save_document 属性 | `persist_sqlite.rs` | 完整的 properties JSON 支持 |

### 7.3 构建验证

```bash
cargo build -p sqlrustgo-gmp  # ✅ 成功 (3 warnings, 0 errors)
cargo build -p sqlrustgo-transaction  # ✅ 成功
```

---

## 八、数据流

```
┌─────────────────────────────────────────────────────────────────┐
│                      CLI (sqlrustgo-gmp-cli)                     │
│  JSON Lines 协议: {"cmd": "VectorSearch", "query": "...", ...}  │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                       GmpCliState                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ GmpExecutor │  │ GraphStore │  │ SqliteBackend           │  │
│  │ (文档/向量) │  │ (图存储)   │  │ (持久化: gmp.db)       │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                ┌───────────┴───────────┐
                ▼                       ▼
        ┌───────────────┐       ┌───────────────┐
        │ MemoryStorage│       │ InMemoryGraph│
        │ (内存文档)   │       │ Store (内存图)│
        └───────────────┘       └───────────────┘
```

---

## 九、测试

### 9.1 单元测试

```bash
cargo test -p sqlrustgo-gmp
```

### 9.2 集成测试

```rust
#[test]
fn test_vector_search() {
    let mut storage = MemoryStorage::new();
    create_gmp_tables(&mut storage).unwrap();
    create_embeddings_table(&mut storage).unwrap();

    // 插入文档和嵌入...

    let results = vector_search(&storage, "Rust memory safety", 2).unwrap();
    assert!(!results.is_empty());
}
```

---

## 十、限制与已知问题

| 问题 | 严重程度 | 说明 |
|------|----------|------|
| 向量嵌入基于 hash | 低 | 非真正语义嵌入，适用于近似匹配 |
| SQLite 单文件 | 中 | 不支持高并发写入 |
| 图关系无事务 | 中 | 内存图和 SQLite 可能短暂不一致 |

---

## 十一、未来工作

1. **真正的语义嵌入**: 集成 Ollama/OpenAI API
2. **事务支持**: 图操作纳入事务
3. **分布式**: GraphShard 支持水平扩展
4. **性能优化**: 索引加速 SQLite 查询

---

## 附录 A: 文件清单

| 文件 | 行数 | 功能 |
|------|------|------|
| `src/lib.rs` | 99 | 公共 API 导出 |
| `src/cli.rs` | 1060 | CLI 实现 + 命令处理 |
| `src/sql_api.rs` | 341 | GmpExecutor 库 API |
| `src/document.rs` | 635 | 文档管理 |
| `src/embedding.rs` | ~300 | 向量嵌入 |
| `src/vector_search.rs` | 397 | 向量/混合搜索 |
| `src/persist_sqlite.rs` | 577 | SQLite 持久化 |
| `src/audit.rs` | ~200 | 审计日志 |
| `src/compliance.rs` | ~150 | 合规检查 |
| `src/report.rs` | ~200 | 报告生成 |

---

*本报告由 Sisyphus AI Agent 生成*
*基于 sqlrustgo-gmp crate v2.6.0*
