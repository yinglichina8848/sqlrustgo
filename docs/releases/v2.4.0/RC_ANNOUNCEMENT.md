# v2.4.0 RC1 发布声明

## 发布信息

| 项目 | 值 |
|------|---|
| 版本 | **v2.4.0-rc1** |
| 代号 | **Graph Intelligence** |
| 发布日期 | 2026-04-09 |
| 阶段 | **RC1 (Release Candidate 1)** |
| 分支 | `release/v2.4.0-rc1` |
| Tag | `v2.4.0-rc1` |

## 简介

v2.4.0 RC1 是 SQLRustGo 的图引擎与智能查询版本，引入以下核心功能：

1. **Graph Engine** - 独立图引擎 crate，支持 GQL 图查询语言
2. **OpenClaw API** - RESTful API，支持自然语言查询
3. **Columnar Compression** - LZ4/Zstd 列式存储压缩
4. **CBO Index Selection** - 基于成本的索引选择优化器
5. **TPC-H SF=1** - 完整性能基准测试

## 核心功能

### Graph Engine (Issue #1077)

```rust
// 图引擎作为独立 crate
use graph_engine::{GraphEngine, GQLParser, GraphExecutor};

// 创建图引擎
let engine = GraphEngine::new();
let plan = GQLParser::parse("MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a, b");
let result = GraphExecutor::execute(&engine, &plan);
```

### OpenClaw API (Issue #1078)

```bash
# REST API 端点
curl http://localhost:8080/query -d '{"sql": "SELECT * FROM users"}'
curl http://localhost:8080/nl_query -d '{"query": "Show all users"}'
curl http://localhost:8080/schema
curl http://localhost:8080/stats
curl http://localhost:8080/memory/save -d '{"key": "context", "value": "..."}'
```

### 列式压缩 (Issue #1302)

```sql
-- 创建带压缩的列式表
CREATE TABLE events (
    id BIGINT,
    data VARCHAR COLUMNAR COMPRESSION LZ4
);

-- 或使用 Zstd
CREATE TABLE large_data (
    id BIGINT,
    payload BLOB COLUMNAR COMPRESSION ZSTD
);
```

### CBO Index Selection (Issue #1303)

```sql
-- 优化器自动选择最优索引
EXPLAIN SELECT * FROM orders WHERE customer_id = 123;
-- 输出显示使用的索引和成本估算
```

## 测试结果

### 测试统计

| 测试类型 | 结果 | 状态 |
|----------|------|------|
| 单元测试 | 35/35 | ✅ 100% |
| 集成测试 | 1040/1042 | ✅ 99.8% |
| TPC-H SF=1 | 11/11 | ✅ 100% |
| OpenClaw API | 11/11 | ✅ 100% |

### TPC-H SF=1 测试结果

| Query | Status | Time |
|-------|--------|------|
| Q1 | ✅ | 0.12s |
| Q6 | ✅ | 0.08s |
| Q12 | ✅ | 0.15s |
| ... | ✅ | ... |

完整报告: [TPCH-SF1-PERFORMANCE-REPORT.md](../../benchmark/TPCH-SF1-PERFORMANCE-REPORT.md)

## 安装

### 从源码构建

```bash
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo
git checkout v2.4.0-rc1
cargo build --release
cargo install --path . --bins
```

### 从 crates.io

```bash
cargo install sqlrustgo
```

## 文档

- [Release Notes](./RELEASE_NOTES.md)
- [Changelog](./CHANGELOG.md)
- [性能报告](../../benchmark/TPCH-SF1-PERFORMANCE-REPORT.md)
- [Graph Engine 文档](../../graph-engine/README.md) (待补充)

## 反馈

如果您在使用 v2.4.0-rc1 时遇到任何问题，请在 [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues) 报告。

## 后续计划

- **v2.4.0 GA**: 收集社区反馈后发布
- **v2.5 开发**: GMP 内审 + OpenClaw 调度优化

## 谢谢

感谢所有参与测试和反馈的社区成员！

---

**SQLRustGo Team**
**2026-04-09**
