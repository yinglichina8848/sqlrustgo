# OpenClaw × SQLRustGo v2.1 原生集成架构设计

> **日期**: 2026-03-29
> **版本**: v1.0
> **类型**: 架构设计文档
> **状态**: 正式进入 v2.1 Roadmap

---

## 一、总体目标架构

### 1.1 架构全景

```
┌─────────────────────────────────────────────────────────────────┐
│                      OpenClaw Runtime                           │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      Agent Skill Layer                          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                  AgentSQL Gateway (新增)                         │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │ Gateway │ │ Schema  │ │ NL2SQL  │ │ Memory  │ │ Security│  │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      SQLRustGo Engine                           │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐              │
│  │ Parser  │ │Optimizer│ │Executor │ │ Storage │              │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘              │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      Storage Layer                               │
│                   (ColumnarStorage)                             │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 能力升级对比

| 能力 | v1.9 | v2.1 |
|------|------|------|
| SQL执行 | ✅ | ✅ |
| Schema introspection | ❌ | ✅ |
| NL2SQL | ❌ | ✅ |
| Agent memory | ❌ | ✅ |
| Policy control | ❌ | ✅ |
| Explain feedback | ❌ | ✅ |
| Embedding schema | ❌ | ✅ |

### 1.3 定位

> **SQLRustGo v2.1 = Agent-native 数据平台核心引擎**
>
> 可作为企业 AI 员工系统的数据底座

---

## 二、为什么必须增加 AgentSQL Gateway

### 2.1 问题分析

| 问题 | 原因 | 影响 |
|------|------|------|
| OpenClaw不直接连接数据库 | 设计如此 | 需要中间层 |
| Agent无法理解schema | 数据库不暴露 | 产生hallucination |
| Agent无法优化SQL | 没有反馈机制 | 性能差 |
| Agent无法共享上下文 | 无记忆服务 | 分析不连续 |
| Agent权限无法控制 | 无安全层 | 数据风险 |

### 2.2 正确架构

```
Agent → Tool → Gateway → Database
```

### 2.3 核心价值

| 价值 | 说明 |
|------|------|
| 安全访问 | 通过Gateway代理访问 |
| Schema感知 | Agent理解数据库结构 |
| 智能优化 | Explain反馈循环 |
| 上下文共享 | Agent Memory Service |
| 权限隔离 | Policy Engine |

---

## 三、AgentSQL Gateway 模块结构

### 3.1 目录结构

```
sqlrustgo/
└── agentsql/
    ├── gateway/          # HTTP入口，OpenClaw Tool接口
    ├── schema/          # Schema查询和graph服务
    ├── nl2sql/          # 自然语言转SQL
    ├── optimizer/        # 查询优化建议
    ├── explain/         # 执行计划解释
    ├── stats/           # 表统计信息
    ├── memory/          # Agent上下文记忆
    └── security/        # 权限控制和策略
```

### 3.2 模块职责

| 模块 | 功能 |
|------|------|
| `gateway` | HTTP入口，OpenClaw Tool接口 |
| `schema` | Schema查询、关系图谱 |
| `nl2sql` | 自然语言SQL生成 |
| `optimizer` | 查询优化建议 |
| `explain` | 执行计划解释 |
| `stats` | 表统计信息 |
| `memory` | Agent上下文记忆 |
| `security` | 权限控制、策略引擎 |

### 3.3 REST API 端点

| API | 方法 | 功能 |
|-----|------|------|
| `/agentsql/schema` | GET | Schema introspection |
| `/agentsql/schema_graph` | GET | Schema关系图谱 |
| `/agentsql/query` | POST | SQL执行 |
| `/agentsql/nl_query` | POST | 自然语言查询 |
| `/agentsql/explain` | POST | 查询计划解释 |
| `/agentsql/optimize` | POST | SQL优化建议 |
| `/agentsql/stats` | GET | 表统计 |
| `/agentsql/memory/save` | POST | 保存记忆 |
| `/agentsql/memory/load` | GET | 加载记忆 |
| `/agentsql/schema_embedding` | GET | 向量化Schema |
| `/agentsql/policy/check` | POST | 权限检查 |

---

## 四、OpenClaw 与 SQLRustGo 通信协议

### 4.1 协议设计

| 协议 | 说明 |
|------|------|
| **Primary** | REST + JSON |
| **Future** | MCP Protocol (Model Context Protocol) |

### 4.2 调用流程

```
OpenClaw Tool
     ↓
HTTP POST
     ↓
/agentsql/*
     ↓
AgentSQL Gateway
     ↓
SQLRustGo Engine
```

### 4.3 请求/响应示例

**自然语言查询**：

```json
POST /agentsql/nl_query

请求：
{
  "question": "统计最近7天订单增长趋势"
}

响应：
{
  "sql": "SELECT DATE(created_at) as day, COUNT(*) as orders FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 7 DAY) GROUP BY day ORDER BY day",
  "rows": [
    {"day": "2026-03-23", "orders": 156},
    {"day": "2026-03-24", "orders": 178},
    {"day": "2026-03-25", "orders": 203}
  ],
  "summary": "最近7天订单呈上升趋势，最高203单/天",
  "confidence": 0.92
}
```

---

## 五、Agent Schema Service

### 5.1 核心能力

Agent第一步必须理解数据库结构

### 5.2 Schema Graph Service

```http
GET /agentsql/schema_graph
```

**响应**：
```json
{
  "tables": [
    {
      "name": "users",
      "columns": [
        {"name": "id", "type": "int", "primary_key": true},
        {"name": "name", "type": "varchar(100)"},
        {"name": "email", "type": "varchar(255)"},
        {"name": "created_at", "type": "timestamp"}
      ]
    },
    {
      "name": "orders",
      "columns": [
        {"name": "id", "type": "int", "primary_key": true},
        {"name": "user_id", "type": "int", "foreign_key": "users.id"},
        {"name": "amount", "type": "decimal(10,2)"},
        {"name": "created_at", "type": "timestamp"}
      ]
    }
  ],
  "foreign_keys": [
    {"from": "orders.user_id", "to": "users.id", "relation": "1:N"}
  ],
  "relations": [
    {"table": "users", "related": "orders", "type": "has_many"}
  ]
}
```

### 5.3 用途

| 用途 | 说明 |
|------|------|
| 自动推断join | Agent理解表关系 |
| 理解业务结构 | 语义层 |
| 减少hallucination | 避免生成错误SQL |

---

## 六、Agent Memory Service

### 6.1 核心创新

传统数据库没有的能力：数据库级别short-term memory

### 6.2 接口设计

**保存记忆**：

```http
POST /agentsql/memory/save
```

**请求**：
```json
{
  "session_id": "analysis001",
  "memory": "用户增长异常来自渠道A，上午10点流量突增"
}
```

**响应**：
```json
{
  "status": "ok",
  "memory_id": "mem_001"
}
```

**加载记忆**：

```http
GET /agentsql/memory/load?session_id=analysis001
```

**响应**：
```json
{
  "session_id": "analysis001",
  "memories": [
    {"id": "mem_001", "content": "用户增长异常来自渠道A，上午10点流量突增", "timestamp": "2026-03-29T10:15:00Z"},
    {"id": "mem_002", "content": "订单量下降与营销活动结束相关", "timestamp": "2026-03-29T10:20:00Z"}
  ]
}
```

### 6.3 用途

| 用途 | 场景 |
|------|------|
| 跨SQL分析 | 多步数据分析 |
| 多轮Agent推理 | 复杂分析任务 |
| 长期数据分析 | 趋势分析 |

---

## 七、Policy Engine

### 7.1 模块位置

```
agentsql/security/
├── policy_engine.rs
├── rbac.rs
├── column_masking.rs
└── audit.rs
```

### 7.2 能力矩阵

| 能力 | 说明 |
|------|------|
| Row-level security | 行级安全 |
| Column masking | 列加密/脱敏 |
| Table policy | 表访问策略 |
| Agent role policy | Agent角色权限 |

### 7.3 策略配置示例

```yaml
# agentsql_policy.yaml
agent_roles:
  analyst:
    description: "数据分析员"
    allow:
      - SELECT users.name
      - SELECT users.email
    deny:
      - users.password
      - users.phone
    max_rows: 1000

  operator:
    description: "数据操作员"
    allow:
      - SELECT
      - INSERT
      - UPDATE
    deny:
      - DELETE
      - users.salary

  admin:
    description: "管理员"
    allow:
      - ALL

table_policies:
  orders:
    analyst: read_only
    operator: full_access

column_policies:
  users.phone:
    analyst: mask   # 返回 ****
  users.password:
    all: deny
```

### 7.4 GMP合规支持

对于制药企业场景：

```yaml
gmp_compliance:
  audit_trail:
    enabled: true
    retention_days: 1825  # 5年

  data_masking:
    batch_records.quality_data: mandatory
    batch_records.responsible_person: track_only
```

---

## 八、Query Explain Feedback Loop

### 8.1 优化流程

```
Agent生成SQL
     ↓
POST /agentsql/explain
     ↓
返回执行计划 + 优化建议
     ↓
Agent改写SQL
     ↓
POST /agentsql/optimize
     ↓
返回优化后SQL
     ↓
Agent执行优化SQL
```

### 8.2 Explain接口

```http
POST /agentsql/explain
```

**请求**：
```json
{
  "sql": "SELECT * FROM users, orders WHERE users.id = orders.user_id"
}
```

**响应**：
```json
{
  "plan": {
    "operation": "Hash Join",
    "estimated_cost": 145.5,
    "estimated_rows": 100000,
    "table_stats": [
      {"table": "users", "rows": 50000, "scan_type": "seq_scan"},
      {"table": "orders", "rows": 500000, "scan_type": "seq_scan"}
    ],
    "warnings": [
      "orders.user_id 上没有索引",
      "建议创建复合索引"
    ]
  },
  "suggestions": [
    {
      "type": "index",
      "sql": "CREATE INDEX idx_orders_user_id ON orders(user_id)",
      "impact": "提升90%性能"
    },
    {
      "type": "sql_rewrite",
      "original": "SELECT *",
      "rewritten": "SELECT users.id, users.name, orders.amount",
      "reason": "减少数据传输量"
    }
  ]
}
```

### 8.3 Optimize接口

```http
POST /agentsql/optimize
```

**请求**：
```json
{
  "sql": "SELECT * FROM users WHERE id IN (SELECT user_id FROM orders)"
}
```

**响应**：
```json
{
  "optimized_sql": "SELECT u.id, u.name FROM users u INNER JOIN orders o ON u.id = o.user_id",
  "reason": "IN子查询改写为JOIN，效率提升5倍",
  "estimated_improvement": "80%"
}
```

---

## 九、Schema Embedding Service

### 9.1 核心能力

将Schema向量化，支持语义搜索

### 9.2 接口

```http
GET /agentsql/schema_embedding
```

**响应**：
```json
{
  "embeddings": [
    {
      "table": "users",
      "description": "用户信息表，包含用户基本信息",
      "embedding": [0.123, -0.456, 0.789, ...],
      "columns": [
        {
          "name": "created_at",
          "description": "用户注册时间",
          "embedding": [0.111, -0.222, 0.333, ...]
        }
      ]
    }
  ]
}
```

### 9.3 用途

| 用途 | 说明 |
|------|------|
| Semantic table search | 语义搜索表 |
| Semantic column match | 语义匹配列 |
| 自动SQL生成 | 基于描述生成SQL |
| 自动join推断 | 推断表关系 |

### 9.4 使用示例

Agent输入："查用户注册时间"

系统自动匹配：
- Table: `users`
- Column: `created_at`
- Description: "用户注册时间"

无需人工提示，Agent自动理解。

---

## 十、OpenClaw Skill 官方集成

### 10.1 目录结构

```
openclaw/extensions/sqlrustgo/
├── sqlrustgo-skill.yaml
├── sqlrustgo-client.ts
├── sqlrustgo-client.py
└── README.md
```

### 10.2 Skill定义

```yaml
# sqlrustgo-skill.yaml
name: sqlrustgo_query
version: "1.0"
description: Query SQLRustGo database using natural language or SQL

provider:
  type: http
  endpoint: "{{SQLRUSTGO_URL}}/agentsql"
  timeout: 30000

tools:
  - name: nl_query
    endpoint: /nl_query
    method: POST
    description: "执行自然语言查询"

  - name: sql_query
    endpoint: /query
    method: POST
    description: "执行SQL查询"

  - name: get_schema
    endpoint: /schema
    method: GET
    description: "获取数据库Schema"

  - name: get_stats
    endpoint: /stats
    method: GET
    description: "获取表统计信息"

security:
  api_key: "{{SQLRUSTGO_API_KEY}}"
  policy: "{{POLICY_FILE}}"
```

### 10.3 Agent调用示例

```python
# 在OpenClaw Agent中
agent = Agent("data_analyst")

# 自然语言查询
result = await agent.call("sqlrustgo_query", {
    "question": "最近7天新增用户"
})

# 返回
# {
#   "sql": "SELECT COUNT(*) FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 7 DAY)",
#   "result": [{"count": 1234}],
#   "summary": "最近7天新增1234个用户"
# }
```

---

## 十一、企业级 Agent Data Flow

### 11.1 完整数据流

```
┌─────────────────────────────────────────────────────────────────┐
│                         User / Employee                          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      OpenClaw Agent                             │
│   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐            │
│   │ Memory  │ │ Skills  │ │ Tools   │ │ LLM     │            │
│   └─────────┘ └─────────┘ └─────────┘ └─────────┘            │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      Skill Router                               │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    AgentSQL Gateway                             │
│   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│   │ Gateway │ │ Schema  │ │ NL2SQL  │ │ Memory  │ │ Security│  │
│   └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      SQLRustGo Engine                           │
│   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐              │
│   │ Parser  │ │Optimizer│ │Executor │ │ Storage │              │
│   └─────────┘ └─────────┘ └─────────┘ └─────────┘              │
└─────────────────────────────────────────────────────────────────┘
```

### 11.2 支持的应用场景

| 场景 | 说明 |
|------|------|
| BI | 商业智能分析 |
| RAG | 检索增强生成 |
| 知识图谱 | 图数据库查询 |
| 审计分析 | 合规审计 |
| 运营分析 | KPI监控 |
| 自动报告 | 报告生成 |

---

## 十二、v2.1 模块规模估算

### 12.1 代码规模

| 模块 | LOC | 说明 |
|------|-----|------|
| `gateway` | 1200 | HTTP入口，路由 |
| `schema` | 800 | Schema查询，graph |
| `stats` | 600 | 统计信息 |
| `memory` | 700 | 记忆服务 |
| `security` | 1500 | 策略引擎 |
| `nl2sql` | 2000 | NL2SQL服务 |
| `optimizer` | 1200 | 查询优化建议 |
| `embedding` | 800 | Schema向量化 |

**总计**：约 **8,000 LOC**

### 12.2 开发周期

| Phase | 内容 | 周期 |
|-------|------|------|
| **Phase 1** | gateway, schema, stats | 2周 |
| **Phase 2** | nl2sql, memory, security | 3周 |
| **Phase 3** | optimizer, embedding | 3周 |

**总周期**：约 **8周**

---

## 十三、Agent-aware Query Engine（v2.1旗舰能力）

### 13.1 能力定义

数据库执行器原生支持：

| 能力 | 说明 |
|------|------|
| Agent session context | Agent会话上下文 |
| Semantic join inference | 语义Join推断 |
| Policy auto rewrite | 策略自动改写 |
| Query explain feedback | 查询反馈循环 |

### 13.2 定位

| 级别 | 说明 |
|------|------|
| **不是** | 数据库 + AI |
| **而是** | AI-native database kernel |

### 13.3 竞争优势

这是当前**开源数据库领域几乎没人做**的方向。

| 竞品 | 当前能力 |
|------|----------|
| PostgreSQL | 插件支持AI |
| MySQL | 无 |
| Snowflake | Cortex AI |
| Databricks | Unity Catalog |
| **SQLRustGo v2.1** | **原生Agent-native** |

---

## 十六、OpenClaw Extension 插件规范（生产级）

### 16.1 插件总体目标

**目标**：实现 `OpenClaw 安装插件 → 自动发现 SQLRustGo → Agent 直接可查询数据库`

### 16.2 插件目录结构

```
extensions/sqlrustgo/
├── index.ts           # 插件入口
├── client.ts          # HTTP客户端
├── tools/
│   ├── query.ts      # SQL查询工具
│   ├── nl_query.ts   # 自然语言查询工具
│   ├── schema.ts     # Schema查询工具
│   ├── stats.ts      # 统计信息工具
│   ├── explain.ts    # Explain工具
│   └── memory.ts     # Memory工具
├── config.yaml       # 服务配置
└── skill.yaml       # Agent技能声明
```

### 16.3 config.yaml（服务配置）

```yaml
server:
  endpoint: http://localhost:8080

timeout:
  seconds: 10

auth:
  api_key: "{{SQLRUSTGO_API_KEY}}"
```

### 16.4 client.ts（统一请求层）

```typescript
export class SQLRustGoClient {
  constructor(endpoint: string) {
    this.endpoint = endpoint;
  }

  async post(path: string, body: any) {
    const response = await fetch(
      `${this.endpoint}${path}`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(body)
      }
    );

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${await response.text()}`);
    }

    return await response.json();
  }

  async get(path: string) {
    const response = await fetch(
      `${this.endpoint}${path}`,
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json"
        }
      }
    );

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${await response.text()}`);
    }

    return await response.json();
  }
}
```

### 16.5 Tool 实现

#### Tool 1: NL 查询工具（核心）

```typescript
// tools/nl_query.ts
export async function sql_nl_query(client: SQLRustGoClient, question: string) {
  return await client.post("/agentsql/nl_query", { question });
}
```

**Agent调用**：`sql_nl_query("最近7天新增用户")`

**返回**：
```json
{
  "sql": "SELECT COUNT(*) FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 7 DAY)",
  "rows": [[42]],
  "summary": "最近7天新增42个用户",
  "confidence": 0.93
}
```

#### Tool 2: SQL 查询工具

```typescript
// tools/query.ts
export async function sql_query(client: SQLRustGoClient, sql: string) {
  return await client.post("/agentsql/query", { sql });
}
```

#### Tool 3: Schema 查询工具

```typescript
// tools/schema.ts
export async function sql_schema(client: SQLRustGoClient) {
  return await client.get("/agentsql/schema");
}
```

#### Tool 4: 统计信息工具

```typescript
// tools/stats.ts
export async function sql_stats(client: SQLRustGoClient) {
  return await client.get("/agentsql/stats");
}
```

#### Tool 5: Explain 工具

```typescript
// tools/explain.ts
export async function sql_explain(client: SQLRustGoClient, sql: string) {
  return await client.post("/agentsql/explain", { sql });
}
```

#### Tool 6: Memory 工具

```typescript
// tools/memory.ts

// 保存记忆
export async function save_memory(
  client: SQLRustGoClient,
  session_id: string,
  memory: string
) {
  return await client.post("/agentsql/memory/save", { session_id, memory });
}

// 加载记忆
export async function load_memory(
  client: SQLRustGoClient,
  session_id: string
) {
  return await client.post("/agentsql/memory/load", { session_id });
}
```

### 16.6 skill.yaml（Agent技能声明）

```yaml
name: sqlrustgo_tools
version: "1.0"
description: SQLRustGo AI Agent Tools - Native database access for autonomous agents

provider:
  type: http
  endpoint: "{{SQLRUSTGO_ENDPOINT}}/agentsql"

tools:
  - name: sql_nl_query
    description: 查询数据库（自然语言），输入问题返回SQL和结果

  - name: sql_query
    description: 执行SQL查询，直接返回结果

  - name: sql_schema
    description: 获取数据库结构，包括表、列、关系

  - name: sql_stats
    description: 获取数据库统计信息，包括行数、表大小

  - name: sql_explain
    description: 获取SQL执行计划，用于优化查询

  - name: save_memory
    description: 保存分析上下文，支持跨查询分析

  - name: load_memory
    description: 加载分析上下文，支持多轮推理
```

### 16.7 index.ts（插件入口）

```typescript
import config from "./config.yaml";
import { SQLRustGoClient } from "./client";
import * as query from "./tools/query";
import * as nl_query from "./tools/nl_query";
import * as schema from "./tools/schema";
import * as stats from "./tools/stats";
import * as explain from "./tools/explain";
import * as memory from "./tools/memory";

const client = new SQLRustGoClient(config.server.endpoint);

export default {
  name: "sqlrustgo",
  version: "1.0",

  tools: {
    sql_nl_query: (question: string) =>
      nl_query.sql_nl_query(client, question),

    sql_query: (sql: string) =>
      query.sql_query(client, sql),

    sql_schema: () =>
      schema.sql_schema(client),

    sql_stats: () =>
      stats.sql_stats(client),

    sql_explain: (sql: string) =>
      explain.sql_explain(client, sql),

    save_memory: (session_id: string, memory: string) =>
      memory.save_memory(client, session_id, memory),

    load_memory: (session_id: string) =>
      memory.load_memory(client, session_id)
  }
};
```

### 16.8 插件加载流程

```
OpenClaw Runtime
     ↓
scan extensions/
     ↓
load sqlrustgo/
     ↓
register tools
     ↓
Agent 可调用
```

### 16.9 v2.1 能力矩阵

| 能力 | 支持 | 说明 |
|------|------|------|
| NL查询 | ✅ | 自然语言转SQL |
| SQL执行 | ✅ | 直接SQL查询 |
| Explain | ✅ | 执行计划分析 |
| Schema graph | ✅ | 表结构理解 |
| Memory context | ✅ | 跨查询分析 |
| Stats | ✅ | 统计信息 |
| Policy control | ✅ | 权限策略 |
| Embedding schema | v2.2 | 语义搜索 |

---

## 十七、v2.1 发布亮点

### 17.1 官方命名

**SQLRustGo AgentSQL Extension** 或 **SQLRustGo AI Connector for OpenClaw**

### 17.2 发布描述

> SQLRustGo v2.1 introduces **AgentSQL**, a native AI Agent access layer and official OpenClaw extension, enabling secure natural-language database interaction, schema-aware querying, and multi-step analytical workflows directly inside autonomous agents.

### 17.3 竞争优势

| 竞品 | AI能力 |
|------|--------|
| PostgreSQL | 需要 pgvector 插件 |
| MySQL | 无AI能力 |
| Snowflake | Cortex AI（商业） |
| Databricks | Unity Catalog（商业） |
| **SQLRustGo v2.1** | **原生AgentSQL（开源）** |

---

## 十八、下一步行动

### 18.1 立即可做

1. 创建 `extensions/sqlrustgo/` 目录结构
2. 实现 `client.ts` HTTP客户端
3. 实现基础 Tool（query, schema, stats）
4. 编写 `skill.yaml` Agent技能声明

### 18.2 后续规划

1. 实现 `nl_query` 自然语言查询
2. 实现 `memory` 上下文记忆
3. 实现 `explain` 执行计划分析
4. 实现 `security` 权限策略

---

## 十九、相关文档

| 文档 | 说明 |
|------|------|
| [openclaw_database_interface_discussion.md](./openclaw_database_interface_discussion.md) | 初步讨论 |
| [v2x-development-plan.md](../v2x-development-plan.md) | v2.x开发计划 |
| [ARCHITECTURE_OVERVIEW.md](./ARCHITECTURE_OVERVIEW.md) | 架构总览 |

---

*本文档为正式架构设计，可进入 SQLRustGo v2.1 Roadmap*
