# SQLRustGo 2.x 系列开发规划

> 版本: 2.0 → 2.5
> 更新日期: 2026-03-29
> 目标: 企业级 GMP 文档精准检索 + RAG + 知识图谱 + OpenClaw AI 驱动 + **AgentSQL Native**

---

## 📋 版本演进总览

| 版本 | 核心目标 | 主要功能 | AI 开发线 |
|------|----------|----------|------------|
| **2.0** | GA 单机稳定 | Phase1-5 完成，RDBMS 核心，TaskScheduler, ParallelExecutor | OpenCode A/B, Claude A/B |
| **2.1** | AgentSQL + GMP检索 | 文档导入/向量化/OpenClaw SQL API/**AgentSQL Extension** | OpenCode A/B, Claude A |
| **2.2** | 高性能向量数据库 | 向量索引/并行KNN/SQL+Vector联合查询 | OpenCode B, Claude A |
| **2.3** | RAG + 全文检索 | 文档问答/LLM集成/OpenClaw驱动 | Claude A, Claude B |
| **2.4** | 知识图谱 + 图检索 | 节点/边表/BFS/DFS/路径搜索 | OpenCode A, Claude B |
| **2.5** | 全面集成 + GMP | SQL+Vector+Graph/OpenClaw全自动/GMP报表 | OpenCode A, Claude B |

---

## 🏗 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                     用户/Agent (OpenClaw + LLM)                 │
└────────────────────────────┬────────────────────────────────────┘
                             │ 调用统一 API
┌────────────────────────────▼────────────────────────────────────┐
│                  SQLRustGo API 层                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ SQL API  │ │Vector API│ │Graph API │ │ RAG API  │          │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘          │
└───────┼────────────┼────────────┼────────────┼──────────────────┘
        │            │            │            │
┌───────▼────────────▼────────────▼────────────▼──────────────────┐
│                    SQLRustGo 核心引擎                            │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│  │   RDBMS 层   │ │ 向量计算函数  │ │  图检索引擎  │            │
│  │ Columnar/Row │ │cosine/knn   │ │ BFS/DFS     │            │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘            │
│         │                │                │                     │
│  ┌──────▼────────────────▼────────────────▼───────┐             │
│  │              存储层 (ColumnarStorage)           │             │
│  │  文档表    │   向量表    │   知识图谱表        │             │
│  └────────────────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🔄 依赖关系

```
2.0 (GA 稳定)
   │
   ▼
2.1 (文档 + 向量化 + SQL API)
   │  └── 文档表结构
   │  └── 向量存储表
   │  └── OpenClaw SQL API
   │
   ▼
2.2 (高性能向量索引)
   │  └── IVF/HNSW 索引
   │  └── 并行 KNN
   │  └── SQL + Vector 联合
   │
   ▼
2.3 (RAG + 全文检索)
   │  └── 倒排索引
   │  └── RAG Pipeline
   │  └── llama.cpp 集成
   │
   ▼
2.4 (知识图谱)
   │  └── 节点/边表
   │  └── 图检索算法
   │  └── SQL + Graph 联合
   │
   ▼
2.5 (全面集成 + GMP)
       └── 全模块整合
       └── OpenClaw 全局调度
       └── GMP 内审报告
```

---

## 📦 v2.1 GMP 文档精准检索原型

**Issue**: #1075  
**目标**: 构建企业级 GMP 文档精准检索原型，支持文档管理和 OpenClaw SQL 访问

### 1. 文档表结构

```sql
-- 文档主表
CREATE TABLE documents (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    doc_type ENUM('SOP', '规范', '审核', '报告', '日志') NOT NULL,
    version TEXT,
    status ENUM('draft', 'approved', 'archived') DEFAULT 'draft',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    effective_date DATE,
    department TEXT,
    author TEXT
);

-- 文档内容表（按章节拆分）
CREATE TABLE document_contents (
    id UUID PRIMARY KEY,
    doc_id UUID REFERENCES documents(id) ON DELETE CASCADE,
    section TEXT,
    content TEXT,
    content_hash TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 关键字/标签表
CREATE TABLE document_keywords (
    id UUID PRIMARY KEY,
    doc_id UUID REFERENCES documents(id) ON DELETE CASCADE,
    keyword TEXT,
    source ENUM('manual', 'auto_extract') DEFAULT 'manual',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 全文索引
CREATE FULLTEXT INDEX idx_doc_content ON document_contents(content);
CREATE INDEX idx_keywords ON document_keywords(keyword);
```

### 2. 向量存储表

```sql
-- 文档向量表
CREATE TABLE document_embeddings (
    id UUID PRIMARY KEY,
    doc_id UUID REFERENCES documents(id) ON DELETE CASCADE,
    content_id UUID REFERENCES document_contents(id) ON DELETE CASCADE,
    embedding FLOAT[] NOT NULL,  -- 或 VECTOR(768)
    model_name TEXT DEFAULT 'embedding-model',
    chunk_text TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 向量索引表（用于 IVF/HNSW）
CREATE TABLE vector_index (
    id UUID PRIMARY KEY,
    index_type ENUM('ivf', 'hnsw', 'flat') NOT NULL,
    vector_dim INT NOT NULL,
    index_data BYTEA,
    metadata JSONB
);
```

### 3. OpenClaw SQL API

```json
// sql_exec: 执行 SQL 查询
{
  "action": "sql_exec",
  "sql": "SELECT * FROM documents WHERE doc_type = 'SOP' AND status = 'approved'",
  "params": {},
  "options": {
    "parallel_degree": 4,
    "timeout_ms": 5000
  }
}

// sql_batch: 批量操作
{
  "action": "sql_batch",
  "operations": [
    {"sql": "INSERT INTO documents (...) VALUES (...)"},
    {"sql": "INSERT INTO document_contents (...) VALUES (...)"}
  ],
  "transaction": true
}

// vector_embed: 文档嵌入向量生成
{
  "action": "vector_embed",
  "text": "GMP 清洁验证标准操作流程",
  "model": "embedding-model",
  "options": {
    "batch_size": 32
  }
}

// vector_search: 向量相似度检索
{
  "action": "vector_search",
  "query": "批量记录质量管理",
  "table": "document_embeddings",
  "top_k": 10,
  "filters": {
    "doc_type": ["SOP", "规范"],
    "status": ["approved"]
  },
  "score_threshold": 0.7
}
```

### 4. 性能要求

| 指标 | 要求 |
|------|------|
| 1M 文档检索响应 | < 300ms |
| 向量生成延迟 | < 500ms/文档 |
| 批量导入吞吐 | > 1000 docs/s |

---

## 📦 v2.2 高性能向量数据库

**Issue**: #1076  
**目标**: 实现高性能向量索引和并行 KNN 检索

### 1. 向量索引

```sql
-- IVF 倒排文件索引
CREATE VECTOR INDEX ivf_doc_emb USING ivf 
ON document_embeddings(embedding)
WITH (nlists = 100, nprobes = 10);

-- HNSW 层次导航小世界图索引
CREATE VECTOR INDEX hnsw_doc_emb USING hnsw
ON document_embeddings(embedding)
WITH (m = 16, ef_construction = 200, ef_search = 100);
```

### 2. 并行 KNN 计算

```rust
// 向量距离计算函数
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
}

pub fn inner_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

// 并行 Top-K 检索
pub async fn parallel_knn_search(
    query: &[f32],
    vectors: &VectorStore,
    k: usize,
    parallel_degree: usize,
) -> Vec<(u64, f32)> {
    // 使用 ParallelExecutor 加速
}
```

### 3. SQL + Vector 联合查询

```sql
-- SQL WHERE 预过滤 + Vector Top-K 排序
SELECT 
    d.id, d.title, d.doc_type,
    ve.cosine_score,
    ts_rank(content, plainto_tsquery('GMP 清洁验证')) AS text_score,
    0.5 * ve.cosine_score + 0.5 * ts_rank(content, plainto_tsquery('GMP 清洁验证')) AS final_score
FROM documents d
JOIN document_embeddings ve ON d.id = ve.doc_id
JOIN document_contents c ON d.id = c.doc_id
WHERE d.doc_type IN ('SOP', '规范')
  AND d.status = 'approved'
ORDER BY final_score DESC
LIMIT 10;
```

### 4. API 扩展

```json
// hybrid_search: SQL + Vector 联合检索
{
  "action": "hybrid_search",
  "query": "GMP 清洁验证 SOP",
  "sql_filters": {
    "doc_type": ["SOP", "规范"],
    "status": ["approved"],
    "effective_date_from": "2025-01-01"
  },
  "vector_field": "embedding",
  "top_k": 10,
  "weights": {
    "sql_score": 0.5,
    "vector_score": 0.5
  }
}
```

---

## 📦 v2.3 RAG + 全文检索 + 知识库

**Issue**: #1077  
**目标**: 实现 RAG 问答系统和知识库管理

### 1. 全文检索

```sql
-- 倒排索引
CREATE INDEX idx_fts_content ON document_contents 
USING GIN(to_tsvector('chinese', content));

-- 分词器配置
CREATE TEXT SEARCH CONFIGURATION chinese_zh (COPY chinese);
ALTER TEXT SEARCH CONFIGURATION chinese_zh 
ADD MAPPING FOR n,v,a,i,e WITH simple;
```

### 2. RAG Pipeline

```rust
// RAG 检索流程
pub struct RAGPipeline {
    sql_executor: SqlExecutor,
    vector_store: VectorStore,
    llm: LlmClient,  // llama.cpp
}

impl RAGPipeline {
    // 1. 检索相关文档
    pub async fn retrieve(&self, query: &str, top_k: usize) -> Vec<DocChunk> {
        // SQL 过滤
        let sql_results = self.sql_executor.query(&query).await?;
        // 向量检索
        let vector_results = self.vector_store.search(&query, top_k).await?;
        // 融合排序
        self.fusion_rerank(sql_results, vector_results)
    }

    // 2. 生成回答
    pub async fn generate(&self, query: &str, context: &[DocChunk]) -> String {
        let prompt = format!(
            "基于以下上下文回答问题。\n\n上下文:\n{}\n\n问题: {}",
            context.iter().map(|c| c.content.as_str()).join("\n---\n"),
            query
        );
        self.llm.generate(&prompt).await
    }
}
```

### 3. 知识库管理

```sql
-- 知识库表
CREATE TABLE knowledge_base (
    id UUID PRIMARY KEY,
    kb_name TEXT NOT NULL,
    kb_type ENUM('document', 'web', 'api', 'database') NOT NULL,
    source_uri TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB
);

-- 知识条目
CREATE TABLE knowledge_entries (
    id UUID PRIMARY KEY,
    kb_id UUID REFERENCES knowledge_base(id) ON DELETE CASCADE,
    entry_type ENUM('chunk', 'entity', 'relation') NOT NULL,
    content TEXT,
    embedding FLOAT[],
    metadata JSONB,
    version INT DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 知识库版本
CREATE TABLE knowledge_versions (
    id UUID PRIMARY KEY,
    kb_id UUID REFERENCES knowledge_base(id) ON DELETE CASCADE,
    version INT NOT NULL,
    change_log TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 4. API

```json
// query_knowledge: 知识检索
{
  "action": "query_knowledge",
  "query": "清洁验证的标准是什么？",
  "kb_ids": ["kb-001", "kb-002"],
  "top_k": 5,
  "mode": "hybrid"  // "sql" | "vector" | "hybrid"
}

// rag_query: RAG 问答
{
  "action": "rag_query",
  "question": "GMP对清洁验证有哪些具体要求？",
  "context_docs": 5,
  "include_sources": true
}

// batch_query: 批量问题处理
{
  "action": "batch_query",
  "questions": [
    "清洁验证的步骤是什么？",
    "哪些SOP涉及设备维护？",
    "最近有哪些规范更新？"
  ],
  "parallel": true
}
```

---

## 📦 v2.4 知识图谱 + 图检索

**Issue**: #1078  
**目标**: 构建知识图谱，支持图检索和关系分析

### 1. 知识图谱表结构

```sql
-- 节点表
CREATE TABLE kg_nodes (
    id UUID PRIMARY KEY,
    type TEXT NOT NULL,  -- 'SOP', 'Step', 'Device', 'Regulation', 'Person', 'Material'
    name TEXT NOT NULL,
    description TEXT,
    metadata JSONB,  -- 存储额外属性
    embedding FLOAT[],  -- 节点向量
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 边表
CREATE TABLE kg_edges (
    id UUID PRIMARY KEY,
    src_id UUID REFERENCES kg_nodes(id) ON DELETE CASCADE,
    dst_id UUID REFERENCES kg_nodes(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL,  -- 'depends_on', 'related_to', 'implements', 'belongs_to', 'affects'
    weight FLOAT DEFAULT 1.0,
    metadata JSONB,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_kg_nodes_type ON kg_nodes(type);
CREATE INDEX idx_kg_nodes_name ON kg_nodes USING gin(to_tsvector('simple', name));
CREATE INDEX idx_kg_edges_src ON kg_edges(src_id);
CREATE INDEX idx_kg_edges_dst ON kg_edges(dst_id);
CREATE INDEX idx_kg_edges_rel ON kg_edges(relation_type);
```

### 2. 图检索算法

```rust
// BFS 遍历
pub fn bfs_traverse(graph: &Graph, start: u64, max_depth: usize) -> Vec<Node> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();
    
    queue.push_back((start, 0));
    visited.insert(start);
    
    while let Some((node_id, depth)) = queue.pop_front() {
        if depth > max_depth { break; }
        result.push(graph.get_node(node_id));
        
        for edge in graph.outgoing_edges(node_id) {
            if !visited.contains(&edge.dst_id) {
                visited.insert(edge.dst_id);
                queue.push_back((edge.dst_id, depth + 1));
            }
        }
    }
    result
}

// 最短路径 (Dijkstra)
pub fn shortest_path(graph: &Graph, src: u64, dst: u64) -> Option<Path> {
    let mut dist = HashMap::new();
    let mut prev = HashMap::new();
    let mut pq = PriorityQueue::new();
    
    dist.insert(src, 0.0);
    pq.push(src, 0.0);
    
    while let Some((node, d)) = pq.pop() {
        if node == dst { break; }
        if d > *dist.get(&node).unwrap_or(&f32::MAX) { continue; }
        
        for edge in graph.outgoing_edges(node) {
            let new_dist = d + edge.weight;
            if new_dist < *dist.get(&edge.dst_id).unwrap_or(&f32::MAX) {
                dist.insert(edge.dst_id, new_dist);
                prev.insert(edge.dst_id, node);
                pq.push(edge.dst_id, new_dist);
            }
        }
    }
    // 重建路径
}

// 环检测
pub fn detect_cycles(graph: &Graph) -> Vec<Vec<u64>> {
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();
    let mut cycles = Vec::new();
    
    fn dfs(graph: &Graph, node: u64, visited: &mut HashSet<u64>, 
           stack: &mut HashSet<u64>, cycles: &mut Vec<Vec<u64>>) {
        visited.insert(node);
        stack.insert(node);
        
        for edge in graph.outgoing_edges(node) {
            if !visited.contains(&edge.dst_id) {
                dfs(graph, edge.dst_id, visited, stack, cycles);
            } else if stack.contains(&edge.dst_id) {
                // 发现环
                cycles.push(vec![edge.dst_id, node]);  // 简化
            }
        }
        stack.remove(&node);
    }
    
    for node in graph.all_nodes() {
        if !visited.contains(&node) {
            dfs(graph, node, &mut visited, &mut recursion_stack, &mut cycles);
        }
    }
    cycles
}
```

### 3. SQL + Graph 联合查询

```sql
-- 找出 SOP 涉及特定设备的所有步骤及依赖路径
WITH device_node AS (
    SELECT id FROM kg_nodes WHERE type = 'Device' AND name = 'Reactor1'
),
sop_node AS (
    SELECT id FROM kg_nodes WHERE type = 'SOP' AND name LIKE '%清洁验证%'
)
SELECT 
    n.id, n.type, n.name, n.description,
    e.relation_type,
    path
FROM kg_nodes n
JOIN kg_edges e ON n.id = e.dst_id
WHERE e.src_id IN (SELECT id FROM sop_node)
  AND e.dst_id IN (SELECT id FROM device_node)
ORDER BY e.weight DESC;

-- 向量 + 图联合查询
SELECT 
    d.title,
    ve.cosine_score AS doc_similarity,
    kg.path_to_device
FROM documents d
JOIN document_embeddings ve ON d.id = ve.doc_id
JOIN LATERAL (
    SELECT array_agg(name) as path_to_device
    FROM kg_nodes
    WHERE id IN (
        SELECT dst_id FROM kg_edges 
        WHERE src_id IN (
            SELECT id FROM kg_nodes WHERE name = d.title
        )
    )
) kg ON true
WHERE ve.query_embedding = :query_vec
ORDER BY ve.cosine_score DESC
LIMIT 10;
```

### 4. API

```json
// graph_query: 图查询
{
  "action": "graph_query",
  "start_node": "清洁验证 SOP",
  "relation": "depends_on",
  "depth": 3,
  "max_results": 50
}

// graph_path: 路径查找
{
  "action": "graph_path",
  "src": "原料入库",
  "dst": "成品出库",
  "algorithm": "shortest",  // "shortest" | "all_paths" | "weighted"
  "max_hops": 10
}

// graph_analyze: 实体分析
{
  "action": "graph_analyze",
  "entity": "Reactor1",
  "analysis_type": "influence",  // "influence" | "dependency" | "cluster"
  "depth": 2
}
```

---

## 📦 v2.5 全面集成 + GMP 内审

**Issue**: #1079  
**目标**: 整合所有模块，支持 OpenClaw 全局调度和 GMP 内审

### 1. 统一查询 API

```json
// unified_query: SQL + Vector + Graph 联合查询
{
  "action": "unified_query",
  "query": "最近批准的GMP清洁验证SOP涉及的设备和物料依赖",
  "mode": "sql_vector_graph",
  "sql_filters": {
    "doc_type": ["SOP"],
    "status": ["approved"]
  },
  "vector_field": "embedding",
  "graph_constraints": {
    "node_type": ["Device", "Material"],
    "relation_type": ["depends_on", "uses"]
  },
  "top_k": 10,
  "weights": {
    "sql": 0.3,
    "vector": 0.4,
    "graph": 0.3
  }
}

// automated_workflow: 自动化工作流
{
  "action": "automated_workflow",
  "workflow": "gmp_audit",
  "params": {
    "audit_scope": ["SOP", "规范"],
    "date_range": ["2025-01-01", "2026-03-01"],
    "generate_report": true
  }
}
```

### 2. OpenClaw 全局调度

```rust
// OpenClaw Agent 调度接口
pub struct SQLRustGoAgent {
    sql_executor: SqlExecutor,
    vector_store: VectorStore,
    graph_store: GraphStore,
    rag_pipeline: RAGPipeline,
}

impl SQLRustGoAgent {
    // 接收自然语言任务
    pub async fn handle_task(&self, task: &AgentTask) -> TaskResult {
        let intent = self.parse_intent(&task.description).await?;
        
        match intent {
            Intent::Query => {
                let results = self.execute_unified_query(&intent.query).await?;
                Ok(TaskResult::QueryResults(results))
            }
            Intent::RAG => {
                let answer = self.rag_pipeline.answer(&intent.question).await?;
                Ok(TaskResult::RAGAnswer(answer))
            }
            Intent::GraphAnalysis => {
                let analysis = self.graph_store.analyze(&intent.entity).await?;
                Ok(TaskResult::GraphAnalysis(analysis))
            }
            Intent::AuditReport => {
                let report = self.generate_audit_report(&intent.params).await?;
                Ok(TaskResult::Report(report))
            }
        }
    }
}
```

### 3. GMP 内审支持

```sql
-- 审计日志表
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    user_id TEXT,
    action_type ENUM('query', 'insert', 'update', 'delete', 'export') NOT NULL,
    table_name TEXT,
    record_id UUID,
    sql_query TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB
);

-- 审计报表表
CREATE TABLE audit_reports (
    id UUID PRIMARY KEY,
    report_type ENUM('daily', 'weekly', 'monthly', 'quarterly', 'annual') NOT NULL,
    period_start DATE,
    period_end DATE,
    content JSONB,
    generated_by TEXT,
    approved_by TEXT,
    status ENUM('draft', 'approved', 'archived') DEFAULT 'draft',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 合规检查记录
CREATE TABLE compliance_checks (
    id UUID PRIMARY KEY,
    check_type TEXT NOT NULL,  -- 'SOP_approval', 'doc_update', 'access_control'
    check_result ENUM('pass', 'fail', 'warning') NOT NULL,
    details JSONB,
    checked_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 4. API

```json
// generate_audit_report: 生成内审报告
{
  "action": "generate_audit_report",
  "report_type": "monthly",
  "period": {
    "start": "2026-02-01",
    "end": "2026-02-28"
  },
  "scope": {
    "departments": ["生产部", "质量部"],
    "doc_types": ["SOP", "规范", "审核"]
  },
  "include": {
    "compliance_checks": true,
    "change_history": true,
    "access_logs": true
  }
}

// compliance_check: 合规检查
{
  "action": "compliance_check",
  "check_type": "SOP_approval",
  "params": {
    "sop_id": "uuid-xxx",
    "required_approvers": 2
  }
}

// replay_audit: 审计回放
{
  "action": "replay_audit",
  "record_id": "uuid-yyy",
  "from": "2026-01-01",
  "to": "2026-03-01"
}
```

---

## 🧪 测试与集成方案

### 单元测试

| 模块 | 测试内容 | 覆盖率目标 |
|------|----------|------------|
| SQL Engine | 查询解析、执行、事务 | > 90% |
| Vector Store | 向量计算、索引、检索 | > 95% |
| Graph Store | BFS/DFS、路径算法 | > 90% |
| RAG Pipeline | 检索、生成、准确率 | > 85% |
| OpenClaw API | API 调用、认证、权限 | > 90% |

### 阶段性集成测试

| 阶段 | 测试内容 | 验收标准 |
|------|----------|----------|
| 2.1 | SQL + 向量检索联合 | 混合查询响应 < 500ms |
| 2.2 | KNN 性能与准确率 | Top-K 准确率 > 95% |
| 2.3 | RAG 问答准确率 | 回答准确率 > 90% |
| 2.4 | 知识图谱路径查询 | 路径查询 < 500ms |
| 2.5 | 全链路联合检验 | 端到端测试通过 |

### 性能基准

| 指标 | 2.1 | 2.2 | 2.3 | 2.4 | 2.5 |
|------|------|------|------|------|------|
| 检索延迟 | < 300ms | < 200ms | < 500ms | < 500ms | < 300ms |
| 向量吞吐 | 1k/s | 100k/s | - | - | 100k/s |
| 图路径查询 | - | - | - | < 500ms | < 500ms |
| RAG 准确率 | - | - | > 90% | - | > 95% |

---

## 👥 AI 开发线分配

| 开发线 | AI | 负责版本 | 主要任务 |
|--------|-----|----------|----------|
| Line A | OpenCode A | 2.1, 2.4, 2.5 | 文档表结构、图存储引擎、全模块整合 |
| Line B | OpenCode B | 2.1, 2.2 | 向量存储、并行计算、ColumnarStorage |
| Line C | Claude A | 2.2, 2.3 | 向量索引优化、RAG 问答、全文检索 |
| Line D | Claude B | 2.3, 2.4, 2.5 | 知识库管理、图检索算法、OpenClaw 调度、GMP 内审 |

---

## 📊 Issue 追踪

| Issue | 版本 | 标题 | 状态 |
|-------|------|------|------|
| #1075 | v2.1 | GMP文档精准检索原型 | OPEN |
| #1076 | v2.2 | 高性能向量数据库 | OPEN |
| #1077 | v2.3 | RAG + 全文检索 + 知识库 | OPEN |
| #1078 | v2.4 | 知识图谱 + 图检索 | OPEN |
| #1079 | v2.5 | 全面集成 + GMP内审 | OPEN |
| #1080 | v2.x | v2.1-v2.5 开发总控 | OPEN |

---

*文档版本: 1.0*  
*最后更新: 2026-03-28*
