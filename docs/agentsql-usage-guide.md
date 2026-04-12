# AgentSQL Extension 使用文档

## 概述

AgentSQL Extension 是 SQLRustGo 的 AI Agent 数据库访问层，为 OpenClaw 等 AI Agent 框架提供自然语言查询、Schema introspection、内存管理和安全策略等能力。

**版本:** 2.1.0  
**Issue:** #1128

---

## 模块列表

| 模块 | 文件 | 描述 |
|------|------|------|
| Gateway | `gateway.rs` | HTTP API 入口和路由 |
| Schema | `schema.rs` | 数据库 Schema introspection |
| Stats | `stats.rs` | 表统计信息 API |
| NL2SQL | `nl2sql.rs` | 自然语言转 SQL |
| Memory | `memory.rs` | Agent 上下文记忆 |
| Policy Engine | `policy_engine.rs` | RBAC 策略引擎 |
| Column Masking | `column_masking.rs` | 列级数据脱敏 |
| Explain | `explain.rs` | SQL 执行计划解释 |
| Optimizer | `optimizer.rs` | SQL 优化建议 |

---

## API 端点

### 核心端点

| 端点 | 方法 | 功能 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/query` | POST | SQL 执行 |
| `/nl_query` | POST | 自然语言查询 |
| `/schema` | GET | 获取完整 Schema |
| `/schema/:table` | GET | 获取指定表 Schema |
| `/stats` | GET | 获取所有表统计 |
| `/stats/:table` | GET | 获取指定表统计 |
| `/stats/queries` | GET | 获取查询统计 |

### Memory 端点

| 端点 | 方法 | 功能 |
|------|------|------|
| `/memory/save` | POST | 保存记忆 |
| `/memory/load` | POST | 加载记忆 |
| `/memory/search` | POST | 搜索记忆 |
| `/memory/clear` | POST | 清除记忆 |
| `/memory/stats` | GET | 获取记忆统计 |

### Security 端点

| 端点 | 方法 | 功能 |
|------|------|------|
| `/policy/check` | POST | 权限检查 |
| `/mask` | POST | 数据脱敏 |

### Explain & Optimize 端点

| 端点 | 方法 | 功能 |
|------|------|------|
| `/explain` | POST | SQL 执行计划 (旧版) |
| `/explain/new` | POST | SQL 执行计划 (详细) |
| `/optimize` | POST | SQL 优化建议 |

---

## Policy Engine

### 概述

Policy Engine 实现了基于角色的访问控制 (RBAC)，支持 Allow/Deny 策略和条件判断。

### 核心类型

```rust
// 策略
pub struct Policy {
    pub id: String,
    pub name: String,
    pub resource: String,      // e.g., "table:*", "table:users"
    pub actions: Vec<String>,  // e.g., ["SELECT", "INSERT"]
    pub conditions: Vec<PolicyCondition>,
    pub effect: PolicyEffect,  // Allow or Deny
}

// 条件操作符
pub enum ConditionOperator {
    Eq, Ne, Gt, Lt, Gte, Lte,
    In, NotIn, Like, IsNull, IsNotNull,
}

// 策略效果
pub enum PolicyEffect {
    Allow,
    Deny,
}

// 权限检查请求
pub struct PolicyCheckRequest {
    pub user_id: String,
    pub resource: String,
    pub action: String,
    pub context: Option<HashMap<String, serde_json::Value>>,
}
```

### 使用示例

```rust
use sqlrustgo_agentsql::policy_engine::{PolicyEngine, PolicyCheckRequest};

let engine = PolicyEngine::new();

// 检查权限
let request = PolicyCheckRequest {
    user_id: "user1".to_string(),
    resource: "table:users".to_string(),
    action: "SELECT".to_string(),
    context: None,
};

let response = engine.check(&request);
println!("Allowed: {}", response.allowed);
println!("Matched Policy: {:?}", response.matched_policy);
```

### 默认策略

| Policy ID | Resource | Action | Effect | Conditions |
|-----------|----------|--------|--------|------------|
| policy_read_all | table:* | SELECT | Allow | - |
| policy_write_restricted | table:sensitive_data | INSERT, UPDATE, DELETE | Allow | user_role = "admin" |
| policy_deny_delete | table:audit_log | DELETE | Deny | - |

---

## Column Masking

### 概述

Column Masking 提供了列级数据脱敏功能，支持多种脱敏类型。

### 脱敏类型

| 类型 | 描述 | 示例 |
|------|------|------|
| Full | 完全脱敏 | "secret" → "****" |
| Partial | 部分脱敏 | "john@example.com" → "j***@example.com" |
| Hash | 哈希脱敏 | "123-45-6789" → "a1b2c3d4" |
| Truncate | 截断脱敏 | "secret_data" → "sec" |
| Null | 置空 | "secret" → null |
| Range | 范围脱敏 | 75000 → "50K - 100K" |

### 使用示例

```rust
use sqlrustgo_agentsql::column_masking::{ColumnMasker, MaskingType, MaskingRule, MaskingConfig};

// 使用默认配置
let masker = ColumnMasker::new();
let masked = masker.mask_value("email", &serde_json::json!("user@example.com"));
println!("Masked email: {}", masked);

// 使用自定义配置
let custom_rules = vec![MaskingRule {
    id: "mask_custom".to_string(),
    column: "secret".to_string(),
    mask_type: MaskingType::Full,
    description: "Fully mask secret".to_string(),
}];
let config = MaskingConfig { rules: custom_rules };
let masker = ColumnMasker::new_with_config(config);
```

### 预配置规则

| Column Pattern | Type | Example |
|----------------|------|---------|
| email | Partial | "j***@example.com" |
| phone | Partial | "***-***-5678" |
| ssn | Hash | "a1b2c3d4" |
| credit_card | Full | "****" |
| salary | Range | "50K - 100K" |

---

## Explain

### 概述

Explain 服务提供 SQL 执行计划的分析和解释。

### 使用示例

```rust
use sqlrustgo_agentsql::explain::{ExplainService, ExplainOptions, ExplainFormat};

let service = ExplainService::new();

// 基本使用
let result = service.explain("SELECT * FROM users WHERE id = 1");
println!("Plan: {:?}", result.plan);
println!("Cost: {}", result.estimated_cost);
println!("Rows: {}", result.estimated_rows);
println!("Warnings: {:?}", result.warnings);

// JSON 格式
let json = service.explain_json("SELECT * FROM users");

// 文本格式
let text = service.explain_text("SELECT * FROM users WHERE id = 1");
println!("{}", text);

// 自定义选项
let options = ExplainOptions {
    format: ExplainFormat::Json,
    verbose: true,
    analyze: true,
};
let service = ExplainService::new_with_options(options);
```

### 输出结构

```json
{
  "plan": {
    "node_type": "Select",
    "table": "users",
    "operation": "Select",
    "cost": 10.5,
    "rows": 100,
    "children": [...]
  },
  "warnings": ["Large result set detected. Consider adding LIMIT clause."],
  "estimated_cost": 10.5,
  "estimated_rows": 100
}
```

---

## Optimizer

### 概述

Optimizer 服务提供 SQL 优化建议。

### 优化规则

| Rule ID | Category | Priority | Description |
|---------|----------|----------|-------------|
| add_limit | QueryRewrite | High | 建议添加 LIMIT 子句 |
| avoid_select_star | QueryRewrite | Medium | 建议避免 SELECT * |
| use_index | Index | High | 建议使用索引 |
| optimize_join_order | Join | Medium | 建议优化 JOIN 顺序 |
| use_explicit_join | Join | Low | 建议使用显式 JOIN 语法 |

### 使用示例

```rust
use sqlrustgo_agentsql::optimizer::{OptimizerService, SuggestionCategory, Priority};

let optimizer = OptimizerService::new();

// 优化 SQL
let result = optimizer.optimize("SELECT * FROM users");
println!("Original: {}", result.original_sql);
println!("Optimized: {}", result.optimized_sql);

// 查看建议
for suggestion in &result.suggestions {
    println!("[{:?}] {}: {}", suggestion.priority, suggestion.title, suggestion.description);
}

// 性能预估
println!("Before: {}ms", result.estimated_improvement.before_ms);
if let Some(after) = result.estimated_improvement.after_ms {
    println!("After: {}ms", after);
}

// 分析 SQL
let suggestions = optimizer.analyze("SELECT * FROM users WHERE id = 1");
```

### 输出结构

```json
{
  "original_sql": "SELECT * FROM users",
  "optimized_sql": "SELECT * FROM users LIMIT 100",
  "suggestions": [
    {
      "id": "add_limit",
      "category": "query_rewrite",
      "priority": "high",
      "title": "Add LIMIT clause",
      "description": "Query does not have a LIMIT clause...",
      "estimated_savings": "50-90%"
    }
  ],
  "estimated_improvement": {
    "before_ms": 100.0,
    "after_ms": 10.0,
    "improvement_percent": 90.0
  }
}
```

---

## Gateway 服务启动

### 使用示例

```rust
use sqlrustgo_agentsql::gateway::{start_server, create_router, AppState};
use std::sync::Arc;
use parking_lot::RwLock;
use sqlrustgo_agentsql::{SchemaService, StatsService, MemoryService};
use sqlrustgo_agentsql::nl2sql::Nl2SqlService;
use sqlrustgo_agentsql::policy_engine::PolicyEngine;
use sqlrustgo_agentsql::column_masking::ColumnMasker;
use sqlrustgo_agentsql::explain::ExplainService;
use sqlrustgo_agentsql::optimizer::OptimizerService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server(8080).await
}
```

### 直接创建 Router

```rust
let schema_service = Arc::new(SchemaService::new());
let stats_service = Arc::new(StatsService::new());
let nl2sql_service = Arc::new(Nl2SqlService::new(schema_service.clone()));
let memory_service = Arc::new(RwLock::new(MemoryService::new()));
let policy_engine = Arc::new(PolicyEngine::new());
let column_masker = Arc::new(ColumnMasker::new());
let explain_service = Arc::new(ExplainService::new());
let optimizer_service = Arc::new(OptimizerService::new());

let state = AppState {
    schema_service,
    stats_service,
    nl2sql_service,
    memory_service,
    policy_engine,
    column_masker,
    explain_service,
    optimizer_service,
};

let app = create_router(state);
```

---

## HTTP API 调用示例

### cURL

```bash
# 健康检查
curl http://localhost:8080/health

# Schema introspection
curl http://localhost:8080/schema
curl http://localhost:8080/schema/users

# 自然语言查询
curl -X POST http://localhost:8080/nl_query \
  -H "Content-Type: application/json" \
  -d '{"query": "show all users"}'

# Policy 检查
curl -X POST http://localhost:8080/policy/check \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user1", "resource": "table:users", "action": "SELECT"}'

# 数据脱敏
curl -X POST http://localhost:8080/mask \
  -H "Content-Type: application/json" \
  -d '{"column": "email", "value": "user@example.com"}'

# SQL 执行计划
curl -X POST http://localhost:8080/explain/new \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 1"}'

# SQL 优化
curl -X POST http://localhost:8080/optimize \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users"}'

# Memory 保存
curl -X POST http://localhost:8080/memory/save \
  -H "Content-Type: application/json" \
  -d '{"content": "User asked about orders", "agent_id": "agent-123"}'

# Memory 加载
curl -X POST http://localhost:8080/memory/load \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "agent-123"}'
```

---

## OpenClaw Extension

AgentSQL Extension 与 OpenClaw Agent 框架集成，提供以下工具：

| Tool | Description |
|------|-------------|
| agentsql_query | SQL 查询执行 |
| agentsql_nl_query | 自然语言转 SQL |
| agentsql_schema | Schema introspection |
| agentsql_stats | 表统计信息 |
| agentsql_explain | 执行计划解释 |
| agentsql_optimize | SQL 优化建议 |
| agentsql_policy_check | 权限检查 |
| agentsql_mask | 数据脱敏 |
| agentsql_memory | 记忆管理 |

详细文档请参考 `extensions/openclaw/skill.yaml`。

---

## 测试

### 运行单元测试

```bash
cd crates/agentsql
cargo test
```

### 运行集成测试

```bash
cargo test --test agentsql_test
```

### 运行回归测试

```bash
cargo test --test regression_test -- --nocapture
```

---

## 相关 Issue

- Issue #1128: AgentSQL Extension - OpenClaw原生AI数据库接口
