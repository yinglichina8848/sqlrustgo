# SQLRustGo v2.0.0 Changelog

## [v2.0.0] - 2026-03-29

### Added

#### Phase 1: 存储稳定性
- **#942** WAL 回放完整实现
- **#952** 性能基准工具搭建 (sysbench)
- **#953** 主从复制 - Binlog/故障转移
- **#954** 并行查询框架 - ParallelExecutor
- **#963** 内存管理优化 - Arena/Pool
- **#964** 批量写入优化 - INSERT batch
- **#965** WAL 组提交优化
- **#966** 主从复制原型
- **#975** 任务调度器 - TaskScheduler
- **#976** 并行执行器 - ParallelExecutor
- **#987** Page Checksum 完整集成
- **#988** Catalog 系统完整集成
- **#989** EXPLAIN 算子覆盖扩展

#### Phase 2: 高可用
- **#943** Phase 2: 高可用 - 主从复制/备份/故障转移
- **#955** 窗口函数实现 - ROW_NUMBER/RANK/SUM OVER
- **#956** RBAC 权限系统 - 用户/角色/GRANT

#### Phase 3: 分布式能力
- **#944** Phase 3: 分布式能力 - Sharding/分布式事务
- **#1091** Participant WAL 集成
- **#1092** Recovery WAL 集成
- **#1087** Coordinator gRPC 调用

#### Phase 4: 安全与治理
- **#945** Phase 4: 安全与治理 - RBAC/SSL/审计
- **#885** 高可用与数据可靠性
- **#886** 安全与权限管理
- **#1093** Phase 4 security - audit, session, TLS

#### Phase 5: 性能优化
- **#946** Phase 5: 性能优化 - 向量化/CBO/列式存储

#### Epic-12: 列式存储
- **#753** ColumnChunk 数据结构
- **#754** ColumnSegment 磁盘布局
- **#755** ColumnarStorage 存储引擎
- **#756** Projection Pushdown 优化器
- **#757** ColumnarScan 执行器节点
- **#758** Parquet 导入导出
- **#1095** ParquetCompat 格式
- **#1099** COPY 语句支持

#### Epic-16: 迁移
- **#840** 全面评估报告 & 后续任务
- **#848** 数据库内核工程化强化计划
- **#887** DeepSeek 审核整改计划
- **#974** Batch insert optimization
- **#972** WAL Group Commit

### Changed

- 升级 arrow v52 → v53 (解决 chrono quarter() 冲突)
- 优化列式存储内存使用
- 改进并行执行器调度算法

### Fixed

- **#1104** 修复 arrow v52 chrono 冲突
- **#1105** 修复 workspace build errors
- **#1106** 修复 workspace build errors (merge)
- **#1096** 修复 2PC integration test errors
- **#1094** 修复 columnar storage compilation errors
- **#1090** 修复 columnar build issues
- **#1088** 修复 rebase corruption

---

## [v1.2.0] - 2026-03-15

### Added
- 架构重构文档
- 分支管理计划
- 性能分析报告

### Changed
- 优化器增强
- 更多 JOIN 类型支持

---

## [v1.1.0] - 2026-03-03

### Added
- LogicalPlan/PhysicalPlan 分离
- ExecutionEngine 插件化
- Client-Server 架构
- HashJoin 实现

### Changed
- 测试覆盖率提升至 93%+
- Criterion 基准测试框架

---

## [v1.0.0] - 2026-02-20

### Added
- 首个 GA 版本
- 完整 SQL-92 子集支持
- 基础存储引擎
- REPL 界面

---

*Changelog generated from Git history*
