# SQLRustGo v2.1.0 Changelog

## [v2.1.0-rc] - 2026-04-03

### RC 阶段变更

#### 合并更新
- **develop/v2.1.0** 完全合并到 **rc/v2.1.0**
- 解决多文件合并冲突 (tpch_compliance_test.rs, commands.rs)
- 添加 TPC-H 合规测试基础设施

#### 已知问题
- 4 个预存测试失败 (不阻塞 RC)
  - test_batch_insert_mixed_columns
  - test_auto_increment_with_explicit_value
  - test_teaching_having
  - test_regression_suite

### RC 后续任务
- [ ] 覆盖率提升至 ≥80%
- [ ] 修复 4 个预存测试失败
- [ ] 完成 TPC-H SF=0.1 完整测试
- [ ] 完成性能基准测试

---

## [v2.1.0-beta] - 2026-04-01

### Beta 阶段完成

### Added

#### 可观测性 (Observability)
- **#1137** 测试覆盖率提升 - 目标 80%
- **M-001** Metrics trait 定义
- **M-002** Buffer Pool Metrics 指标
- **M-003** Executor Metrics 执行器指标
- **M-004** `/metrics` Prometheus 端点
- **M-005** Grafana Dashboard 配置
- **M-006** 慢查询日志 (slow_query_log)

#### SQL Firewall 安全
- **#1134** SQL 防火墙核心模块
- **#1134** 告警系统
- **#1135** KILL 语句支持
- **#1135** PROCESSLIST 显示

#### 工具链 (Tools)
- **#1018** Physical Backup CLI (全量/增量备份)
- **#1198** 备份保留策略 (prune --keep/--keep-days)
- **#1022** mysqldump 导入导出工具
- **#1022** 日志轮转 (log rotation)
- **#1023** LRU Cache 优化 O(n) → O(1)

#### SQL 功能扩展
- **#1210** TPC-H Phase 1: BETWEEN 操作符
- **#1210** TPC-H Phase 1: DATE 字面量
- **#1210** TPC-H Phase 1: IN value list
- **#1128** UUID 数据类型
- **#1128** ARRAY 数据类型
- **#1128** ENUM 数据类型

#### 存储过程
- **#1164** 存储过程控制流 (IF/WHILE/LOOP/DECLARE)
- **#1164** 存储过程 SQL 集成

#### AgentSQL Extension
- **#1128** Phase 1: agentsql-core 框架 + gateway
- **#1128** Phase 2: Enhanced schema + stats API
- **#1128** Phase 3: NL2SQL + Memory modules
- **#1128** Phase 3: Security, Explain, Optimizer modules
- **#1128** Phase 4: OpenClaw TypeScript plugin

### Changed

- MockStorage 已废弃，使用 MemoryStorage 进行测试
- LRU Cache 从 O(n) 优化到 O(1)
- 集成测试注册数量增加 (70+ 测试文件)

### Fixed

- **#1190** LRU cache O(n) → O(1) 优化
- **#1195** 修复 reserved keyword 问题
- **#1193** 修复 CacheEntry.last_access 字段
- **#1180** 修复 duplicate struct definitions
- **#1177** 存储过程控制流改进

---

## [v2.0.0] - 2026-03-29

### Added

#### Phase 1: 存储稳定性
- **#942** WAL 回放完整实现
- **#952** 性能基准工具搭建 (sysbench)
- **#963** 内存管理优化 - Arena/Pool
- **#964** 批量写入优化 - INSERT batch
- **#965** WAL 组提交优化
- **#975** 任务调度器 - TaskScheduler
- **#976** 并行执行器 - ParallelExecutor
- **#987** Page Checksum 完整集成
- **#988** Catalog 系统完整集成
- **#989** EXPLAIN 算子覆盖扩展

#### Phase 2: 高可用
- **#953** 主从复制 - Binlog/故障转移
- **#955** 窗口函数实现 - ROW_NUMBER/RANK/SUM OVER
- **#956** RBAC 权限系统 - 用户/角色/GRANT

#### Phase 3: 分布式能力
- **#944** Phase 3: 分布式能力 - Sharding/分布式事务
- **#1091** Participant WAL 集成
- **#1092** Recovery WAL 集成
- **#1087** Coordinator gRPC 调用

#### Phase 4: 安全与治理
- **#945** Phase 4: 安全与治理 - RBAC/SSL/审计

#### Phase 5: 性能优化
- **#946** Phase 5: 性能优化 - 向量化/CBO/列式存储

#### Epic-12: 列式存储
- **#753** ColumnChunk 数据结构
- **#754** ColumnSegment 磁盘布局
- **#755** ColumnarStorage 存储引擎
- **#756** Projection Pushdown 优化器
- **#757** ColumnarScan 执行器节点
- **#758** Parquet 导入导出

### Changed

- 升级 arrow v52 → v53 (解决 chrono quarter() 冲突)
- 优化列式存储内存使用
- 改进并行执行器调度算法

### Fixed

- **#1104** 修复 arrow v52 chrono 冲突
- **#1106** 修复 workspace build errors
- **#1096** 修复 2PC integration test errors

---

## [v1.0.0] - 2026-02-20

### Added
- 首个 GA 版本
- 完整 SQL-92 子集支持
- 基础存储引擎
- REPL 界面

---

*Changelog generated from Git history*
