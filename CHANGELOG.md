# 变更日志

SQLRustGo 的所有显着更改都将记录在此文件中。

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [开发中] v1.6.0

> 代号: 事务隔离 & 性能优化
> 目标: L3+ Transaction Ready

### Added

#### 事务支持 (T-01 ~ T-06)

- **T-01**: MVCC 骨架 (快照隔离、版本链管理)
- **T-02**: 事务管理器 (BEGIN/COMMIT/ROLLBACK)
- **T-03**: READ COMMITTED 隔离级别
- **T-04**: 行级锁 (排他锁、共享锁)
- **T-05**: 死锁检测
- **T-06**: SAVEPOINT 支持

#### WAL 改进 (W-01 ~ W-03)

- **W-01**: WAL 并发写入
- **W-02**: 检查点优化
- **W-03**: WAL 归档

#### 索引增强 (I-03 ~ I-06)

- **I-03**: 唯一索引
- **I-04**: 复合索引
- **I-05**: 索引统计
- **I-06**: 全文索引

#### 性能优化 (P-01 ~ P-04)

- **P-01**: 查询缓存
- **P-02**: 连接池
- **P-03**: TPC-H 基准测试
- **P-04**: SIMD 优化

#### 数据类型 (D-01 ~ D-04)

- **D-01**: DATE 类型
- **D-02**: TIMESTAMP 类型
- **D-03**: BLOB 类型
- **D-04**: BOOLEAN 增强

#### REPL 增强 (R-01 ~ R-04)

- **R-01**: .tables 命令
- **R-02**: .schema 命令
- **R-03**: .indexes 命令
- **R-04**: 语法高亮

### Performance 目标

- WAL 吞吐量: ≥500 MB/s (v1.5 基线 366 MB/s)
- TPC-H Q1: ≥1.5x (v1.5 基线)
- 事务并发: ≥3x (v1.5 基线)
- 查询缓存命中率: ≥80%

### Documentation

- **新**: v1.6.0 Release Notes
- **新**: v1.6.0 CHANGE_LOG
- **新**: v1.6.0 开发计划
- **新**: v1.6.0 发布门禁检查清单
- **新**: v1.6.0 迁移指南
- **新**: v1.6.0 API 变更文档

## [2.1.0] - 2026-04-03 (RC)

> 代号: Enterprise Observability
> 状态: RC - Release Candidate

### Added

#### 可观测性 (Observability)
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

#### SQL 功能扩展
- **#1210** TPC-H Phase 1: BETWEEN, DATE, IN
- **#1128** UUID 数据类型
- **#1128** ARRAY 数据类型
- **#1128** ENUM 数据类型

#### 存储过程
- **#1164** 存储过程控制流 (IF/WHILE/LOOP/DECLARE)
- **#1164** 存储过程 SQL 集成

### RC 阶段状态

| 检查项 | 状态 |
|--------|------|
| 单元测试 | ✅ 35/35 (100%) |
| 集成测试 | ⚠️ 1035/1039 (99.6%) |
| 覆盖率 | ⏳ ≥80% (待验证) |
| 性能测试 | ⏳ (待完成) |

### RC 后续任务

- [ ] 覆盖率提升至 ≥80%
- [ ] 修复 4 个预存测试失败
- [ ] 完成 TPC-H SF=0.1 完整测试
- [ ] 完成性能基准测试

---

## [1.5.0] - 2026-03-18 (GA)

### Added

- **架构**：存储引擎完整重构
  - 页式存储 (S-01)
  - 缓冲池 LRU 缓存 (S-02)
  - WAL 预写日志 (S-03)
  - 表堆存储 (S-04)
- **索引**：B+Tree 索引 (I-01) 和 IndexScan 算子 (I-02)
- **表达式**：常量折叠增强 (E-01) 和表达式简化 (E-02)
- **统计**：基础统计信息收集 (ST-01)
- **测试**：性能基准测试 (PB-01 ~ PB-05)
  - PB-01: 缓冲池命中率
  - PB-03: WAL 性能
  - PB-04: 页读写吞吐量
  - PB-05: TPC-H 查询
- **集成测试**：存储引擎和索引集成测试

### Performance

- WAL 吞吐量: 366 MB/s
- 顺序读: 3777 MB/s
- 缓冲池命中率: 100%
- 整体覆盖率: 90.46%

### Security

- **审核**: cargo audit 通过 (无漏洞)

### Documentation

- **新**: v1.5.0 Release Notes
- **新**: v1.5.0 性能测试报告
- **新**: v1.5.0 集成测试报告

## [1.5.0] - 2026-03-18 (GA)

### Added

- **架构**：存储引擎完整重构
  - 页式存储 (S-01)
  - 缓冲池 LRU 缓存 (S-02)
  - WAL 预写日志 (S-03)
  - 表堆存储 (S-04)
- **索引**：B+Tree 索引 (I-01) 和 IndexScan 算子 (I-02)
- **表达式**：常量折叠增强 (E-01) 和表达式简化 (E-02)
- **统计**：基础统计信息收集 (ST-01)
- **测试**：性能基准测试 (PB-01 ~ PB-05)
  - PB-01: 缓冲池命中率
  - PB-03: WAL 性能
  - PB-04: 页读写吞吐量
  - PB-05: TPC-H 查询
- **集成测试**：存储引擎和索引集成测试

### Performance

- WAL 吞吐量: 366 MB/s
- 顺序读: 3777 MB/s
- 缓冲池命中率: 100%
- 整体覆盖率: 90.46%

### Security

- **审核**: cargo audit 通过 (无漏洞)

### Documentation

- **新**: v1.5.0 Release Notes
- **新**: v1.5.0 性能测试报告
- **新**: v1.5.0 集成测试报告
>>>>>>> origin/main

## [1.4.0] - 2026-03-17

### Added

- **架构**：存储引擎完整重构
  - 页式存储 (S-01)
  - 缓冲池 LRU 缓存 (S-02)
  - WAL 预写日志 (S-03)
  - 表堆存储 (S-04)
- **索引**：B+Tree 索引 (I-01) 和 IndexScan 算子 (I-02)
- **表达式**：常量折叠增强 (E-01) 和表达式简化 (E-02)
- **统计**：基础统计信息收集 (ST-01)
- **测试**：性能基准测试 (PB-01 ~ PB-05)
  - PB-01: 缓冲池命中率
  - PB-03: WAL 性能
  - PB-04: 页读写吞吐量
  - PB-05: TPC-H 查询

### Performance

- WAL 吞吐量: 366 MB/s
- 顺序读: 3777 MB/s
- 缓冲池命中率: 100%

### Security

- **审核**: cargo audit 通过 (无漏洞)

## [1.3.0] - 2026-03-15

### Added

- **架构**：Volcano Executor trait 统一所有算子
- **架构**：L4 企业级架构升级
- **功能**：TableScan 算子 - 完整表扫描实现
- **功能**：Projection 算子 - 列投影功能
- **功能**：Filter 算子 - 条件过滤
- **功能**：HashJoin 算子 - 内连接实现
- **功能**：Executor 测试框架 (mock storage + 测试数据生成器)
- **功能**：Planner 测试框架 - Planner 完整测试套件
- **功能**：Metrics trait 定义 - 可观测性基础
- **功能**：/health/live 端点 - 存活探针
- **功能**：/health/ready 端点 - 就绪探针
- **功能**：BufferPoolMetrics 初步集成
- **测试**：整体行覆盖率 78.88% (目标 ≥65%)
- **测试**：Executor 行覆盖率 87.71% (目标 ≥60%)
- **测试**：Planner 行覆盖率 76.44% (目标 ≥60%)
- **测试**：Optimizer 行覆盖率 82.12% (目标 ≥40%)

### Changed

- **重构**：统一 Executor trait 接口
- **重构**：PhysicalPlan 迁移到 Volcano 模型
- **重构**：测试框架标准化

### Fixed

- **修复**：Clippy 警告全部清除
- **修复**：cargo fmt 格式问题
- **修复**：代码质量门禁全通过

### Security

- **审核**：依赖审计通过（无高危漏洞）

### Documentation

- **新**：v1.3.0 发布门禁检查清单
- **新**：v1.3.0 版本计划
- **新**：v1.3.0 开发计划

## [1.2.0] - 2026-03-13

### Added

- **架构**：带有 RecordBatch 的矢量化执行引擎
- **架构**：可插拔存储后端的 StorageEngine 特征
- **架构**：文件存储和内存存储实现
- **架构**：带有统计信息的基于成本的优化器（CBO）
- **功能**：用于统计收集的 ANALYZE 命令
- **功能**：带有表/列统计信息的简化 CBO
- **功能**：用于嵌入式使用的 LocalExecutor
- **功能**：HashJoinExec 表连接实现
- **功能**：聚合函数 (COUNT, SUM, AVG, MIN, MAX)
- **功能**：ProjectionExec 列投影 (Wildcard, Alias, BinaryExpr)
- **功能**：FilterExec 谓词过滤
- **功能**：SeqScanExec 全表扫描
- **优化器**：Predicate Pushdown 谓词下推
- **优化器**：Projection Pruning 投影裁剪
- **优化器**：Constant Folding 常量折叠
- **优化器**：Expression Simplification 表达式简化
- **优化器**：Join Reordering 连接重排序
- **测试**：LocalExecutor 测试框架
- **测试**：覆盖率提升至 80%+

### Changed

- **重构**：存储层抽象
- **重构**：统计基础设施
- **重构**：PhysicalPlan execute() 方法实现

### Fixed

- **修复**：Optimizer 规则测试编译错误
- **修复**：CI 配置完善 (release/* 分支触发)
- **修复**：Benchmark 编译适配

### Security

- **审核**：依赖审核已通过（无高严重性漏洞）

### Documentation

- **新**: v1.2.0 Release Notes
- **新增**：v1.2.0 升级指南
- **新**：v1.2.0 成熟度评估
- **新**：v1.2.0 测试计划（目标覆盖率超过 85%）
- **新增**：v1.2.0 性能测试报告
- **新增**：覆盖率改进计划

## [1.1.0] - 2026-03-05

### Added

- **架构**：查询执行的逻辑计划/物理计划分离
- **架构**：可插入执行器的 ExecutionEngine 特征
- **架构**：具有异步网络层的客户端-服务器架构
- **功能**：HashJoin 实现高效的连接操作
- **功能**：连接池支持多个客户端
- **功能**：WHERE 子句 AND/OR 逻辑运算符支持
- **功能**：BinaryOp 的表达式评估（+、-、*、/）
- **功能**：TEXT 列索引支持（基于哈希）
- **测试**：使用 Criterion 的性能基准框架
- **测试**：测试覆盖率提高至 90.66%

### Changed

- **重构**：用执行器中正确的错误传播替换了 unwrap
- **重构**：改进了 SqlResult<T> 的错误处理

### Fixed

- **修复**：已解决 Clippy 警告（零警告）
- **修复**：Rust 2021 兼容性（let 链语法）
- **修复**：代码格式问题

### Security

- **审核**：依赖审核已通过
- **审计**：无敏感信息泄露

## [1.0.0] - 2026-02-22

---

## 版本历史

| 版本 | 日期 | 成熟度 | 说明 |
|------|------|--------|------|
| **v1.6.0** | TBD | **L3+ 开发中** | **事务隔离、性能优化** |
| v1.5.0 | 2026-03-18 | L3+ GA | 存储引擎、索引、表达式优化 |
| v1.4.0 | 2026-03-17 | L3+ | 存储引擎重构、索引、表达式优化 |
| v1.3.0 | 2026-03-15 | L4 | 企业功能、可观察性 |
| v1.2.0 | 2026-03-13 | L3+ | 矢量化、CBO、存储抽象 |
| v1.1.0 | 2026-03-05 | L3 | 架构升级，Clippy通过 |
| v1.0.0 | 2026-02-22 | L3 GA | 初次发布 |

---

## 路线图

- **v1.6.0**: 🔨 开发中 (MVCC、事务隔离、WAL 改进)
- **v1.5.0**: ✅ GA 发布 (2026-03-18)
- **v2.0**: 分布式架构 (前置: v1.6.0 MVCC)

---

*此变更日志由 yinglichina8848 维护*
