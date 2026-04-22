# v2.7.0 升级指南

> **版本**: v2.7.0
> **适用**: 从 v2.6.0 升级

---

## 概述

本指南帮助用户从 v2.6.0 升级到 v2.7.0。v2.7.0 是生产就绪版本，包含 WAL 崩溃恢复、外键稳定性增强、备份恢复机制、审计证据链等重大功能改进。

**重要**: v2.7.0 包含破坏性变更 (WAL 格式更新)，请在升级前仔细阅读本指南。

---

## 重大变更

### Breaking Changes

| 变更 | 影响 | 迁移建议 |
|------|------|----------|
| WAL 格式升级 | 需迁移 WAL 格式 | 使用 `sqlrustgo-tools migrate-wal` |
| 审计配置 | 需要配置审计存储 | 见下方配置变更章节 |

### API 变更

#### 新增 QMD Bridge API

v2.7.0 引入了 QMD Bridge (Query Metadata Bridge) API:

```rust
use sqlrustgo::QmdBridge;

// 创建 QMD 桥接器
let bridge = QmdBridge::new(engine.clone());

// 同步数据到 QMD
bridge.sync_to_qmd(&qmd_data)?;

// 执行混合检索
let result = bridge.hybrid_search(&query)?;
```

#### 新增统一搜索 API

v2.7.0 提供了统一的混合搜索接口:

```rust
use sqlrustgo::UnifiedSearch;

let search = UnifiedSearch::new(storage);
let results = search.hybrid_search(params)?;
```

#### 废弃 API

| API | 废弃版本 | 替代方案 | 移除版本 |
|-----|----------|----------|----------|
| `ExecutionEngine::execute_raw` | v2.7.0 | `ExecutionEngine::execute` | v2.8.0 |
| `VectorSearch::search_knn` | v2.7.0 | `UnifiedSearch::hybrid_search` | v2.8.0 |
| `SearchAPI::legacy_search()` | v2.7.0 | `SearchAPI::unified_search()` | v2.8.0 |

---

## SQL 语法变更

### 新增功能

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

-- 检查同步状态
SELECT * FROM qmd_sync_status();
```

#### GMP Top10

```sql
-- 查询热点数据 (GMP Top10)
SELECT * FROM gmp_top10('products', 10);
```

### 性能改进

| 操作 | v2.6.0 | v2.7.0 | 提升 |
|------|--------|--------|------|
| WAL 写入 | 基准 | 提升 20% | ✅ |
| 外键验证 | 基准 | 提升 30% | ✅ |
| 备份速度 | 基准 | 提升 40% | ✅ |
| 搜索延迟 | 基准 | 降低 25% | ✅ |
| 重排序速度 | 基准 | 提升 35% | ✅ |
| 审计写入 | 基准 | 提升 50% | ✅ |

---

## 升级步骤

### 阶段 1: 准备

1. **备份所有数据库文件**

```bash
# 创建备份目录
mkdir -p /backup/v2.6.0

# 执行全量备份
sqlrustgo-tools backup --output /backup/v2.6.0 --mode full
```

2. **停止所有运行中的 SQLRustGo 实例**

```bash
# 检查运行中的实例
ps aux | grep sqlrustgo

# 优雅停止 (发送 SIGTERM)
kill -TERM <pid>
```

### 阶段 2: 升级

3. **切换到 v2.7.0 标签**

```bash
git checkout v2.7.0
```

4. **重新构建**

```bash
cargo build --release
```

5. **迁移 WAL 格式 (关键步骤)**

```bash
# 迁移 WAL 格式
sqlrustgo-tools migrate-wal --database mydb

# 验证迁移
sqlrustgo-tools verify --database mydb
```

### 阶段 3: 配置

6. **配置审计存储 (新增功能)**

编辑配置文件 `sqlrustgo.toml`:

```toml
[audit]
enabled = true
storage_path = "/var/lib/sqlrustgo/audit"
retention_days = 365
immutable = true
```

初始化审计存储:

```bash
sqlrustgo-tools init-audit --path /var/lib/sqlrustgo/audit
```

7. **配置 QMD Bridge (可选，如使用混合检索)**

```toml
[qmd]
enabled = true
host = "localhost"
port = 6632

[hybrid_search]
default_weights = [0.4, 0.3, 0.3]
rerank_enabled = true
```

### 阶段 4: 验证

8. **启动新版本实例**

```bash
# 启动 sqlrustgo
sqlrustgo --config sqlrustgo.toml
```

9. **验证数据完整性**

```bash
sqlrustgo-tools check --integrity
```

10. **验证 WAL 恢复机制**

```bash
# 触发崩溃恢复测试
sqlrustgo-tools test-recovery --simulate-crash
```

11. **运行门禁测试**

```bash
# L0 冒烟测试
cargo fmt --check --all
cargo clippy --all-features -- -D warnings
cargo build --all-features

# 运行测试套件
cargo test --all-features
```

---

## 配置变更

### 构建要求

v2.7.0 需要以下依赖:

- Rust 1.85+
- Cargo (随 Rust 安装)

### 特性标志

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 全特性构建 (包含所有功能)
cargo build --all-features

# QMD Bridge 支持
cargo build --features qmd-bridge
```

### 配置项变更

#### 新增配置项

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

#### 废弃配置项

| 配置项 | 废弃版本 | 替代方案 | 移除版本 |
|--------|----------|----------|----------|
| `[vector].hnsw_enable` | v2.7.0 | `[vector].index_type` | v2.8.0 |

---

## 数据迁移

### 存储格式

v2.7.0 与 v2.6.0 数据格式兼容，但 **WAL 格式不兼容**。

### 索引

升级后建议重建索引以获得最佳性能:

```sql
REINDEX DATABASE;
```

---

## 已知问题

### 待解决 (v2.7.1)

1. **极端并发**: 在极端并发下可能出现少量性能波动
2. **大型事务**: 大型事务 (> 2GB) 处理时间较长

### 限制

- 分布式事务支持 (跨节点) 仍在设计中
- SSI 隔离级别 (可串行化快照隔离) 仍在规划中

---

## 回滚

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

## 常见问题

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

---

## 获取帮助

- [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues)
- [文档首页](./README.md)
- [版本计划](./VERSION_PLAN.md)
- [迁移指南](./MIGRATION_GUIDE.md)
- [变更日志](./CHANGELOG.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-22*
