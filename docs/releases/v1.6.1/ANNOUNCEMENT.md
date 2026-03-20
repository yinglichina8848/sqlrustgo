# SQLRustGo v1.6.1 开发公告

> **发布日期**: 2026-03-20
> **版本**: v1.6.1-dev
> **状态**: 🚀 开发中

---

## 一、版本概述

v1.6.1 是针对 v1.6.0 性能数据可信性问题的修复版本，核心目标是：

1. **建立可信 Benchmark 体系**
2. **提升测试覆盖率**
3. **完善性能分析工具链**

---

## 二、v1.6.0 总结

### 成就

- ✅ 架构达到工业级 (L4)
- ✅ 完整事务系统 (MVCC + 隔离级别)
- ✅ 50+ 并发支持
- ❌ 性能数据不可信 (内存 vs 磁盘对比不公平)

### 反馈

详见: [EXTERNAL_EVALUATION_FEEDBACK.md](../v1.6.0/EXTERNAL_EVALUATION_FEEDBACK.md)

---

## 三、v1.6.1 目标

### 性能可信

| 任务 | 说明 |
|------|------|
| 关闭 Query Cache | Benchmark 模式默认关闭 |
| PostgreSQL 对比 | 统一配置对比 |
| P50/P95/P99 延迟 | 专业指标 |
| 三层测试模式 | Embedded / TCP / PostgreSQL |

### 覆盖率提升

| 模块 | 当前 | 目标 |
|------|------|------|
| 总覆盖率 | 75.20% | 80%+ |
| Executor | 低 | +5% |
| 并发路径 | 低 | +3% |

### 工具链

| 工具 | 说明 |
|------|------|
| Flamegraph | CPU 热点分析 |
| Tracing | 关键路径埋点 |
| CI 回归 | 防止性能退化 |

---

## 四、开发任务

### Epic-01: Benchmark 系统重构

| Issue | 任务 | 认领 |
|-------|------|------|
| #B-01 | Benchmark Runner CLI | AI-CLI |
| #B-02 | OLTP Workload | AI-CLI |
| #B-03 | TPC-H 基准 | AI-CLI |

### Epic-02: 可信性修复 ✅ 已完成

| Issue | 任务 | 状态 | PR |
|-------|------|------|-----|
| #B-04 | 关闭 Query Cache | ✅ 完成 | #695 |
| #B-05 | PostgreSQL 对比 | ✅ 完成 | #695 |
| #B-06 | 统一 SQLite 配置 | ✅ 完成 | #695 |

### Epic-03: Metrics 系统

| Issue | 任务 | 认领 |
|-------|------|------|
| #B-07 | P50/P95/P99 延迟 | AI-CLI |
| #B-08 | JSON 输出 | AI-CLI |

### Epic-04: 环境标准化

| Issue | 任务 | 认领 |
|-------|------|------|
| #B-09 | 配置模板 | AI-CLI |
| #B-10 | 数据规模校验 | AI-CLI |

### Epic-05: CI 集成

| Issue | 任务 | 认领 |
|-------|------|------|
| #B-11 | Benchmark CI | AI-CLI |

---

## 五、交付标准

### 功能标准

- [ ] Benchmark CLI 支持多模式
- [ ] OLTP Workload 可配置
- [ ] P99 延迟统计
- [ ] PostgreSQL 对比
- [ ] JSON 输出

### 数据标准

- [ ] 关闭 Query Cache 测试
- [ ] 三种模式对比数据
- [ ] 瓶颈分解

### 覆盖率

- [ ] 总覆盖率 ≥ 80%

---

## 六、时间线

| 阶段 | 日期 | 任务 |
|------|------|------|
| Sprint 1 | Week 1-2 | Benchmark Runner + Metrics |
| Sprint 2 | Week 3-4 | PostgreSQL 对比 + CI |
| Sprint 3 | Week 5-6 | 覆盖率提升 + 测试补全 |
| RC | Week 7 | 冻结 + 测试 |
| GA | Week 8 | 发布 |

---

## 七、正确表述声明

### ✅ 可以说

- "架构达到工业级标准"
- "支持 MVCC + 隔离级别"
- "通过 TPC-H 正确性验证"

### ❌ 禁止说

- "比 SQLite 快 3000 倍"
- "TPS 达到 43 万"
- "无死锁"

---

## 八、相关文档

- [GOALS_AND_PLANNING.md](./GOALS_AND_PLANNING.md)
- [BENCHMARK_REPORT_TEMPLATE.md](./BENCHMARK_REPORT_TEMPLATE.md)
- [PERFORMANCE_ANALYSIS_TOOLCHAIN.md](./PERFORMANCE_ANALYSIS_TOOLCHAIN.md)

---

**公告日期**: 2026-03-20
**下次更新**: 2026-03-27 (Sprint 1 结束)
