# SQLRustGo v1.4.0 变更日志

> **版本**: 1.4.0
> **发布日期**: 2026-03-18

---

## v1.4.0 (2026-03-18)

### ✨ 新功能

#### CBO 成本优化器
- 添加 CBO-01: 成本模型基础框架
- 添加 CBO-02: 统计信息集成
- 添加 CBO-03: Join 顺序优化
- 添加 CBO-04: 索引选择优化

#### Join 算子
- 添加 SortMergeJoinExecutor (SMJ-01)
- 添加 SortMergeJoin 单元测试 (SMJ-02)
- 添加 NestedLoopJoinExecutor (NLJ-01)

#### 可观测性
- 添加 Prometheus 指标格式 (M-003)
- 添加 /metrics HTTP 端点 (M-004)
- 添加 Grafana Dashboard 模板 (M-005)

#### 基准测试
- 添加 TPC-H 基准测试 (PB-01)
- 添加性能对比报告 (PB-02)

### 🐛 修复

- 修复 SortMergeJoin 编译错误 (first_or_null → sort_key)
- 修复 left join 逻辑错误

### 📚 文档

- 添加 v1.4.0 开发计划
- 添加 v1.4.0 版本计划
- 添加性能基准测试报告

### ⚡ 性能提升

- HashJoin 性能提升 ~15%
- 平均查询延迟降低 ~5.2%
- 吞吐量提升 ~5.6%

---

## v1.3.0 (2026-03-15)

### ✨ 新功能

- Executor 模块覆盖率提升到 87%+
- Planner 模块覆盖率提升到 76%+
- Optimizer 模块覆盖率提升到 82%+
- 健康检查端点
- 完整的测试框架

### 🐛 修复

- 多处 bug 修复和稳定性改进

---

## v1.2.0 (2026-02-XX)

### ✨ 新功能

- 基础 SQL 解析
- 物理计划生成
- 基础执行器

---

**完整变更日志请查看 [CHANGE_LOG.md](../CHANGE_LOG.md)**
