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

| Issue | 标题 | 状态 | PR |
|-------|------|------|-----|
| #B-01 | 实现 Benchmark Runner CLI | ✅ 已完成 | #692 |
| #B-02 | 实现 OLTP Workload (YCSB-like) | ✅ 已完成 | #692 |
| #B-03 | 实现 TPC-H 基准 (Q1/Q3/Q6/Q10) | ✅ 已完成 | #692 |

### EPIC-02: Benchmark 可信性修复 (P0)

| Issue | 标题 | 状态 | PR |
|-------|------|------|-----|
| #B-04 | 禁用 Query Cache (Benchmark 模式) | ✅ 已完成 | #695 |
| #B-05 | 引入 PostgreSQL 对比 | ✅ 已完成 | #695 |
| #B-06 | 统一 SQLite 配置 | ✅ 已完成 | #695 |

### EPIC-03: Metrics 系统 (P1)

| Issue | 标题 | 状态 | PR |
|-------|------|------|-----|
| #B-07 | 实现延迟统计 (P50/P95/P99) | ✅ 已完成 | #691 |
| #B-08 | 统一结果格式 (JSON) | ✅ 已完成 | #691 |

### EPIC-04: 环境标准化 (P1)

| Issue | 标题 | 状态 | PR |
|-------|------|------|-----|
| #B-09 | Benchmark 配置模板 | ✅ 已完成 | #694 |
| #B-10 | 数据规模校验 | ✅ 已完成 | #694 |

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

### Alpha 阶段完成度

| 任务 | 状态 | 说明 |
|------|------|------|
| Benchmark CLI | ✅ | 支持多模式 |
| OLTP Workload | ✅ | YCSB-like |
| TPC-H 基准 | ✅ | Q1/Q3/Q6/Q10 |
| Query Cache 关闭 | ✅ | Benchmark 模式 |
| PostgreSQL 对比 | ✅ | 统一配置 |
| P50/P95/P99 延迟 | ✅ | 已实现 |
| JSON 输出 | ✅ | 统一格式 |
| 配置模板 | ✅ | YAML 配置 |
| 数据规模校验 | ✅ | 数据验证 |

### 待完成 (Beta/RC)

| 任务 | 状态 | 优先级 |
|------|------|--------|
| Benchmark CI | ⏳ 待处理 | P2 |

---

## 六、版本阶段

| 阶段 | 日期 | 状态 |
|------|------|------|
| Craft | 2026-03-20 | ✅ |
| **Alpha** | **2026-03-20** | **🔥 进行中** |
| Beta | TBD | ⏳ |
| RC | TBD | ⏳ |
| GA | TBD | ⏳ |

---

*创建日期: 2026-03-20*
*更新日期: 2026-03-21*