# 变更日志

SQLRustGo 的所有显着更改都将记录在此文件中。

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [未发布]

### Added

- **版本**: alpha/v2.6.0

## [2.6.0] - TBD (Alpha)

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
