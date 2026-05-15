# OO-5: Provenance Tracking 设计文档

> **版本**: v1.0
> **日期**: 2026-05-16
> **基于**: v3.2.0
> **维护人**: hermes-z6g4
> **状态**: 设计中

---

## 一、概述

### 1.1 目标

实现数据溯源追踪系统 (Provenance Tracking)，追踪数据的完整生命周期：

- **血缘关系**: 记录数据从产生到当前状态的全过程
- **来源追溯**: 查询任何数据记录的来源
- **变换追踪**: 记录数据的所有转换操作
- **可视化支持**: 提供血缘关系可视化

### 1.2 核心理念

```
Provenance = Data Lineage + Transformation History + Source Tracking
```

---

## 二、技术架构

### 2.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                  Provenance Tracking System                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Lineage   │───▶│  Transform   │───▶│   Source        │  │
│  │   Graph     │    │   Tracker    │    │   Tracker       │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Graph     │    │   Transform  │    │   Source        │  │
│  │   Database  │    │   Operations │    │   Tables        │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Query Engine                                 │   │
│  │  - Lineage Queries                                       │   │
│  │  - Impact Analysis                                       │   │
│  │  - Dependency Graph                                       │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、数据结构

### 3.1 数据来源表 (gmp_data_provenance)

```sql
CREATE TABLE gmp_data_provenance (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name          TEXT NOT NULL,
    record_id           TEXT NOT NULL,
    operation           TEXT NOT NULL,           -- INSERT/SELECT/UPDATE/DELETE
    source_table        TEXT,                     -- 源表（用于 JOIN/INSERT SELECT）
    source_record_ids   JSONB,                   -- 源记录 ID 列表
    transformation_type TEXT,                    -- DERIVED/COMPUTED/IMPORTED
    transformation_desc  TEXT,                    -- 转换描述
    operator_id         TEXT,                    -- 操作人
    input_data          JSONB,                   -- 输入数据快照
    output_data         JSONB,                   -- 输出数据快照
    lineage_depth       INT DEFAULT 0,
    parent_provenance_id UUID,                   -- 父节点
    root_provenance_id  UUID,                   -- 根节点
    timestamp           BIGINT NOT NULL,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

### 3.2 血缘关系图表 (gmp_lineage_graph)

```sql
CREATE TABLE gmp_lineage_graph (
    id                  BIGSERIAL PRIMARY KEY,
    provenance_id       UUID NOT NULL REFERENCES gmp_data_provenance(id),
    parent_id           UUID REFERENCES gmp_data_provenance(id),
    edge_type           TEXT NOT NULL,           -- DERIVES_FROM/COPY_OF/JOIN
    edge_weight         FLOAT DEFAULT 1.0,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

---

## 四、血缘模型

### 4.1 血缘类型

| 类型 | 说明 | 示例 |
|------|------|------|
| DIRECT | 直接插入 | INSERT INTO t VALUES (...) |
| DERIVED | 派生数据 | SELECT col1 + col2 FROM t |
| IMPORTED | 导入数据 | COPY FROM / IMPORT |
| JOINED | 连接产生 | SELECT * FROM t1 JOIN t2 |
| AGGREGATED | 聚合产生 | SELECT SUM(col) FROM t |

### 4.2 血缘深度

```rust
/// 计算血缘深度
fn compute_lineage_depth(provenance: &ProvenanceRecord) -> i32 {
    if let Some(parent_id) = provenance.parent_provenance_id {
        let parent = get_provenance(parent_id);
        parent.lineage_depth + 1
    } else {
        0
    }
}
```

---

## 五、API 设计

### 5.1 核心 Trait: `ProvenanceProvider`

```rust
/// 数据溯源提供者接口
pub trait ProvenanceProvider {
    /// 记录数据溯源
    fn record_provenance(
        &self,
        record: &ProvenanceRecord,
    ) -> Result<Uuid, Error>;

    /// 查询数据来源
    fn query_source(
        &self,
        table_name: &str,
        record_id: &str,
    ) -> Result<Vec<ProvenanceRecord>, Error>;

    /// 查询数据影响
    fn query_impact(
        &self,
        table_name: &str,
        record_id: &str,
    ) -> Result<Vec<ProvenanceRecord>, Error>;

    /// 构建血缘图
    fn build_lineage_graph(
        &self,
        table_name: &str,
        record_id: &str,
    ) -> Result<LineageGraph, Error>;

    /// 分析数据依赖
    fn analyze_dependencies(
        &self,
        table_name: &str,
        record_id: &str,
    ) -> Result<DependencyAnalysis, Error>;
}
```

### 5.2 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub id: Uuid,
    pub table_name: String,
    pub record_id: String,
    pub operation: Operation,
    pub source_table: Option<String>,
    pub source_record_ids: Vec<String>,
    pub transformation_type: TransformationType,
    pub transformation_desc: Option<String>,
    pub operator_id: String,
    pub input_data: Option<JsonValue>,
    pub output_data: JsonValue,
    pub lineage_depth: i32,
    pub parent_provenance_id: Option<Uuid>,
    pub root_provenance_id: Option<Uuid>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageGraph {
    pub root: Uuid,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub total_depth: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub provenance_id: Uuid,
    pub table_name: String,
    pub record_id: String,
    pub operation: String,
    pub depth: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub edge_type: String,
}
```

---

## 六、SQL 语句支持

### 6.1 查询数据来源

```sql
-- 查询记录的血缘关系
SELECT * FROM gmp_data_provenance
WHERE table_name = 'report_table'
AND record_id = 'report-001'
ORDER BY lineage_depth;

-- 查询数据来源链
SELECT GET_LINEAGE_CHAIN('orders', '12345');

-- 查询数据影响链
SELECT GET_IMPACT_CHAIN('orders', '12345');
```

### 6.2 血缘分析

```sql
-- 分析依赖关系
SELECT ANALYZE_DEPENDENCIES('derived_table', '聚合记录');

-- 验证数据来源
SELECT VERIFY_PROVENANCE('sensitive_data', '123');
```

---

## 七、实现状态

| 阶段 | 任务 | 状态 | PR |
|------|------|------|-----|
| 1 | 数据结构定义 | ✅ | #1024 |
| 2 | 溯源记录 | ✅ | #1024 |
| 3 | 血缘图构建 | ✅ | #1024 |
| 4 | 查询 API | ✅ | #1024 |

---

## 八、依赖

- GMP-1: 审计链 (已完成)
- GMP-5: Provenance Tracking (进行中)

---

*本文档由 hermes-agent 创建*
*版本 1.0 - 2026-05-16*