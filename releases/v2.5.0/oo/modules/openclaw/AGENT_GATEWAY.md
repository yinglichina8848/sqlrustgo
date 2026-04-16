# OpenClaw Agent 网关设计文档

**版本**: v2.5.0

---

## 概述

OpenClaw Agent 网关提供 AI Agent 与 SQLRustGo 数据库的桥梁接口。

## 架构

### 核心组件

```
┌─────────────────────────────────────────────────────────┐
│                  OpenClawHttpServer                   │
├─────────────────────────────────────────────────────────┤
│  AppState                                             │
│  - SchemaService                                      │
│  - StatsService                                       │
│  - Nl2SqlService                                      │
│  - MemoryService                                      │
│  - PolicyEngine                                       │
│  - ColumnMasker                                       │
│  - ExplainService                                     │
│  - OptimizerService                                   │
└─────────────────────────────────────────────────────────┘
```

## 服务定义

### SchemaService

```rust
pub struct SchemaService {
    // 获取表结构
    pub fn get_table(&self, table_name: &str) -> TableSchema;
    
    // 获取所有表
    pub fn list_tables(&self) -> Vec<TableInfo>;
    
    // 获取列信息
    pub fn get_columns(&self, table_name: &str) -> Vec<ColumnSchema>;
}
```

### StatsService

```rust
pub struct StatsService {
    // 获取查询统计
    pub fn get_query_stats(&self) -> QueryStats;
    
    // 获取表统计
    pub fn get_table_stats(&self, table_name: &str) -> TableStats;
    
    // 获取性能指标
    pub fn get_performance_metrics(&self) -> PerformanceMetrics;
}
```

## API 端点

### 健康检查

```
GET /health
```

### 查询

```
POST /query
{
    "sql": "SELECT * FROM users WHERE id = 1"
}
```

### 自然语言查询

```
POST /nl_query
{
    "query": "查找年龄大于30的用户"
}
```

### Schema

```
GET /schema
GET /schema/{table_name}
```

### 统计

```
GET /stats
GET /stats/queries
GET /stats/tables
```

### 内存

```
POST /memory/save
GET /memory/load/{key}
POST /memory/search
DELETE /memory/clear
```

### 执行计划

```
POST /explain
{
    "sql": "SELECT * FROM users"
}
```

## 统一查询集成

```rust
// execute_unified_query 示例
pub async fn execute_unified_query(
    state: &AppState,
    req: UnifiedQueryRequest,
) -> Result<UnifiedQueryResponse, Error> {
    let mode = req.mode.as_str();
    
    let sql_results = if mode.contains("sql") {
        Some(execute_sql(state, &req).await?)
    } else { None };
    
    let vector_results = if mode.contains("vector") {
        Some(execute_vector(state, &req).await?)
    } else { None };
    
    let graph_results = if mode.contains("graph") {
        Some(execute_graph(state, &req).await?)
    } else { None };
    
    Ok(UnifiedQueryResponse {
        sql_results,
        vector_results,
        graph_results,
        execution_time: elapsed,
    })
}
```

## 配置

```toml
[openclaw]
host = "0.0.0.0"
port = 8080
max_connections = 100

[openclaw.rag]
enabled = true
model = "gpt-4"

[openclaw.security]
column_masking = true
query_validation = true
```

---

*文档版本: 1.0*
*最后更新: 2026-04-16*