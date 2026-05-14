# v3.0.0 → v3.1.0 升级指南

> **当前版本**: v3.0.0 GA  
> **目标版本**: v3.1.0 GA  
> **发布日期**: 2026-05-14  
> **升级类型**: 标准升级 (向后兼容)

---

## 一、升级概述

### 1.1 版本信息

| 项目 | v3.0.0 | v3.1.0 |
|------|--------|--------|
| 发布日期 | 2026-05-08 | 2026-05-14 |
| 状态 | GA | GA |
| TPC-H SF=1 | OOM (失败) | 22/22 (16.5s) |
| Point Select QPS | ~398K | ~743K |
| SQL 兼容性 | 93.4% | 98.5% |
| 覆盖率 | ~76% | 81.65% |

### 1.2 升级兼容性

| 兼容性维度 | 状态 | 说明 |
|------------|------|------|
| 数据格式 | ✅ 兼容 | 无存储格式变更 |
| SQL 语法 | ✅ 兼容 | v3.0.0 SQL 全部支持 |
| 配置文件 | ✅ 兼容 | 新增配置项可选 |
| API | ✅ 兼容 | 新增 API 可选使用 |
| 网络协议 | ✅ 兼容 | MySQL 协议兼容 |

---

## 二、主要新增功能

### 2.1 P0 新增功能

#### INFORMATION_SCHEMA 完善

| 表 | v3.0.0 | v3.1.0 |
|----|--------|--------|
| SCHEMATA | ❌ | ✅ 完整实现 |
| TABLES | 部分 | ✅ 完整实现 |
| COLUMNS | 部分 | ✅ 完整实现 |
| STATISTICS | ❌ | ✅ 完整实现 |
| REFERENTIAL_CONSTRAINTS | ❌ | ✅ 完整实现 |
| CHARACTER_SETS | ❌ | ✅ 完整实现 |
| COLLATIONS | ❌ | ✅ 完整实现 |

#### SQL Operations 提升

| 功能 | v3.0.0 | v3.1.0 |
|------|--------|--------|
| SQL 兼容性 | 93.4% | 98.5% |
| SAVEPOINT | ❌ | ✅ |
| ROLLBACK TO SAVEPOINT | ❌ | ✅ |
| LIMIT/OFFSET 优化 | 基础 | ✅ 完整 |
| TRUNCATE TABLE | ❌ | ✅ |
| REPLACE INTO | ❌ | ✅ |
| EXPLAIN ANALYZE | 基础 | ✅ 完整 |

#### MERGE 语句

```sql
MERGE INTO target_table AS t
USING source_table AS s
ON t.id = s.id
WHEN MATCHED THEN
  UPDATE SET t.value = s.value
WHEN NOT MATCHED THEN
  INSERT (id, value) VALUES (s.id, s.value);
```

### 2.2 P1 新增功能

#### Full-Text Search (FTS)

```sql
SELECT * FROM articles
WHERE MATCH(title, content) AGAINST('database' IN NATURAL LANGUAGE MODE);
```

#### CBO 代价模型完善

| 优化 | v3.0.0 | v3.1.0 |
|------|--------|--------|
| 索引选择 | 基础 | ✅ 完整 |
| 连接重排序 | 基础 | ✅ 完整 |

#### Event Scheduler

```sql
CREATE EVENT daily_cleanup
ON SCHEDULE EVERY 1 DAY
DO
  DELETE FROM session_history WHERE created_at < NOW() - INTERVAL 7 DAY;
```

### 2.3 架构改进

#### Gap Locking (SERIALIZABLE)

| 隔离级别 | v3.0.0 | v3.1.0 |
|----------|--------|--------|
| SERIALIZABLE | ❌ | ✅ (Next-Key Lock) |

#### 聚簇索引

| 特性 | v3.0.0 | v3.1.0 |
|------|--------|--------|
| 主键索引 | 二级索引 | ✅ 聚簇索引 |
| 无主键表 | 隐式 RowID | ✅ 隐式主键 (UUID) |

#### 存储加密

| 功能 | v3.0.0 | v3.1.0 |
|------|--------|--------|
| 数据页加密 | ❌ | ✅ AES-256-GCM |
| WAL 加密 | ❌ | ✅ |

### 2.4 JOIN 算法增强

| 算法 | v3.0.0 | v3.1.0 |
|------|--------|--------|
| NESTED LOOP JOIN | ✅ | ✅ |
| HASH JOIN | ❌ | ✅ |
| MERGE JOIN | ❌ | ✅ |
| BNL JOIN | ❌ | ✅ |

---

## 三、配置变更

### 3.1 新增配置项

```toml
[fulltext]
enabled = true
english_tokenizer = "simple"
chinese_tokenizer = "jieba"

[optimizer]
enable_cbo = true
cbo_join_reordering = true
cbo_index_selection = true

[storage]
clustered_index = true

[storage.encryption]
enabled = false
algorithm = "AES-256-GCM"

[event_scheduler]
enabled = false
timezone = "UTC"
```

### 3.2 行为变更配置

| 配置项 | v3.0.0 默认 | v3.1.0 默认 |
|--------|-------------|-------------|
| `optimizer.enable_cbo` | false | true |
| `storage.clustered_index` | false | true |

---

## 四、升级步骤

### 4.1 升级前准备

```bash
# 备份数据
cp -r /var/lib/sqlrustgo /var/lib/sqlrustgo.v3.0.0.backup
cp /etc/sqlrustgo/config.toml /etc/sqlrustgo/config.toml.v3.0.0.backup
```

### 4.2 升级步骤

```bash
# 1. 停止服务
sudo systemctl stop sqlrustgo

# 2. 安装新版本
cargo install sqlrustgo --version 3.1.0

# 3. 启动服务
sudo systemctl start sqlrustgo

# 4. 验证
sqlrustgo --version
curl http://localhost:8080/health
```

### 4.3 回滚步骤

```bash
# 1. 停止服务
sudo systemctl stop sqlrustgo

# 2. 恢复数据
rm -rf /var/lib/sqlrustgo
cp -r /var/lib/sqlrustgo.v3.0.0.backup /var/lib/sqlrustgo

# 3. 降级
cargo install sqlrustgo --version 3.0.0

# 4. 启动
sudo systemctl start sqlrustgo
```

---

## 五、变更摘要

**性能提升**:
- Point Select QPS: 398K → 743K (+87%)
- TPC-H SF=1: OOM → 16.5s

**新增 (P0)**:
- INFORMATION_SCHEMA 完整实现
- SQL Operations 98.5%
- MERGE 语句
- TPC-H SF=1 支持

**新增 (P1)**:
- Full-Text Search
- CBO 代价模型完善
- Event Scheduler
- 聚簇索引
- Gap Locking
- 存储加密
- JOIN 算法增强

---

*升级指南版本: v3.1.0*  
*最后更新: 2026-05-14*
