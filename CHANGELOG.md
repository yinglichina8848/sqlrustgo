# 变更日志

SQLRustGo 的所有显着更改都将记录在此文件中。

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.2.0] - 2026-05-18 (RC → GA)

### 目标

Trust Convergence（可信收敛）版本，聚焦 GMP 工业标准验证，确保：
- MySQL 协议完整兼容
- 性能稳定无回归
- 审计链完整性验证
- Crash Recovery 验证
- GMP Long-Run 稳定性

### 核心原则

> **禁止架构扩散，聚焦可信收敛**

### 已完成

| 功能 | PR | 状态 |
|------|-----|------|
| UPDATE/DELETE WHERE 子句修复 | #1174 | ✅ |
| 性能回归调查（无回归发现） | #1174 | ✅ |
| Audit Chain Validator 增强 | #1180 | ✅ |
| WAL Crash Recovery 测试 | #1168 | ✅ |
| Crash Recovery 验证 | #1166 | ✅ |
| GMP Timestamp 验证 | #1171 | ✅ |

### 性能数据

| 操作 | v3.2.0 实测 | v3.0.0 基线 | 提升 |
|------|------------|------------|------|
| UPDATE | 109,988 QPS | 43,121 QPS | +155% |
| DELETE | 134,312 QPS | 64,896 QPS | +107% |
| INSERT | 73,261 QPS | 28,698 QPS | +155% |

### GMP 审计链增强

- `verify_chain()` 新增时间戳单调递增验证
- `verify_chain()` 新增事务 ID 追踪（孤立条目检测）
- `AuditChainError` 新增 5 个变体：`TimestampNotMonotonic`, `SignatureInvalid`, `OrphanEntry`, `WorkflowLinkBroken`, `ProvenanceIncomplete`
- CLI `audit-chain-verify` 处理所有新错误类型

### Bug 修复

- **UPDATE WHERE 子句被忽略**: `expression_to_value()` 不支持行上下文，导致 WHERE 条件无法求值。添加 `evaluate_row_expression()` 方法支持行上下文和列名→索引映射。
- **DELETE WHERE 子句被忽略**: 同上，使用 `get_table_records_mut()` + 索引收集实现正确的条件过滤。

## [2.8.0] - 2026-05-01 (GA)

### 目标

生产化+分布式+安全版本，MySQL 5.7 功能覆盖率 92%，分布式能力（分区表、主从复制、故障转移），安全性评分 92%。

### 已完成

| 功能 | PR | 状态 |
|------|-----|------|
| PR 100 CASE WHEN NULLIF | #100 | ✅ |
| PR 101 clippy fix | #101 | ✅ |
| PR 102 OFFSET/LIMIT | #102 | ✅ |
| Git 历史清理 (target/ 移除) | #103 | ✅ |
| 仓库压缩 62.8GB→95MB | #103 | ✅ |
| SSH fetch 修复 (3秒) | #103 | ✅ |

### 发布 PR

- [#103](https://192.168.0.252:3000/openclaw/sqlrustgo/pulls/103) - chore: R-Gate cleanup

## [2.7.0] - 2026-04-22 (GA)

### 目标

企业级韧性版本，实现 WAL 崩溃恢复、外键稳定性增强、备份恢复机制、审计证据链等企业级功能。

### 已完成

| 功能 | PR | 状态 |
|------|-----|------|
| T-01 事务/WAL 恢复 | - | ✅ |
| T-02 FK/约束稳定化 | - | ✅ |
| T-03 备份恢复演练 | - | ✅ |
| T-04 qmd-bridge 统一检索层 | #1713 | ✅ |
| T-05 统一检索 API (lex/vec/graph/hybrid) | #1714 | ✅ |
| T-06 混合检索重排 (RRF/Linear/Composite) | #1714 | ✅ |
| T-07 GMP Top 10 审核查询模板 | #1714 | ✅ |
| T-08 审计证据链 (防篡改哈希链) | #1718 | ✅ |

### 发布 PR

- [#1729](https://github.com/minzuuniversity/sqlrustgo/pull/1729) - chore: v2.7.0 GA release

## [2.6.0] - 2026-04-22 (GA)

### 目标

生产就绪版本，实现 SQL-92 完整支持。

### 已完成 (2026-04-18)

| 功能 | PR | 状态 |
|------|-----|------|
| SQL-92 聚合函数 | #1545 | ✅ |
| SQL-92 JOIN 语法 | #1545 | ✅ |
| SQL-92 GROUP BY | #1545 | ✅ |
| SQL-92 HAVING 子句 | #1567 | ✅ |
| DELETE 语句 | #1557 | ✅ |
| 外键约束 | #1436, #1567 | ✅ |
| 集成测试修复 | #1561 | ✅ |
| 覆盖率测试提升 | #1559, #1564 | ✅ |
| ExecutionEngine API | #1566 | ✅ |
| Clippy 零警告 | #1570 | ✅ |
| SQL Corpus 100% | - | ✅ (59/59) |

### P0 功能 (进行中)

| 功能 | 状态 | Issue |
|------|--------|-------|
| 功能集成 (索引扫描、CBO、存储过程、触发器、WAL) | 进行中 | #1497 |
| MVCC SSI (可串行化快照隔离) | 待开发 | #1389 |

### P1 功能

| 功能 | Issue |
|------|-------|
| FULL OUTER JOIN | #1380 |

### 文档

- **新**: v2.6.0 发布文档目录
- **新**: VERSION_PLAN.md - 版本计划
- **新**: RELEASE_GATE_CHECKLIST.md - 门禁检查清单
- **新**: TEST_PLAN.md - 测试计划
- **新**: INTEGRATION_STATUS.md - 功能集成状态
- **新**: PERFORMANCE_TARGETS.md - 性能目标
- **新**: SQL_REGRESSION_PLAN.md - SQL 回归测试计划
- **新**: INTEGRATION_TEST_PLAN.md - 集成测试计划

## [2.5.0] - 2026-04-16

### Added

- **架构**: MVCC 并发控制 - 快照隔离实现
- **架构**: WAL 启用验证 - 崩溃恢复测试
- **功能**: 语义嵌入 API - 替换 HashEmbedding
- **功能**: 分布式存储 - ShardGraph/ShardVector
- **功能**: Cost-based Optimizer - CBO 实现
- **功能**: Prepared Statement - 参数化查询
- **功能**: 连接池 - Connection Pool 实现
- **功能**: 子查询优化 - EXISTS/IN/ANY/ALL
- **功能**: Cypher 查询语言 - 图查询子集实现
- **功能**: JOIN 完整实现 - LEFT/RIGHT/CROSS JOIN
- **功能**: Graph 持久化 - DiskGraphStore 实现
- **功能**: 事务系统集成 - MVCC + WAL 到 SQL 执行路径
- **功能**: 全模块集成测试 - SQL+Vector+Graph 混合负载
- **功能**: 图查询性能基准 - BFS/DFS/多跳查询
- **功能**: 向量检索性能基准 - 10万/100万向量 KNN
- **功能**: TPC-H SF=10 性能测试
- **功能**: GMP 内审支持 - 审计日志+合规检查+内审报表
- **功能**: OpenClaw 全局调度 - 任务流编排+Agent 协作
- **功能**: 统一优化器 - CBO 自动选择执行路径
- **功能**: 统一存储层 - Document 表+向量+图索引联动
- **功能**: 统一查询 API - SQL+Vector+Graph 联合查询

### 测试结果

| 测试套件 | 结果 |
|----------|------|
| Storage lib tests | 55/55 ✅ |
| Parser tests | 37/37 ✅ |
| Executor lib tests | 9/9 ✅ |
| sql-corpus | 59 cases, 12 passed (20.3%) |
| 整体覆盖率 | 49% |

## [2.4.0] - 2026-04-08

### Added

- **架构**: SIMD 加速全面化
- **架构**: 向量存储层整合：BinaryStorage + B+Tree + mmap
- **功能**: 查询计划器 - 自动选择索引
- **功能**: 列式存储压缩 (LZ4/Zstd)
- **功能**: Hash 索引支持
- **功能**: 内存映射存储 (mmap)
- **功能**: SIMD 向量计算加速

### 性能

- **优化**: TPC-H SF=1 性能基准测试
- **优化**: v2.2+v2.3 整合测试

## [2.0.0] - 2026-03-25

### Added

- **架构**: 异步网络层
- **功能**: 客户端-服务器架构
- **功能**: 连接池支持多个客户端

---

## 版本历史

| 版本 | 日期 | 成熟度 | 说明 |
|------|------|--------|-------|
| v2.6.0 | TBD | Alpha | 生产就绪、SQL-92 完整 |
| v2.5.0 | 2026-04-16 | Alpha | MVCC、Vector/Graph、统一查询 |
| v2.4.0 | 2026-04-08 | Alpha | SIMD、列式存储、压缩 |
| v2.0.0 | 2026-03-25 | Alpha | 异步网络、连接池 |
| v1.1.0 | 2026-03-05 | Alpha | 架构升级、Clippy 通过 |
| v1.0.0 | 2026-02-22 | GA | 初次发布 |

---

## 路线图

- **v2.6.0**: 生产就绪版本 (当前开发)
- **v2.5.0**: MVCC、Vector/Graph 存储
- **v2.4.0**: SIMD 加速、列式存储
- **v2.0.0**: 异步网络架构
- **v1.1.0**: 架构升级
- **v1.0.0**: 初始版本

---

*此变更日志由 yinglichina8848 维护*
