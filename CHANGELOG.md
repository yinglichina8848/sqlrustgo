# 变更日志

> SQLRustGo 所有显著更改记录
> 
> 基于 [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) 格式
> 遵循 [语义化版本 (Semantic Versioning)](https://semver.org/spec/v2.0.0.html)

---

## [未发布] - v2.6.0 (Alpha)

> **目标**: 生产就绪版本，SQL-92 完整支持
> **阶段**: Alpha
> **发布日期**: TBD (目标 2026-05-12)

### 已完成功能

| 功能 | 状态 | 说明 |
|------|------|------|
| SQL-92 聚合函数 | ✅ | SUM/COUNT/AVG/MIN/MAX |
| SQL-92 JOIN 语法 | ✅ | INNER/LEFT/RIGHT/CROSS |
| SQL-92 GROUP BY | ✅ | 分组查询 |
| SQL-92 HAVING 子句 | ✅ | 分组过滤 |
| DELETE 语句 | ✅ | 删除操作 |
| 外键约束 | ✅ | CASCADE/SET NULL |
| 集成测试修复 | ✅ | 测试稳定性 |
| 覆盖率提升 | ✅ | 49% → 55% |
| ExecutionEngine API | ✅ | 统一执行接口 |
| Clippy 零警告 | ✅ | 代码质量 |
| SQL Corpus | ✅ | 59/59 通过 |
| 文档治理 | ✅ | 链接修复、规范 |

### 进行中

| 功能 | Issue | 状态 |
|------|-------|------|
| MVCC SSI (可串行化快照隔离) | #1389 | 进行中 |
| 存储过程 | #1384 | 进行中 |
| 触发器 | #1385 | 进行中 |

---

## [2.5.0] - 2026-04-16

> **发布**: GitHub Release v2.5.0
> **阶段**: Alpha → 已归档

### 新增功能

- **MVCC**: 快照隔离实现，并发控制
- **WAL**: 预写日志，崩溃恢复
- **向量存储**: Semantic Embedding API
- **图存储**: DiskGraphStore 持久化
- **Cost-based Optimizer**: CBO 自动选择
- **Prepared Statement**: 参数化查询
- **连接池**: Connection Pool
- **子查询优化**: EXISTS/IN/ANY/ALL
- **Cypher 图查询**: 子集实现
- **统一查询 API**: SQL+Vector+Graph 混合查询

### 测试结果

| 测试套件 | 结果 |
|----------|------|
| Storage lib tests | 55/55 ✅ |
| Parser tests | 37/37 ✅ |
| Executor lib tests | 9/9 ✅ |
| sql-corpus | 20.3% (12/59) |
| 整体覆盖率 | 49% |

---

## [2.4.0] - 2026-04-08

> **发布**: GitHub Release v2.4.0 GA
> **阶段**: Alpha → 已归档

### 新增功能

- **SIMD 加速**: AVX-512 向量计算
- **列式存储**: ColumnarStorage + 压缩 (LZ4/Zstd)
- **B+ Tree 索引**: 磁盘持久化
- **Hash 索引**: 内存索引
- **内存映射**: mmap 文件存储
- **查询计划器**: 自动索引选择

### 性能

- TPC-H SF=1 基准测试
- 查询优化 40%+ 提升

---

## [2.1.0] - 2026-04-05

> **发布**: GitHub Release v2.1.0
> **阶段**: Alpha → 已归档

### 新增功能

- **网络层重构**: 异步网络架构
- **TCP 服务器**: MySQL 风格协议
- **连接池**: 多客户端支持
- **数据加载器**: 批量导入

---

## [2.0.0] - 2026-03-29

> **发布**: GitHub Release v2.0.0
> **阶段**: Alpha → 已归档

### 新增功能

- **异步网络层**: Tokio 异步运行时
- **客户端-服务器架构**: 独立服务器
- **多客户端连接**: 连接池

---

## [1.9.0] - 2026-03-27

> **发布**: GitHub Release v1.9.0
> **阶段**: Alpha → 已归档

### 新增功能

- **JOIN 优化**: Sort-Merge Join
- **TPC-H SF=0.1**: 小规模基准

---

## [1.7.0] - 2026-03-22

> **发布**: GitHub Release v1.7.0
> **阶段**: Alpha → 已归档

### 新增功能

- **存储引擎增强**: Buffer Pool 实现
- **B+ Tree**: 磁盘索引

---

## [1.5.0] - 2026-03-18

> **发布**: GitHub Release v1.5.0 GA
> **阶段**: Alpha → 已归档

### 新增功能

- **CBO**: 基于成本的优化器
- **SortMergeJoin**: 排序合并连接

---

## [1.4.0] - 2026-03-18

> **发布**: GitHub Release v1.4.0
> **阶段**: Alpha → 已归档

### 新增功能

- **CBO 基础**: 成本模型
- **SortMergeJoin**: 排序合并连接

---

## [1.3.0] - 2026-03-15

> **发布**: GitHub Release v1.3.0
> **阶段**: Alpha → 已归档

### 新增功能

- **模块化**: Crate 拆分
- **Storage Trait**: 存储抽象层

---

## [1.2.0] - 2026-03-12

> **发布**: GitHub Release v1.2.0 GA
> **阶段**: Alpha → 已归档

### 新增功能

- **架构重构**: 完整模块化
- **向量化执行**: RecordBatch

---

## [1.1.0] - 2026-03-04

> **发布**: GitHub Release v1.1.0 GA
> **阶段**: Alpha → 已归档

### 新增功能

- **查询规划器**: Logical/Physical Plan
- **基础优化器**: RBO
- **文件存储**: 持久化
- **事务支持**: 基础事务
- **索引支持**: B+ Tree

---

## [1.0.0] - 2026-02-21

> **发布**: GitHub Release v1.0.0
> **阶段**: GA
> **首次发布**

### 核心功能

- **Lexer**: 词法分析器
- **Parser**: SQL 语法分析
- **基础执行器**: 火山模型
- **内存存储**: 内存表
- **基础 SQL**: SELECT/INSERT/UPDATE/DELETE

---

## 版本发布历史

| 版本 | 发布日期 | 阶段 | 核心特性 |
|------|--------|------|----------|
| **v2.6.0** | TBD | Alpha | 生产就绪、SQL-92 完整 |
| v2.5.0 | 2026-04-16 | Alpha | MVCC、Vector/Graph、统一查询 |
| v2.4.0 | 2026-04-08 | Alpha | SIMD、列式存储、压缩 |
| v2.1.0 | 2026-04-05 | Alpha | 异步网络、连接池 |
| v2.0.0 | 2026-03-29 | Alpha | 异步架构 |
| v1.9.0 | 2026-03-27 | Alpha | JOIN 优化 |
| v1.7.0 | 2026-03-22 | Alpha | Buffer Pool |
| v1.5.0 | 2026-03-18 | Alpha | CBO |
| v1.4.0 | 2026-03-18 | Alpha | SortMergeJoin |
| v1.3.0 | 2026-03-15 | Alpha | 模块化 |
| v1.2.0 | 2026-03-12 | Alpha | 重构、向量化 |
| v1.1.0 | 2026-03-04 | GA | 查询规划、文件存储 |
| v1.0.0 | 2026-02-21 | GA | 初始版本 |

---

*最后更新: 2026-04-19*
*维护人: yinglichina8848*