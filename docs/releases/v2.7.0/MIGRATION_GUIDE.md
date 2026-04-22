# 迁移指南

> **版本**: v2.7.0
> **适用版本**: 从 v2.6.0 升级

---

## 1. 概述

本文档提供从 v2.6.0 迁移到 v2.7.0 的完整指南。

v2.7.0 是生产就绪版本，包含 WAL 恢复机制、外键稳定性、备份恢复、统一搜索 API、混合重排序、GMP Top10、审计证据链、性能回归修复和 72 小时稳定性增强等重大功能改进。

---

## 2. 重大变更

### 2.1 WAL 格式迁移 (Breaking Change)

v2.7.0 引入了新的 WAL (Write-Ahead Logging) 恢复机制，需要迁移现有 WAL 格式。

```bash
# 使用迁移工具迁移 WAL 格式
sqlrustgo-tools migrate-wal --database mydb

# 验证迁移
sqlrustgo-tools verify --database mydb
```

**影响**: 如果不迁移，v2.7.0 将无法读取 v2.6.0 的 WAL 日志。

**回滚**: 如需回滚到 v2.6.0，请使用备份恢复功能。

### 2.2 审计配置 (Breaking Change)

v2.7.0 引入了审计证据链功能，需要配置审计存储。

编辑配置文件添加审计配置:

```toml
[audit]
enabled = true
storage_path = "/var/lib/sqlrustgo/audit"
retention_days = 365
immutable = true
```

**影响**: 未配置审计存储将导致审计功能禁用，但不影响核心数据库功能。

---

## 3. 新功能

### 3.1 QMD Bridge (Query Metadata Bridge)

v2.7.0 引入了 QMD Bridge 功能，支持 SQL + 向量 + 图谱的混合检索。

#### Rust API

```rust
// v2.7.0 新增
use sqlrustgo::QmdBridge;

// 创建 QMD 桥接器
let bridge = QmdBridge::new(engine.clone());

// 同步数据到 QMD
bridge.sync_to_qmd(&qmd_data)?;

// 执行混合检索
let result = bridge.hybrid_search(&query)?;
```

#### SQL 语法

```sql
-- 混合检索 (向量 + 图谱 + 全文)
SELECT * FROM users 
WHERE HYBRID_SEARCH(
    embedding => query_vector,
    graph_pattern => 'MATCH (a)-[r]->(b)',
    text_query => '关键词',
    weights => [0.4, 0.3, 0.3],
    limit => 10
);

-- 同步到 QMD
SYNC TO QMD FROM users WHERE condition;

-- 检查同步状态
SELECT * FROM qmd_sync_status();
```

### 3.2 统一搜索 API

```rust
// v2.7.0 新增 - 统一检索接口
use sqlrustgo::UnifiedSearch;

let search = UnifiedSearch::new(storage);
let results = search.hybrid_search(params)?;
```

### 3.3 混合重排序 (Hybrid Rerank)

混合检索现在支持向量相似度与 BM25 的融合排序:

```sql
-- 混合检索 + 重排序
SELECT * FROM products
WHERE HYBRID_SEARCH(
    embedding => query_vector,
    text_query => '关键词',
    rerank => true,
    weights => [0.5, 0.5]
)
ORDER BY score DESC;
```

### 3.4 GMP Top10 场景

v2.7.0 支持热点数据自动识别和缓存策略增强:

```sql
-- 查询热点数据 (GMP Top10)
SELECT * FROM gmp_top10('products', 10);
```

### 3.5 备份与恢复

v2.7.0 提供了完整的物理备份和逻辑备份功能:

```bash
# 全量备份
sqlrustgo-tools backup --output /backup/v2.7.0 --mode full

# 增量备份
sqlrustgo-tools backup --output /backup/v2.7.0 --mode incremental

# 恢复
sqlrustgo-tools restore --input /backup/v2.7.0
```

### 3.6 外键稳定性增强

v2.7.0 修复了外键在并发场景下的不稳定问题，增强了外键级联操作的正确性。

---

## 4. SQL 语法变更

### 4.1 新增功能

#### 混合检索 (HYBRID_SEARCH)

```sql
-- v2.6.0: 不支持混合检索
-- v2.7.0: 支持向量 + 图谱 + 全文混合检索

SELECT * FROM users 
WHERE HYBRID_SEARCH(
    embedding => '[0.1, 0.2, 0.3]',
    graph_pattern => 'MATCH (a)-[r]->(b)',
    text_query => '关键词',
    weights => [0.4, 0.3, 0.3],
    limit => 10
);
```

#### QMD 同步

```sql
-- v2.7.0 新增
SYNC TO QMD FROM users WHERE status = 'active';
```

### 4.2 性能改进

| 操作 | v2.6.0 | v2.7.0 | 提升 |
|------|--------|--------|------|
| WAL 写入 | 基准 | 提升 20% | ✅ |
| 外键验证 | 基准 | 提升 30% | ✅ |
| 备份速度 | 基准 | 提升 40% | ✅ |
| 搜索延迟 | 基准 | 降低 25% | ✅ |
| 重排序速度 | 基准 | 提升 35% | ✅ |
| 审计写入 | 基准 | 提升 50% | ✅ |

---

## 5. 数据迁移

### 5.1 存储格式

v2.7.0 与 v2.6.0 数据格式兼容，但 WAL 格式不兼容。

**迁移步骤**:

1. 备份数据
```bash
sqlrustgo-tools backup --output /backup/v2.6.0
```

2. 迁移 WAL 格式
```bash
sqlrustgo-tools migrate-wal --database mydb
```

3. 验证迁移
```bash
sqlrustgo-tools verify --database mydb
```

### 5.2 索引重建

升级后建议重建索引以获得最佳性能:

```sql
REINDEX DATABASE;
```

### 5.3 审计日志

审计日志需要初始化:

```bash
# 初始化审计存储
sqlrustgo-tools init-audit --path /var/lib/sqlrustgo/audit
```

---

## 6. 配置变更

### 6.1 构建要求

v2.7.0 需要以下依赖:

- Rust 1.85+
- Cargo (随 Rust 安装)

### 6.2 特性标志

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 全特性构建
cargo build --all-features

# QMD Bridge 支持 (需要启用)
cargo build --features qmd-bridge
```

### 6.3 新增配置项

```toml
# v2.7.0 新增配置

[audit]
enabled = true
storage_path = "/var/lib/sqlrustgo/audit"
retention_days = 365

[qmd]
enabled = true
host = "localhost"
port = 6632

[hybrid_search]
default_weights = [0.4, 0.3, 0.3]
rerank_enabled = true
```

---

## 7. 弃用说明

### 7.1 弃用的 API

| API | 弃用版本 | 替代方案 | 移除版本 |
|-----|----------|----------|----------|
| `ExecutionEngine::execute_raw` | v2.7.0 | `ExecutionEngine::execute` | v2.8.0 |
| `VectorSearch::search_knn` | v2.7.0 | `UnifiedSearch::hybrid_search` | v2.8.0 |

### 7.2 弃用的 SQL 语法

| 语法 | 弃用版本 | 替代方案 | 移除版本 |
|------|----------|----------|----------|
| `VECTOR_SEARCH(...)` | v2.7.0 | `HYBRID_SEARCH(...)` | v2.8.0 |

### 7.3 弃用的配置项

| 配置项 | 弃用版本 | 替代方案 | 移除版本 |
|--------|----------|----------|----------|
| `[vector].hnsw_enable` | v2.7.0 | `[vector].index_type` | v2.8.0 |

---

## 8. 回滚

如果升级后遇到问题，可以回滚到 v2.6.0:

```bash
# 切换到 v2.6.0 标签
git checkout v2.6.0

# 重新构建
cargo build --release

# 使用备份恢复数据
sqlrustgo-tools restore --input /backup/v2.6.0
```

**注意**: 回滚后需要重新应用 v2.6.0 的 WAL 文件。

---

## 9. 常见问题

### Q1: WAL 迁移失败怎么办?

确保数据库处于一致状态后重试:

```bash
# 检查数据库状态
sqlrustgo-tools check --database mydb

# 如果需要，强制修复
sqlrustgo-tools repair --database mydb

# 重新迁移
sqlrustgo-tools migrate-wal --database mydb
```

### Q2: 审计存储空间不足?

审计日志会自动清理超过 retention_days 的记录。如需手动清理:

```bash
# 清理审计日志
sqlrustgo-tools clean-audit --before 2026-01-01
```

### Q3: 混合检索返回空结果?

检查 QMD 服务状态和同步状态:

```sql
SELECT * FROM qmd_sync_status();
```

### Q4: 性能回退?

运行性能验证:

```bash
cargo bench --package sqlrustgo-bench
```

如发现性能问题，请报告至 [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues)。

---

## 10. 获取帮助

- [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues)
- [文档首页](./README.md)
- [版本计划](./VERSION_PLAN.md)
- [升级指南](./UPGRADE_GUIDE.md)

---

## 11. 参考文档

- [QMD Bridge 设计文档](./qmd-bridge-design.md)
- [GMP Top10 场景](./gmp-top10-scenarios.md)
- [功能矩阵](./FEATURE_MATRIX.md)
- [变更日志](./CHANGELOG.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-22*
