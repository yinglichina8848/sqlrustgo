# SQLRustGo v1.6.1 开发公告

> **发布日期**: 2026-03-20
> **版本**: v1.6.1
> **状态**: ✅ Alpha 已发布 (2026-03-20)

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

### Epic-01: Benchmark 系统重构 ✅ 已完成

| Issue | 任务 | 状态 | PR |
|-------|------|------|-----|
| #B-01 | Benchmark Runner CLI | ✅ 完成 | #692 |
| #B-02 | OLTP Workload | ✅ 完成 | #692 |
| #B-03 | TPC-H 基准 | ✅ 完成 | #692 |

### Epic-02: 可信性修复 ✅ 已完成

| Issue | 任务 | 状态 | PR |
|-------|------|------|-----|
| #B-04 | 关闭 Query Cache | ✅ 完成 | #695 |
| #B-05 | PostgreSQL 对比 | ✅ 完成 | #695 |
| #B-06 | 统一 SQLite 配置 | ✅ 完成 | #695 |

### Epic-03: Metrics 系统 ✅ 已完成

| Issue | 任务 | 状态 | PR |
|-------|------|------|-----|
| #B-07 | P50/P95/P99 延迟 | ✅ 完成 | #691 |
| #B-08 | JSON 输出 | ✅ 完成 | #691 |

### Epic-04: 环境标准化 ✅ 已完成

| Issue | 任务 | 状态 | PR |
|-------|------|------|-----|
| #B-09 | 配置模板 | ✅ 完成 | #694 |
| #B-10 | 数据规模校验 | ✅ 完成 | #694 |

### Epic-05: CI 集成 ⏳ 待开发

| Issue | 任务 | 状态 |
|-------|------|------|
| #B-11 | Benchmark CI | ⏳ 待处理 |

---

## 五、交付标准

### 功能标准

- [x] Benchmark CLI 支持多模式
- [x] OLTP Workload 可配置
- [x] P50/P95/P99 延迟统计
- [x] PostgreSQL 对比
- [x] JSON 输出

### 数据标准

- [x] 关闭 Query Cache 测试
- [x] 三种模式对比数据 (SQLite/PostgreSQL/SQLRustGo)
- [x] 瓶颈分解

### 覆盖率

- [ ] 总覆盖率 ≥ 80% (当前 75.20%)

---

## 六、时间线

| 阶段 | 日期 | 状态 |
|------|------|------|
| Sprint 1 | Week 1-2 | ✅ 完成 |
| Sprint 2 | Week 3-4 | ✅ 完成 |
| Alpha | 2026-03-20 | ✅ 已发布 |
| Sprint 3 (覆盖率) | Week 5-6 | ⏳ 待处理 |
| EPIC-05 CI 集成 | TBD | ⏳ 待处理 |
| Beta | TBD | ⏳ |
| RC | TBD | ⏳ |
| GA | TBD | ⏳ |

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
**更新日期**: 2026-03-20 (Alpha 已发布)
**下次更新**: Sprint 3 开始前
