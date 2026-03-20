# SQLRustGo v1.6.1 发布计划

> **版本**: v1.6.1
> **发布日期**: TBD
> **代号**: Benchmark 修复版
> **目标**: 恢复性能数据可信性 + 建立标准 Benchmark 体系

---

## 一、版本定位

v1.6.1 is a **stability and benchmarking correctness release**.

This version does NOT introduce new features. Instead, it focuses on:

- Rebuilding a reproducible benchmark system
- Fixing misleading performance measurements
- Establishing a fair comparison baseline (PostgreSQL / SQLite)

---

## 二、Epic 拆分

### EPIC-01: Benchmark 系统重构 (P0)

| Issue | 标题 | 状态 |
|-------|------|------|
| #B-01 | 实现 Benchmark Runner CLI | ✅ 已完成 |
| #B-02 | 实现 OLTP Workload (YCSB-like) | ✅ 已完成 |
| #B-03 | 实现 TPC-H 基准 (Q1/Q3/Q6/Q10) | ✅ 已完成 |

### EPIC-02: Benchmark 可信性修复 (P0)

| Issue | 标题 | 状态 |
|-------|------|------|
| #B-04 | 禁用 Query Cache (Benchmark 模式) | ⏳ 待处理 |
| #B-05 | 引入 PostgreSQL 对比 | ⏳ 待处理 |
| #B-06 | 统一 SQLite 配置 | ⏳ 待处理 |

### EPIC-03: Metrics 系统 (P1)

| Issue | 标题 | 状态 |
|-------|------|------|
| #B-07 | 实现延迟统计 (P50/P95/P99) | ✅ 已完成 |
| #B-08 | 统一结果格式 (JSON) | ✅ 已完成 |

### EPIC-04: 环境标准化 (P1)

| Issue | 标题 | 状态 |
|-------|------|------|
| #B-09 | Benchmark 配置模板 | ✅ 已完成 |
| #B-10 | 数据规模校验 | ✅ 已完成 |

### EPIC-05: CI 集成 (P2)

| Issue | 标题 | 状态 |
|-------|------|------|
| #B-11 | Benchmark CI (轻量版) | ⏳ 待处理 |

---

## 三、PR 结构

| PR | 内容 |
|----|------|
| PR-1 | Benchmark Runner |
| PR-2 | OLTP Workload |
| PR-3 | Metrics |
| PR-4 | PostgreSQL Adapter |
| PR-5 | Cache Disable |

---

## 四、Release Notes 要点

```
⚠️ Performance numbers in v1.6.0 were measured under non-standard conditions.

v1.6.1 introduces a reproducible benchmark system.
```

---

## 五、交付标准

- [x] Benchmark 可复现
- [ ] 有 PostgreSQL 对比
- [x] 有 P99 延迟
- [ ] Cache 默认关闭
- [x] 数据规模合理
- [x] JSON 输出

---

*创建日期: 2026-03-20*
