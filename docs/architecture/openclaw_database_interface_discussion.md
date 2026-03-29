# OpenClaw AI Agent 数据库接口讨论

> **日期**: 2026-03-29
> **版本**: v1.0
> **类型**: 技术讨论文档
> **状态**: 初始讨论

---

## 一、背景：给软件自动增加 AI API 的开源方案

### 1.1 现有方案概览

已经形成一类标准架构：**Agent Tool Layer / MCP Layer / Skill Layer**

### 1.2 典型代表

| 类型 | 工具 | 作用 |
|------|------|------|
| Agent Runtime | OpenClaw | Agent 调度 + Skills |
| Tool Framework | LangChain SQL toolkit | SQL Tool 抽象 |
| MCP Connector | Jentic / Guardclaw | API代理安全层 |
| DB-Agent Bridge | BridgeScope | 数据库操作工具化 |
| NL2SQL Agent | AskDB | 自然语言数据库操作 |

### 1.3 核心思想

不让 AI 直接连数据库，而是通过**工具接口层**访问数据库：

```
Agent → Tool API → DB Proxy → SQL Database
```

---

## 二、OpenClaw 操作数据库的方式

OpenClaw 通过以下组件组合执行任务：
- Skill
- Tool
- Gateway
- Provider

数据库访问一般有 **4 种方式**。

---

## 三、主流 AI Agent 操作 SQL 的 4 种方式

### 方式1：CLI 包装层（最原始）

```bash
agent.exec("psql -c 'SELECT * FROM users'")
```

| 特点 | 说明 |
|------|------|
| **优点** | 快速、无需开发、OpenClaw默认支持 |
| **缺点** | 不安全、不可控、无权限细粒度管理、无schema awareness |
| **适用场景** | 个人环境、实验环境、PoC |

### 方式2：REST API 数据库代理（推荐）

```
Agent → REST API → SQL Proxy → Database
```

**示例请求**：
```json
POST /query
{
  "question": "今天新增多少用户"
}
```

**示例响应**：
```json
{
  "sql": "SELECT COUNT(*) FROM users WHERE created_at > '2026-03-29'",
  "result": [{"count": 42}],
  "explanation": "查询今天新增用户数量"
}
```

| 优点 | 说明 |
|------|------|
| 非常适合 Agent | Agent天然适合调用HTTP工具 |
| OpenClaw原生支持 | Agent → Tool → HTTP endpoint |

### 方式3：LangChain SQL Tool（当前主流）

```python
SQLDatabaseToolkit()
```

提供的能力：
- `query_sql_db` - 执行SQL查询
- `list_tables` - 列出表
- `describe_table` - 描述表结构
- `run_sql` - 运行SQL

Agent可以：
1. 查看schema
2. 生成SQL
3. 执行SQL
4. 修复SQL
5. 再次执行

属于：**Schema-aware SQL Agent**

### 方式4：BridgeScope数据库代理层（下一代架构）

把数据库能力拆成工具：

| 工具 | 作用 |
|------|------|
| `READ` | 读取数据 |
| `WRITE` | 写入数据 |
| `TRANSACTION` | 事务管理 |
| `SCHEMA` | Schema查询 |
| `CONTEXT` | 上下文管理 |

**Agent调用示例**：
```python
get_schema()      # 获取表结构
read_rows()       # 读取行
write_rows()      # 写入行
commit_tx()       # 提交事务
```

| 优点 | 说明 |
|------|------|
| 安全 | 细粒度权限控制 |
| 可控 | 操作可追溯 |
| 低token消耗 | 论文实验：减少80% token使用量 |
| 权限隔离 | 不同Agent不同权限 |

---

## 四、OpenClaw 推荐的数据库接入方式

官方推荐模式：**Skill + Tool + Gateway API**

```
Agent → Skill YAML → Tool API → Service Layer → Database
```

**示例 Skill 定义**：

```yaml
# skill.md
name: query_users
tool: http
endpoint: /db/query
method: POST
```

**Agent执行**：
```python
query_users("今天新增多少用户")
```

---

## 五、最适合 AI Agent 的 SQL API 设计

### 5.1 Agent Database API Layer

| 接口 | 方法 | 说明 |
|------|------|------|
| `/db/schema` | GET | 获取数据库Schema |
| `/db/query` | POST | 执行查询 |
| `/db/insert` | POST | 插入数据 |
| `/db/update` | POST | 更新数据 |
| `/db/delete` | POST | 删除数据 |
| `/db/explain` | POST | 解释查询计划 |
| `/db/statistics` | GET | 获取统计信息 |

### 5.2 推荐返回格式

**统一 JSON 响应**：

```json
POST /db/query

请求：
{
  "question": "今天新增多少用户",
  "sql": "SELECT COUNT(*) FROM users WHERE DATE(created_at) = CURDATE()"
}

响应：
{
  "sql": "SELECT COUNT(*) FROM users WHERE DATE(created_at) = CURDATE()",
  "rows": [{"count": 42}],
  "columns": ["count"],
  "summary": "今天新增42个用户",
  "confidence": 0.95
}
```

### 5.3 为什么Agent喜欢这种格式

| 特性 | 原因 |
|------|------|
| 可解释 | 包含SQL和summary |
| 可验证 | 包含执行结果 |
| 可调试 | 包含confidence |
| 可循环执行 | 响应格式一致 |

---

## 六、AgentSQL API v1 规范（推荐实现）

### 6.1 Schema查询

```http
GET /db/schema
```

**响应**：
```json
{
  "tables": [
    {
      "name": "users",
      "columns": [
        {"name": "id", "type": "INT", "nullable": false},
        {"name": "name", "type": "VARCHAR(100)", "nullable": false},
        {"name": "email", "type": "VARCHAR(255)", "nullable": true}
      ],
      "relations": [
        {"from": "orders.user_id", "to": "users.id", "type": "1:N"}
      ],
      "indexes": [
        {"name": "idx_email", "columns": ["email"], "unique": true}
      ]
    }
  ]
}
```

### 6.2 自然语言查询

```http
POST /db/nl_query
```

**请求**：
```json
{
  "question": "今天新增多少用户"
}
```

**响应**：
```json
{
  "sql": "SELECT COUNT(*) FROM users WHERE DATE(created_at) = CURDATE()",
  "result": [{"count": 42}],
  "explanation": "查询今天(2026-03-29)新增用户数量",
  "confidence": 0.95
}
```

### 6.3 SQL执行接口

```http
POST /db/sql
```

**请求**：
```json
{
  "sql": "SELECT * FROM users LIMIT 10"
}
```

**响应**：
```json
{
  "rows": [
    {"id": 1, "name": "张三", "email": "zhang@example.com"},
    {"id": 2, "name": "李四", "email": "li@example.com"}
  ],
  "execution_time_ms": 12,
  "plan": {
    "operation": "Seq Scan",
    "estimated_cost": 10.5
  }
}
```

### 6.4 查询解释接口

```http
POST /db/explain
```

**响应**：
```json
{
  "query_plan": "Hash Join",
  "cost": 145.5,
  "optimization_suggestion": "建议在orders.user_id上创建索引"
}
```

### 6.5 数据统计接口

```http
GET /db/stats
```

**响应**：
```json
{
  "tables": {
    "users": {
      "row_count": 1000000,
      "size_bytes": 52428800,
      "index_usage": {"idx_email": 0.85}
    }
  }
}
```

---

## 七、企业级 Agent 数据库接口：权限控制层

### 7.1 AgentRolePolicy

```yaml
agent:
  name: "user_query_agent"
  read_only: true
  tables:
    - users
    - orders
  denied_columns:
    - users.password
    - users.ssn
  max_rows_per_query: 1000
```

### 7.2 权限控制矩阵

| Agent角色 | SELECT | INSERT | UPDATE | DELETE |
|-----------|--------|--------|--------|--------|
| `readonly_agent` | ✅ | ❌ | ❌ | ❌ |
| `data_entry_agent` | ✅ | ✅ | ❌ | ❌ |
| `admin_agent` | ✅ | ✅ | ✅ | ✅ |
| `dev_agent` | ✅ | ✅ | ✅ | ❌ |

### 7.3 安全考虑

> ⚠️ **警告**：如果不加权限控制，Agent迟早删库（真实案例）

---

## 八、OpenClaw + SQLRustGo 数据库接口层：推荐架构

### 8.1 整体架构

```
OpenClaw
   ↓
Skill
   ↓
AgentSQL REST API
   ↓
SQLRustGo
```

### 8.2 模块划分

| 模块 | 职责 |
|------|------|
| `agentsql-core` | 核心API框架 |
| `agentsql-security` | 权限控制、认证 |
| `agentsql-schema` | Schema管理和查询 |
| `agentsql-nl2sql` | 自然语言转SQL |
| `agentsql-optimizer` | 查询优化建议 |

### 8.3 技术特性

| 特性 | 说明 |
|------|------|
| RESTful API | 符合OpenAPI 3.0规范 |
| Schema-aware | 支持表结构感知 |
| Token优化 | 减少80% token消耗 |
| 权限隔离 | 细粒度权限控制 |
| 审计日志 | 完整操作记录 |

---

## 九、可以作为 SQLRustGo 2.1 核心卖点

### 9.1 差异化特性

| 特性 | 传统数据库 | SQLRustGo 2.1 |
|------|------------|-----------------|
| AI接口 | 无 | 原生AgentSQL API |
| 自然语言查询 | 无 | 内置NL2SQL |
| Token消耗 | N/A | 减少80% |
| 权限控制 | 粗粒度 | 细粒度Agent权限 |
| 查询优化建议 | 无 | AI自动建议 |

### 9.2 定位

- **AI-native database interface standard**
- **OpenClaw官方推荐的数据库接入方式**

---

## 十、下一步行动

### 10.1 立即可做

1. 设计完整 OpenAPI 3.0 规范
2. 实现基础 AgentSQL REST API
3. 集成到 SQLRustGo 2.1 开发路线

### 10.2 后续规划

1. 实现 agentsql-security 权限控制
2. 实现 agentsql-nl2sql 自然语言转SQL
3. 实现 agentsql-optimizer 查询优化建议

---

## 十一、SQLRustGo v2.1 AgentSQL API 设计规范

### 11.1 总体设计

**目标**：让 OpenClaw / Agent / MCP / RAG 系统安全、高效、可解释地访问数据库

```
Agent
  ↓
Skill / Tool
  ↓
AgentSQL API
  ↓
SQLRustGo Engine
```

**模块划分**：

```
agentsql/
├── schema        # Schema introspection
├── query         # SQL execution
├── nl2sql        # Natural language to SQL
├── optimizer     # Query optimizer
├── stats         # Statistics
├── security      # Security & permissions
└── memory        # Context memory
```

**REST API 端点**：

| API | Agent用途 |
|-----|----------|
| `/agentsql/schema` | 获取数据库结构 |
| `/agentsql/query` | 执行SQL |
| `/agentsql/nl_query` | 自然语言查询 |
| `/agentsql/explain` | 查询解释 |
| `/agentsql/stats` | 数据统计 |
| `/agentsql/context` | Agent记忆上下文 |

---

### 11.2 核心 API 设计

#### 1. Schema Introspection API

Agent第一步必须知道数据库结构

```http
GET /agentsql/schema
```

**响应**：
```json
{
  "tables": [
    {
      "name": "users",
      "columns": [
        {"name": "id", "type": "int"},
        {"name": "created_at", "type": "timestamp"}
      ]
    }
  ],
  "relations": []
}
```

#### 2. SQL 执行 API（核心）

```http
POST /agentsql/query
```

**输入**：
```json
{
  "sql": "SELECT COUNT(*) FROM users"
}
```

**输出**：
```json
{
  "columns": ["count"],
  "rows": [[42]],
  "execution_time_ms": 2
}
```

#### 3. 自然语言查询 API（AI核心能力）

```http
POST /agentsql/nl_query
```

**输入**：
```json
{
  "question": "今天新增多少用户"
}
```

**输出**：
```json
{
  "sql": "SELECT COUNT(*) FROM users WHERE created_at >= CURRENT_DATE",
  "rows": [[42]],
  "summary": "今天新增 42 个用户",
  "confidence": 0.93
}
```

#### 4. Explain 查询计划 API

```http
POST /agentsql/explain
```

**输入**：
```json
{
  "sql": "SELECT * FROM users WHERE id=5"
}
```

**输出**：
```json
{
  "plan": "index scan",
  "cost": 0.03,
  "suggestion": "使用主键索引"
}
```

#### 5. 数据统计 API

```http
GET /agentsql/stats
```

**输出**：
```json
{
  "tables": [
    {
      "name": "users",
      "rows": 120003,
      "size_mb": 23
    }
  ]
}
```

---

### 11.3 Agent Context API（AI-native核心能力）

```http
POST /agentsql/context
```

**输入**：
```json
{
  "session_id": "agent123",
  "memory": "当前正在分析用户增长趋势"
}
```

**输出**：
```json
{
  "status": "ok"
}
```

**用途**：
- Agent共享数据库上下文
- 跨查询记忆
- 多轮分析支持

---

### 11.4 安全模型

**权限模型** (`agentsql_policy.yaml`)：

```yaml
agent_role:
  analyst:
    allow:
      - SELECT

  operator:
    allow:
      - SELECT
      - INSERT

  admin:
    allow:
      - ALL
```

**表级权限**：

```yaml
tables:
  users:
    analyst: read
    admin: write
```

**列级权限**：

```yaml
columns:
  users.phone:
    analyst: deny
```

---

### 11.5 Schema Embedding API（AI数据库关键创新）

```http
GET /agentsql/schema_embedding
```

**返回**：vectorized schema

**用途**：
- 支持RAG
- 支持semantic SQL search
- 支持自动SQL生成

---

### 11.6 AgentSQL 内部执行流程

```
Agent question
  ↓
schema lookup
  ↓
vector schema search
  ↓
nl2sql
  ↓
optimizer
  ↓
executor
  ↓
explain feedback
  ↓
result
```

---

### 11.7 OpenClaw Skill 示例

```yaml
# skill.yaml
name: query_database
tool: http
endpoint: /agentsql/nl_query
method: POST
```

**Agent调用**：
```python
query_database("统计最近7天新增用户")
```

---

## 十二、SQLRustGo v2.1 Roadmap

### 12.1 版本目标

> **SQLRustGo v2.1 = AI Agent Native Database**

### 12.2 开发阶段

| Phase | 内容 | 周期 |
|-------|------|------|
| **Phase 1** | schema API, query API, stats API | 2周 |
| **Phase 2** | nl_query API, context API, policy API | 3周 |
| **Phase 3** | embedding schema, explain engine, optimizer feedback | 4周 |

### 12.3 总周期

**9周** → 完成全球第一批 **Agent-native embedded database engine** 级别能力

---

## 十三、参考资料

| 资源 | 说明 |
|------|------|
| OpenClaw | Agent Runtime框架 |
| LangChain SQL Toolkit | SQL Tool抽象 |
| BridgeScope | DB-Agent Bridge论文 |
| AskDB | NL2SQL Agent |
| Databricks Unity Catalog | AI Schema概念参考 |
| Snowflake Cortex | AI-native数据库参考 |

---

*本文档为技术讨论记录，可作为 SQLRustGo 2.1 版本开发参考*
