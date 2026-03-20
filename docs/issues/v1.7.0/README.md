# v1.7.0 Issues Index

## 版本信息

- **版本**: v1.7.0
- **代号**: SQL + 可观测性补完版
- **发布日期**: 2026-04-25
- **创建日期**: 2026-03-21

---

## Epic 总览

| Epic | 名称 | 资源占比 | Issues | 总工作量 |
|------|------|----------|--------|----------|
| Epic-01 | SQL 补完 | 40% | 5 | ~580 行 |
| Epic-02 | 可观测性 | 30% | 4 | ~650 行 |
| Epic-03 | Benchmark 完善 | 20% | 3 | ~250 行 |
| Epic-04 | 错误系统 | 10% | 3 | ~100 行 |
| **总计** | - | **100%** | **15** | **~1580 行** |

---

## Issue 列表

### Epic-01: SQL 补完

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| SQL-01 | UNION / UNION ALL 支持 | P0 | 150 行 |
| SQL-02 | INTERSECT / EXCEPT | P1 | 100 行 |
| SQL-03 | VIEW 创建和查询 | P0 | 200 行 |
| SQL-04 | BOOLEAN 类型补齐 | P0 | 50 行 |
| SQL-05 | BLOB 类型支持 | P1 | 80 行 |

### Epic-02: 可观测性

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| OBS-01 | EXPLAIN 执行计划 | P0 | 200 行 |
| OBS-02 | EXPLAIN ANALYZE | P0 | 250 行 |
| OBS-03 | 算子级 Profiling | P1 | 150 行 |
| OBS-04 | 格式化输出 | P0 | 50 行 |

### Epic-03: Benchmark 完善

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| BEN-01 | Benchmark CLI 完善 | P1 | 100 行 |
| BEN-02 | JSON 输出格式 | P0 | 50 行 |
| BEN-03 | PostgreSQL 对比测试 | P1 | 100 行 |

### Epic-04: 错误系统

| Issue | 标题 | 优先级 | 工作量 |
|-------|------|--------|--------|
| ERR-01 | Unknown column 错误 | P1 | 30 行 |
| ERR-02 | Table not found 错误 | P1 | 30 行 |
| ERR-03 | Duplicate key 错误 | P2 | 40 行 |

---

## 里程碑

```
Week 1 (03/21-03/27)     Week 2 (03/28-04/03)     Week 3 (04/04-04/10)
├── SQL-01 UNION      ├── SQL-03 VIEW           ├── OBS-01 EXPLAIN
├── SQL-04 BOOLEAN   ├── SQL-05 BLOB           ├── OBS-02 ANALYZE
└── SQL-02 INTERSECT └── OBS-01 token         └── BEN-02 JSON

Week 4 (04/11-04/17)     Week 5 (04/18-04/24)     GA (04/25)
├── OBS-02 ANALYZE    ├── ERR-01~03            ├── 文档完善
├── OBS-03 Profiling  └── 集成测试             └── 最终验证
```

---

## 相关文档

| 文档 | 说明 |
|------|------|
| [EPIC-01-SQL补完.md](./EPIC-01-SQL补完.md) | SQL 补完详细设计 |
| [EPIC-02-可观测性.md](./EPIC-02-可观测性.md) | 可观测性详细设计 |
| [EPIC-03-Benchmark完善.md](./EPIC-03-Benchmark完善.md) | Benchmark 详细设计 |
| [EPIC-04-错误系统.md](./EPIC-04-错误系统.md) | 错误系统详细设计 |
| [../2026-03-21-v170-issue-spec.md](../2026-03-21-v170-issue-spec.md) | 完整 Issue 规格 |
| [../2026-03-21-v170-release-plan.md](../2026-03-21-v170-release-plan.md) | 发布计划 |
| [../../VERSION_ROADMAP.md](../../VERSION_ROADMAP.md) | 版本路线图 |

---

## GitHub Labels

建议为 Issues 添加以下 Labels：

- `epic/epic-01` - SQL 补完
- `epic/epic-02` - 可观测性
- `epic/epic-03` - Benchmark 完善
- `epic/epic-04` - 错误系统
- `priority/p0` - P0 优先级
- `priority/p1` - P1 优先级
- `priority/p2` - P2 优先级
- `type/feature` - 新功能
- `type/bug` - Bug 修复
- `type/docs` - 文档