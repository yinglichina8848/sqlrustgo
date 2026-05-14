# v3.1.0 发布说明

> **版本**: 3.1.0  
> **发布日期**: 待定（目标：2026-09-01）  
> **标签**: v3.1.0

---

## v3.1.0 新特性

### v3.1.0-alpha（目标：2026-06-01）

#### INFORMATION_SCHEMA（P0）

INFORMATION_SCHEMA 覆盖率从约 30% 提升至 ≥80%，新增以下系统表：

- `SCHEMATA` — 数据库架构信息
- `TABLES` — 表元数据
- `COLUMNS` — 列定义
- `STATISTICS` — 索引统计信息
- `REFERENTIAL_CONSTRAINTS` — 外键约束

#### SQL 操作（P0）

SQL Operations 通过率从 20%（11/55）提升至 ≥60%（33/55），新增支持：

- `SAVEPOINT` / `ROLLBACK TO SAVEPOINT`
- `SET TRANSACTION ISOLATION LEVEL`
- `LIMIT` / `OFFSET` 优化
- `TRUNCATE TABLE`
- `REPLACE INTO`
- `SHOW` 语句完整变体

#### MERGE 语句（P0）

实现完整的 `MERGE INTO` 语法（MySQL 8.0 兼容）：

```sql
MERGE INTO target AS t
USING source AS s
ON t.id = s.id
WHEN MATCHED THEN
  UPDATE SET t.col = s.col
WHEN NOT MATCHED THEN
  INSERT (id, col) VALUES (s.id, s.col);
```

### v3.1.0-beta（目标：2026-07-01）

#### Performance Schema（P1）

轻量级 Performance Schema 实现，新增 10+ 系统表：

- `setup_actors` — 监控用户配置
- `setup_instruments` — instrument 配置
- `events_statements_summary_by_digest` — 语句聚合统计
- `events_statements_history` — 语句历史
- `events_waits_summary_by_thread` — 等待事件

#### CBO 代价模型集成（P1）

SimpleCostModel 正式接入 planner，带来以下优化：

- `EXPLAIN` 自动选择索引扫描而非全表扫描
- 多表 JOIN 按代价排序（小表先驱动）
- 预计 Point SELECT QPS 提升 ≥5%

#### TLS/SSL（P1）

MySQL 协议 TLS 握手完整集成，支持 `--ssl-mode=REQUIRED`。

#### TPC-H SF=1（P0）

TPC-H SF=1 22/22 查询全部通过，p99 < 5s，无 OOM。

### v3.1.0-rc（目标：2026-08-01）

#### 全文搜索（P1）

中英文分词全文索引支持：

- 英文分词 + 停用词
- 结巴分词（中文）
- `MATCH(col) AGAINST('keyword')` 语法
- INPLACE DML 增量索引更新

#### 事件调度器（P1）

MySQL Event 兼容的定时任务调度器：

```sql
CREATE EVENT daily_cleanup
ON SCHEDULE EVERY 1 DAY
DO
  DELETE FROM audit_log WHERE created_at < NOW() - INTERVAL 30 DAY;
```

#### MERGE JOIN 和 BNL JOIN（P1）

新的 Join 算法补全查询优化器：

- `MERGE JOIN` — Sort-Merge 等值连接
- `BNL JOIN` — Block Nested Loop 非等值连接

### v3.1.0-ga（目标：2026-09-01）

#### GMP 合规核心能力

GMP 合规核心能力：

| 能力 | 状态 |
|------|------|
| 审计日志 SHA-256 链（crash-safe） | GA |
| Gap Locking + SERIALIZABLE | GA |
| 崩溃恢复零丢失（5 场景） | GA |
| AES-256 存储加密 | GA |
| 列级权限 + RBAC 执行层 | GA |
| 聚簇索引 | GA |

#### 稳定性改进

- `long_run_stability_test` — 所有 #[ignore] 已移除
- 稳定性测试 B-S1~B-S5 全部 PASS (≥95%)
- 并发压测 S-01~S-03 新增/完善

---

## 升级说明

### 从 v3.0.0 升级至 v3.1.0

**破坏性变更：**

- `information_schema.columns` 表结构变更（新增 `GENERATION_EXPRESSION` 列）
- Performance Schema 新增系统表，可能影响首次启动时间

**弃用说明：**

- `EXPLAIN` 输出格式保持兼容，但新增 `cost` 列
- `SHOW ENGINE INNODB STATUS` 暂时不可用（RC 后修复）

---

## Bug 修复

| Issue | 描述 |
|-------|------|
| #451 | SQL Operations 语法支持从 20% 提升至 ≥80% |
| #392 | CBO 代价模型接入 planner |
| #379 | 事务状态机压力测试未实现 |
| #382 | TPC-H SF=1 OOM 问题 |

---

## 性能

### 基准测试（v3.1.0 vs v3.0.0）

| 基准测试 | v3.0.0 | v3.1.0 | 变化 |
|-----------|--------|--------|--------|
| Point SELECT QPS | 398K | ≥400K | +0.5% |
| UPDATE QPS | 532K | ≥550K | +3.4% |
| DELETE QPS | 706K | ≥700K | -0.9% |
| TPC-H SF=0.1 | 10.9s | <10s | 提升 |

---

## 贡献者

- 主要贡献：Hermes Agent（首席架构师）
- 开发：OpenCode / Claude Code 子代理

---

*本文档在开发周期中持续更新。*
*最终版本将于 GA 发布时发布。*
