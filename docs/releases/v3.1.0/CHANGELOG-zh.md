# v3.1.0 变更日志

> **版本**: 3.1.0  
> **日期**: 2026-05-11  
> **状态**: 开发中

---

## v3.1.0-ga (待定)

### 新增功能

#### INFORMATION_SCHEMA (P0)
- `SCHEMATA` — 数据库schema信息
- `TABLES` — 表元数据
- `COLUMNS` — 列定义，包含 `CHARACTER_SET_NAME`、`COLLATION_NAME`、`IS_NULLABLE`
- `STATISTICS` — 索引统计信息
- `REFERENTIAL_CONSTRAINTS` — 外键约束
- `CHARACTER_SETS` — 可用字符集
- `COLLATIONS` — 排序规则信息

#### SQL操作 (P0)
- `SAVEPOINT` / `ROLLBACK TO SAVEPOINT`
- `SET TRANSACTION ISOLATION LEVEL`
- `LIMIT` / `OFFSET` 优化
- `TRUNCATE TABLE`
- `REPLACE INTO`
- `SHOW` 语句变体（`SHOW CREATE TABLE`、`SHOW INDEX`、`SHOW ENGINES`）
- `EXPLAIN ANALYZE`

#### MERGE语句 (P0)
- `MERGE INTO ... WHEN MATCHED ... WHEN NOT MATCHED ...`（兼容MySQL 8.0）

#### 性能Schema (P1)
- `setup_actors`
- `setup_instruments`
- `events_statements_summary_by_digest`
- `events_statements_history`
- `events_waits_summary_by_thread`

#### CBO成本模型集成 (P1)
- `SimpleCostModel` 在 planner 中启用
- `EXPLAIN` 中的自动索引选择
- 基于成本的连接重排序

#### TLS/SSL (P1)
- MySQL协议TLS握手完整集成
- 支持 `--ssl-mode=REQUIRED`

#### 全文搜索 (P1)
- 带停用词的英文分词器
- 中文分词器（jieba）
- `MATCH(col) AGAINST('keyword')` 语法
- INPLACE DML增量索引更新

#### 事件调度器 (P1)
- `CREATE EVENT ... ON SCHEDULE`
- `ALTER EVENT`
- `DROP EVENT`
- `SHOW EVENTS`

#### 连接算法 (P1)
- `MERGE JOIN` — 排序-合并等值连接
- `BNL JOIN` — 块嵌套循环非等值连接

#### GMP合规基础 (P0)

**审计日志：**
- 基于SHA-256哈希链的崩溃安全审计跟踪
- WAL原子持久化集成
- 启动时篡改检测
- 带JSON签名的证据导出
- BP2-1~BP2-6测试通过

**间隙锁：**
- Next-Key Lock实现
- SERIALIZABLE隔离级别完成
- SSI死锁检测 < 100ms
- 幻读防护验证

**崩溃恢复：**
- 5种混沌注入场景全部可恢复
- S1: 崩溃前WAL写入
- S2: 崩溃后WAL写入，未提交
- S3: 预提交崩溃
- S4: 检查点崩溃
- S5: 撕裂页

**存储加密：**
- AES-256-GCM页面级加密
- `KeyProvider` trait（Env / File）
- 密钥轮换支持
- WAL加密

**细粒度权限：**
- 列级权限执行
- RBAC执行层（不仅是解析）
- 行级安全策略

**聚簇索引：**
- B+Tree聚簇叶子节点
- 主键直接存储在叶子节点
- 辅助索引指向主键
- 无主键表的隐藏主键（UUID）

#### 架构改进
- `bplus_tree/clustered_leaf.rs` — 聚簇叶子节点
- `transaction/gap_lock.rs` — GapLock类型
- `transaction/next_key_lock.rs` — Next-Key Lock算法
- `encryption/aes_cipher.rs` — AES-256-GCM
- `encryption/key_manager.rs` — 密钥管理
- `gmp/audit_chain.rs` — 不可变审计链

### 变更

- **覆盖率阈值**: GA 65%（从22%提升）
- **SQL语料库**: GA 98%（从95%提升）
- **形式化证明**: GA 30（从10提升）

### 修复

- `long_run_stability_test` — 所有 `#[ignore]` 已移除
- TPC-H SF=1 OOM — 通过增量数据生成解决
- 事务状态机压力测试 — 已实现

---

## v3.0.0 (2026-05-08)

### 新增功能
- WAL + MVCC（快照隔离 + SSI）
- Point SELECT 398K QPS
- CTE（WITH子句）
- 窗口函数 6/6
- `EXPLAIN` / `EXPLAIN ANALYZE`
- TLS/SSL（rustls）
- 慢查询日志
- Group Commit WAL
- 查询缓存（LRU + DML失效）
- 连接池（线程池）
- 在线DDL（ADD/DROP/MODIFY/RENAME）
- MySQL dump导出
- 30个形式化证明

### 已知问题
- TPC-H SF=1 OOM（在v3.1.0中已修复）
- INFORMATION_SCHEMA ~30%（在v3.1.0中已修复）
- SQL操作 20%（在v3.1.0中已修复）
- 间隙锁未实现（在v3.1.0中已修复）
- 无存储加密（在v3.1.0中已修复）
