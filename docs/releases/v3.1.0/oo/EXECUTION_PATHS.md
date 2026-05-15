# v3.1.0 纵向执行路径分析

> **版本**: v3.1.0 GA | **日期**: 2026-05-15
> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系, 300 执行流
> 从 SQL 输入到结果输出的完整纵向链路分析

---

## 1. SELECT 执行全链路

```
SQL String
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ 1. PARSER LAYER                                              │
│                                                               │
│  Lexer.tokenize(sql)                                         │
│    ├── 识别关键字: SELECT/FROM/WHERE/JOIN/ORDER BY           │
│    ├── 识别标识符: 表名/列名/别名                             │
│    ├── 识别字面量: 数字/字符串/日期                            │
│    └── 输出: Vec<Token>                                      │
│                                                               │
│  Parser.parse(tokens)                                        │
│    ├── SELECT 列解析 → Vec<SelectItem>                       │
│    ├── FROM 子句 → Vec<TableRef> (含 JOIN)                   │
│    ├── WHERE 子句 → Option<Expression>                       │
│    ├── GROUP BY → Vec<Expression>                            │
│    ├── HAVING → Option<Expression>                           │
│    ├── ORDER BY → Vec<OrderByItem>                           │
│    └── 输出: Statement::Select(SelectStatement)              │
│                                                               │
│  复杂度: O(n), n=SQL长度                                      │
│  瓶颈: 无                                                     │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. PLANNER LAYER                                              │
│                                                               │
│  Planner.plan(select_stmt)                                   │
│    ├── 构建逻辑计划:                                          │
│    │   Scan(table) → Filter(where) → Join(on)               │
│    │   → Aggregate(group_by) → Having → Project(select)     │
│    │   → Sort(order_by) → Limit                              │
│    └── 输出: LogicalPlan                                     │
│                                                               │
│  JOIN 规划 (v3.1.0 增强):                                     │
│    ├── JoinPlanner.split_predicates(where)                   │
│    │   ├── 跨表等值条件 → JoinPredicate                      │
│    │   └── 同表/非等值条件 → FilterPredicate                 │
│    ├── JoinPlanner.build_join_plan()                         │
│    │   ├── 贪心排序: 选择最小表为起始                          │
│    │   └── 逐步贪心: 选择使中间结果最小的下一表               │
│    └── 输出: JoinPlan { base_table, joins, filters }         │
│                                                               │
│  复杂度: O(t^2 * p), t=表数, p=谓词数                         │
│  瓶颈: 多表JOIN时贪心搜索                                     │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. OPTIMIZER LAYER (CBO)                                     │
│                                                               │
│  Optimizer.optimize(logical_plan)                            │
│    ├── 规则优化 (RBO):                                        │
│    │   ├── PredicatePushdown: 谓词下推到 Scan                │
│    │   ├── ProjectionPruning: 裁剪未使用列                   │
│    │   └── ConstantFolding: 常量折叠                         │
│    ├── 成本优化 (CBO):                                        │
│    │   ├── 统计信息: 行数/唯一值数/NULL比例                   │
│    │   ├── Join 顺序: DP 或贪心                              │
│    │   ├── 索引选择: 全表扫描 vs 索引扫描                     │
│    │   └── 算法选择: Hash vs NestedLoop vs Merge              │
│    └── 输出: PhysicalPlan                                    │
│                                                               │
│  复杂度: O(2^t) DP, O(t^2) 贪心                              │
│  瓶颈: 多表JOIN时DP搜索空间爆炸                                │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. EXECUTOR LAYER                                             │
│                                                               │
│  LocalExecutor.execute(physical_plan)                        │
│    ├── SeqScanExec: 顺序扫描表                               │
│    │   └── storage.scan(table) → Vec<Record>                 │
│    ├── IndexScanExec: 索引扫描 (v3.1.0 聚簇索引)             │
│    │   └── btree.range_search(low, high) → Vec<RowId>       │
│    ├── HashJoinExec: Hash Join                               │
│    │   ├── Build Phase: 小表构建 HashMap                     │
│    │   └── Probe Phase: 大表探测匹配                         │
│    ├── FilterExec: 应用 WHERE 条件                           │
│    ├── AggregateExec: GROUP BY + 聚合函数                    │
│    ├── WindowExec: 窗口函数 (v3.1.0)                         │
│    ├── SortExec: 排序 (ORDER BY)                             │
│    └── LimitExec: LIMIT 截断                                 │
│                                                               │
│  ParallelExecutor (v3.1.0 增强):                              │
│    ├── 分区扫描: 表按范围分区，并行执行                       │
│    ├── 并行 Hash Join: Build 后分区 Probe                     │
│    └── 结果合并: 合并分区结果                                 │
│                                                               │
│  复杂度: 取决于算子组合                                       │
│  瓶颈: Hash Join Build Phase 内存, 排序内存                  │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. STORAGE LAYER                                              │
│                                                               │
│  MemoryStorage / FileStorage                                  │
│    ├── scan(table) → 全表扫描                                 │
│    ├── index_scan(table, index, key) → 索引扫描              │
│    ├── ClusteredIndex (v3.1.0):                               │
│    │   ├── 主键即行指针                                       │
│    │   ├── 二级索引 → 主键 → 聚簇索引查找                    │
│    │   └── 范围查询优化 (物理有序)                            │
│    └── BufferPool → Page → B+Tree                            │
│                                                               │
│  复杂度: O(log n) 索引查找, O(n) 全表扫描                     │
│  瓶颈: BufferPool 淘汰, 磁盘IO                               │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │  ResultSet   │
                    └──────────────┘
```

---

## 2. INSERT + 触发器执行链路

```
INSERT INTO users (id, name, status) VALUES (1, 'Alice', 'active')
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. Parser → Statement::Insert                                │
│ 2. Planner → LogicalPlan::Insert                             │
│ 3. Executor.execute_insert()                                 │
│    ├── 检查约束:                                              │
│    │   ├── validate_primary_key() → 全表扫描 O(k) ⚠️         │
│    │   ├── validate_unique_constraints() → 全表扫描 O(k) ⚠️  │
│    │   └── validate_foreign_keys() → 全表扫描 O(k) ⚠️        │
│    ├── BEFORE INSERT 触发器:                                  │
│    │   ├── TriggerExecutor.execute_before_insert()            │
│    │   ├── 查找表上的 BEFORE INSERT 触发器                    │
│    │   ├── 执行触发器体 (可修改 NEW 行)                       │
│    │   └── 返回 ModifiedNewRow 或 Unmodified                  │
│    ├── 实际写入:                                              │
│    │   ├── storage.insert_row(table, row)                     │
│    │   ├── 更新聚簇索引 (v3.1.0)                              │
│    │   ├── 更新二级索引                                       │
│    │   ├── 更新全文索引 (FTS, v3.1.0)                         │
│    │   └── 写入 WAL                                          │
│    └── AFTER INSERT 触发器:                                   │
│        ├── TriggerExecutor.execute_after_insert()             │
│        └── 执行触发器体 (不可修改行)                          │
└──────────────────────────────────────────────────────────────┘
```

---

## 3. MERGE INTO 执行链路 (v3.1.0 新增)

```
MERGE INTO target t
  USING source s ON t.id = s.id
  WHEN MATCHED THEN UPDATE SET t.name = s.name
  WHEN NOT MATCHED THEN INSERT (id, name) VALUES (s.id, s.name)
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. Parser → Statement::Merge (已实现)                        │
│ 2. Planner → LogicalPlan::Merge (已实现)                     │
│ 3. Executor → ⚠️ 未实现                                      │
│                                                               │
│ 预期执行流程:                                                 │
│    ├── 扫描 source 表                                        │
│    ├── 对每行:                                               │
│    │   ├── 在 target 中查找匹配 (ON 条件)                    │
│    │   ├── MATCHED → UPDATE target row                       │
│    │   └── NOT MATCHED → INSERT into target                  │
│    └── 返回合并统计                                           │
│                                                               │
│ 状态: Parser ✅ | Planner ✅ | Executor ❌                    │
│ 预计工作量: 中 (1-2周)                                        │
└──────────────────────────────────────────────────────────────┘
```

---

## 4. 事务执行链路 (SSI)

```
BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ TransactionManager.begin(IsolationLevel::Serializable)       │
│    ├── 生成 TxId                                             │
│    ├── 创建 Snapshot (commit_ts + active_tx_list)            │
│    └── 初始化 SsiDetector (v3.1.0)                           │
│                                                               │
│ SELECT ... (读操作)                                           │
│    ├── MVCCStorage.read(key, snapshot)                       │
│    │   ├── 遍历 VersionChain                                 │
│    │   ├── is_visible() 可见性判断                            │
│    │   └── 返回可见版本                                       │
│    └── SsiDetector.record_read(tx_id, key) ← v3.1.0         │
│                                                               │
│ INSERT/UPDATE (写操作)                                        │
│    ├── MVCCStorage.write_version(key, value, tx_id)          │
│    │   └── 追加新版本到 VersionChain                          │
│    ├── SsiDetector.record_write(tx_id, key) ← v3.1.0        │
│    └── 获取分布式锁 ← v3.1.0                                 │
│                                                               │
│ COMMIT                                                        │
│    ├── SsiDetector.validate_commit(tx_id) ← v3.1.0          │
│    │   ├── 检查 RW 冲突: 我的读集 ∩ 他人写集                  │
│    │   ├── 检查 WR 冲突: 我的写集 ∩ 他人读集                  │
│    │   └── 如果同时存在 → RW-WR 循环 → SsiError              │
│    ├── MvccEngine.commit_transaction(tx_id)                  │
│    │   └── commit_versions(tx_id, commit_ts)                 │
│    ├── WAL.write_commit_record(tx_id)                        │
│    └── SsiDetector.release(tx_id) ← v3.1.0                  │
│                                                               │
│ ROLLBACK                                                      │
│    ├── rollback_versions(tx_id)                              │
│    └── SsiDetector.release(tx_id) ← v3.1.0                  │
└──────────────────────────────────────────────────────────────┘
```

---

## 5. MySQL 协议完整链路

```
MySQL Client (mysql crate / mysql-cli / sysbench)
    │
    ▼ TCP connect
┌──────────────────────────────────────────────────────────────┐
│ mysql-server::run_server()                                   │
│    ├── 监听 TCP 端口 (默认 3306)                              │
│    └── TLS 配置 (rustls 自签名证书)                           │
│                                                               │
│ handle_connection(stream)                                    │
│    ├── Phase 1: TLS 握手 (可选)                               │
│    ├── Phase 2: MySQL 握手                                    │
│    │   ├── 发送 HandshakePacket (Protocol 10)                │
│    │   ├── 接收 HandshakeResponse                            │
│    │   └── verify_mysql_native_password()                    │
│    └── Phase 3: 命令循环                                      │
│         ├── COM_QUERY:                                       │
│         │   ├── split_sql_statements(sql)                    │
│         │   ├── replace_placeholders(sql, params)            │
│         │   ├── engine.execute(sql)                          │
│         │   └── send_result_set(rows) ← 逐行 flush ⚠️       │
│         ├── COM_STMT_PREPARE:                                │
│         │   ├── parse(sql) → Statement                       │
│         │   ├── 计算参数数和列数                              │
│         │   └── 返回 statement_id                            │
│         ├── COM_STMT_EXECUTE:                                │
│         │   ├── parse_stmt_execute_params()                  │
│         │   ├── replace_placeholders()                       │
│         │   └── execute + send_result                        │
│         ├── COM_PING → OK                                    │
│         ├── COM_INIT_DB → switch database                   │
│         └── COM_QUIT → close connection                     │
└──────────────────────────────────────────────────────────────┘
```

---

## 6. 全文搜索执行链路

```
CREATE FULLTEXT INDEX idx_content ON articles(content)
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. DDL 执行: CREATE INDEX                                    │
│ 2. 扫描表所有行                                              │
│ 3. 对每行:                                                   │
│    ├── MultiLanguageTokenizer.tokenize(content)              │
│    │   ├── SimpleTokenizer: lowercase + 去停用词             │
│    │   └── ChineseTokenizer: unigram + bigram                │
│    └── InvertedIndex.add_document(row_id, tokens)            │
│         └── index[token].insert(row_id)                      │
└──────────────────────────────────────────────────────────────┘

SELECT * FROM articles WHERE MATCH(content) AGAINST('数据库系统')
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. Parser: 识别 MATCH ... AGAINST 语法                       │
│ 2. Executor: 调用 InvertedIndex.search()                     │
│    ├── tokenize("数据库系统")                                 │
│    │   → ["数据", "据库", "库系", "系统", "数据库", "库系统"] │
│    ├── AND 搜索: 所有 token 的 doc_id 交集                   │
│    └── 返回匹配行                                             │
│ 3. Fetch rows by matched doc_ids                             │
│ 4. 返回结果集                                                │
└──────────────────────────────────────────────────────────────┘
```

---

## 7. UPDATE 执行链路

```
UPDATE orders SET total = 100 WHERE id = 1
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. Parser → Statement::Update                                │
│ 2. Planner → LogicalPlan::Update                             │
│ 3. Executor.execute_update()                                 │
│    ├── BEFORE UPDATE 触发器:                                  │
│    │   ├── TriggerExecutor.execute_before_update()            │
│    │   ├── OLD = 旧行, NEW = 新行                            │
│    │   ├── 执行触发器体 (可修改 NEW 行)                       │
│    │   └── 返回 ModifiedNewRow 或 Unmodified                  │
│    ├── 存储层更新:                                            │
│    │   ├── storage.update(table, filters, updates)            │
│    │   ├── 如果在事务中: WalStorage.log_update()              │
│    │   ├── MVCC: 追加新版本到 VersionChain                    │
│    │   └── 更新索引 (B+Tree delete + insert)                  │
│    ├── AFTER UPDATE 触发器:                                   │
│    │   └── TriggerExecutor.execute_after_update()             │
│    └── 审计日志: AuditLogger.log_update()                     │
└──────────────────────────────────────────────────────────────┘
```

---

## 8. DELETE 执行链路

```
DELETE FROM orders WHERE id = 1
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. Parser → Statement::Delete                                │
│ 2. Planner → LogicalPlan::Delete                             │
│ 3. Executor.execute_delete()                                 │
│    ├── BEFORE DELETE 触发器:                                  │
│    │   └── TriggerExecutor.execute_before_delete()            │
│    ├── 存储层删除:                                            │
│    │   ├── storage.delete(table, filters)                     │
│    │   ├── 如果在事务中: WalStorage.log_delete()              │
│    │   ├── MVCC: 追加 RowVersion::new_deleted(tx_id)          │
│    │   └── 更新索引 (B+Tree delete)                           │
│    ├── AFTER DELETE 触发器:                                   │
│    │   └── TriggerExecutor.execute_after_delete()             │
│    └── 审计日志: AuditLogger.log_delete()                     │
└──────────────────────────────────────────────────────────────┘
```

---

## 9. DDL 建表执行链路

```
CREATE TABLE users (
  id INT PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  email VARCHAR(255) UNIQUE
)
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. Parser → Statement::CreateTable                           │
│    ├── 列定义转换: Parser ColumnDef → Storage ColumnDef       │
│    └── 构建 TableInfo { name, columns, primary_key, ... }    │
│                                                               │
│ 2. 存储层建表                                                 │
│    ├── storage.create_table(&table_info)                      │
│    ├── MemoryStorage: HashMap 插入                            │
│    ├── FileStorage: 创建 TableData, 持久化到磁盘              │
│    └── WalStorage: 透传到 inner                               │
│                                                               │
│ 3. Binlog 复制 (如启用)                                       │
│    └── MasterNode.write_ddl(tx_id, table_id, db, table, sql) │
│                                                               │
│ 4. 审计日志                                                   │
│    └── record_ddl_audit(storage, tx_id, user, sql)            │
└──────────────────────────────────────────────────────────────┘
```

---

## 10. 崩溃恢复执行链路

```
Server Restart after crash
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. WalManager.recover()                                      │
│    ├── 读取 WAL 文件                                          │
│    ├── 逐条解析 WalEntry                                      │
│    └── 返回 Vec<WalEntry>                                     │
│                                                               │
│ 2. Recovery.scan_incomplete_transactions()                   │
│    ├── 构建 tx_map: HashMap<tx_id, TxState>                  │
│    ├── 有 Prepare 无 Commit/Rollback → 2PC 恢复              │
│    └── 无 Prepare/Commit/Rollback → 需要回滚                 │
│                                                               │
│ 3. WalStorage.recover()                                      │
│    ├── 已提交事务: 重放 Insert/Update/Delete                  │
│    └── 未提交事务: 忽略 (或回滚)                              │
│                                                               │
│ 4. 恢复验证                                                   │
│    ├── manifest_valid: RecoveryManifest 校验                  │
│    ├── wal_chain_valid: WalChainState 校验                    │
│    └── page_checksums_valid: PageChecksumStore                │
│                                                               │
│ 5. PITR 时间点恢复 (可选)                                     │
│    └── PitrRecovery.recover_to_timestamp(target_ts)           │
│                                                               │
│ 6. Clustered Index 恢复                                       │
│    └── ClusteredWalEntry 解码, 恢复页面状态                   │
└──────────────────────────────────────────────────────────────┘
```

---

## 11. 存储过程执行链路

```
CALL compute_order_stats(2024)
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. Parser → Statement::Call                                  │
│ 2. StoredProcExecutor.execute_call("compute_order_stats", args)│
│    ├── Catalog.get_stored_procedure("compute_order_stats")    │
│    └── 返回 StoredProcedure { body, params }                  │
│                                                               │
│ 3. 创建 ProcedureContext                                      │
│    ├── 参数绑定: args → ctx.set_var()                         │
│    └── 初始化: local_variables, scope_stack, handler_stack    │
│                                                               │
│ 4. 逐条执行 StoredProcStatement                              │
│    ├── DECLARE → ctx.set_var(name, default)                   │
│    ├── SET → evaluate_expression() → ctx.set_var()            │
│    ├── IF → evaluate_condition() → 分支执行                   │
│    ├── WHILE → 循环执行                                       │
│    ├── RawSql → expand_variables_in_sql() → execute_sql()     │
│    ├── SELECT INTO → 执行 SELECT, 结果写入变量                │
│    ├── LEAVE/ITERATE → 循环控制                               │
│    ├── RETURN → 退出并返回值                                  │
│    ├── SIGNAL → 异常抛出, 检查 handler_stack                  │
│    └── CALL → 递归调用其他存储过程                            │
│                                                               │
│ 5. 异常处理                                                   │
│    ├── SQLSTATE 匹配                                          │
│    ├── CONTINUE handler → 继续下一条语句                      │
│    └── EXIT handler → 退出当前 Block                          │
└──────────────────────────────────────────────────────────────┘
```

---

## 12. 分布式 2PC 执行链路

```
Coordinator: PREPARE distributed transaction
    │
    ▼
┌──────────────────────────────────────────────────────────────┐
│ 1. Coordinator 发送 Prepare                                   │
│    └── PrepareRequest { gid, coordinator_node_id, changes }   │
│                                                               │
│ 2. Participant 投票                                           │
│    ├── Participant.handle_prepare()                           │
│    ├── 获取锁 → 创建事务上下文                                │
│    ├── WAL: log_prepare(tx_id)                                │
│    └── 返回 VoteResponse { VoteCommit / VoteAbort }           │
│                                                               │
│ 3. Coordinator 收集投票                                       │
│    ├── 全部 VoteCommit → 发送 CommitRequest                   │
│    └── 任一 VoteAbort → 发送 RollbackRequest                  │
│                                                               │
│ 4. Participant 执行决定                                       │
│    ├── handle_commit(): WAL log_commit, 释放锁, 清理          │
│    └── handle_rollback(): WAL log_rollback, 释放锁, 清理      │
└──────────────────────────────────────────────────────────────┘
```

---

## 执行路径总览

| # | 执行路径 | 入口 | 核心模块 | 步骤数 | 关键瓶颈 |
|---|----------|------|----------|--------|---------|
| 1 | SELECT | ExecutionEngine::execute | parser→planner→optimizer→executor | 9 | Sort/Limit 未实现 |
| 2 | INSERT+触发器 | execute_insert | parser→trigger→constraint→storage→WAL | 10 | 约束全表扫描 O(k) |
| 3 | MERGE INTO | ⚠️ 未实现 | parser→planner (executor gap) | - | Executor 未实现 |
| 4 | 事务(SSI) | TransactionManager::begin | tx_mgr→SSI→MVCC→WAL→lock | 4 | SSI 未实现 |
| 5 | MySQL协议 | handle_connection | TCP→handshake→command→execute | 7 | 逐行 flush 109x |
| 6 | 全文搜索 | InvertedIndex::search | tokenizer→index→fetch | 4 | 模糊搜索 O(T) |
| 7 | UPDATE | execute_update | parser→trigger→storage→WAL | 8 | 触发器解释执行 |
| 8 | DELETE | execute_delete | parser→trigger→storage→MVCC→WAL | 7 | MVCC 版本链膨胀 |
| 9 | DDL建表 | storage.create_table | parser→TableInfo→storage→binlog | 8 | 无在线DDL |
| 10 | 崩溃恢复 | WalManager::recover | WAL→tx_state→replay→verify | 8 | PITR 全量扫描 |
| 11 | 存储过程 | StoredProcExecutor::execute_call | catalog→context→body→exception | 6 | 解释执行无预编译 |
| 12 | 分布式2PC | Participant::handle_prepare | prepare→vote→commit/rollback | 4 | 同步阻塞 |

---

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v1.0 | v3.1.0 纵向执行路径分析，含6条核心链路 |
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，扩展至12条链路，补充执行路径总览 |
