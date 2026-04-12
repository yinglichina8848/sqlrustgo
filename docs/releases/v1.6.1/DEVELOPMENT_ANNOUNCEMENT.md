# v1.6.1 开发进度公告

> **发布日期**: 2026-03-20  
> **版本**: v1.6.1 (Benchmark 修复版)  
> **状态**: 开发中 (7/11 任务已完成)

---

## 一、已完成任务 ✅

### EPIC-01: Benchmark 系统重构 (100% 完成)

| Issue | 任务 | PR |
|-------|------|-----|
| B-01 | Benchmark Runner CLI | #692 |
| B-02 | OLTP Workload | #692 |
| B-03 | TPC-H 基准 (Q1/Q3/Q6/Q10) | #692 |

### EPIC-03: Metrics 系统 (100% 完成)

| Issue | 任务 | PR |
|-------|------|-----|
| B-07 | P50/P95/P99 延迟统计 | #691 |
| B-08 | JSON 结果格式 | #691 |

### EPIC-04: 环境标准化 (100% 完成)

| Issue | 任务 | PR |
|-------|------|-----|
| B-09 | Benchmark 配置模板 | #694 |
| B-10 | 数据规模校验 | #694 |

---

## 二、待处理任务 ⏳

### EPIC-02: Benchmark 可信性修复 (P0)

| Issue | 任务 | 优先级 |
|-------|------|--------|
| B-04 | 禁用 Query Cache (Benchmark 模式) | P0 |
| B-05 | 引入 PostgreSQL 对比 | P0 |
| B-06 | 统一 SQLite 配置 | P0 |

### EPIC-05: CI 集成 (P2)

| Issue | 任务 | 优先级 |
|-------|------|--------|
| B-11 | Benchmark CI (轻量版) | P2 |

---

## 三、已合并 PR 汇总

| PR | 标题 | 贡献者 |
|----|------|--------|
| #688 | test: 提升测试覆盖率到80% | @yinglichina8848 |
| #689 | docs: v1.6.1 规划文档 | @sonaheartopen |
| #691 | feat(metrics): P50/P95/P99 + JSON | @sonaheartopen |
| #692 | feat: EPIC-01 Benchmark CLI | @sonaheartopen |
| #693 | test: 提升测试覆盖率到80% - v1.6.1 | @yinglichina8848 |
| #694 | feat(bench): 配置和校验 | @sonaheartopen |

---

## 四、交付标准进度

| 标准 | 状态 |
|------|------|
| Benchmark 可复现 | ✅ |
| 数据规模合理 | ✅ |
| JSON 输出 | ✅ |
| P99 延迟 | ✅ |
| PostgreSQL 对比 | ⏳ |
| Cache 默认关闭 | ⏳ |

---

## 五、测试状态

```
✅ 编译通过
✅ Clippy 无警告
✅ 格式化通过
✅ 所有测试通过 (259+ tests)
```

---

## 六、下一步计划

1. **B-04**: 实现 Benchmark 模式（默认关闭 Query Cache）
2. **B-05**: 引入 PostgreSQL 对比基线
3. **B-06**: 统一 SQLite 配置
4. **B-11**: Benchmark CI 集成

---

## 七、参与贡献

欢迎提交 PR！请参考 [开发指南](./docs/plans/)。

- GitHub: https://github.com/minzuuniversity/sqlrustgo
- 分支: `develop/v1.6.1`

---

*感谢所有贡献者的努力！*
