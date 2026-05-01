# SQLRustGo v2.0 OO 架构文档

**版本**: v2.0.0 (Vector Engine + Cascades)
**发布日期**: 2026-03-26

---

## 一、文档目录

```
docs/releases/v2.0/oo/
├── architecture/           # 系统架构设计
│   ├── ARCHITECTURE_V2.md   # v2.0 架构总览
│   └── CASCADES_DESIGN.md  # Cascades 优化器设计
├── modules/             # 模块设计
│   ├── optimizer/         # 优化器模块
│   │   └── CBO_DESIGN.md
│   ├── executor/         # 执行器模块
│   │   └── VECTORIZED_EXEC.md
│   └── storage/          # 存储模块
│       └── BUFFER_POOL.md
├── algorithms/         # 核心算法设计
│   ├── COST_MODEL.md
│   └── JOIN_ALGORITHMS.md
├── api/               # API 参考
│   └── STORAGE_API.md
├── user-guide/         # 用户指南
│   └── USER_MANUAL.md  (引用外层)
└── reports/           # 报告
    └── PERFORMANCE_REPORT.md
```

---

## 二、架构特点 (vs v2.5.0)

| 特性 | v2.0 | v2.5 |
|------|------|------|
| 执行模型 | 向量化 | 向量化 + 并行 |
| 优化器 | Cascades | CBO + BloomFilter |
| 事务 | 无 | MVCC + WAL |
| 图引擎 | 无 | Cypher |
| 向量索引 | Flat | HNSW/IVFPQ |
| 统一查询 | 无 | SQL+向量+图 |

---

## 三、快速导航

| 模块 | 文档 |
|------|------|
| 架构设计 | [oo/architecture/ARCHITECTURE_V2.md](./architecture/ARCHITECTURE_V2.md) |
| Cascades | CASCADES_DESIGN.md *(已归档)* |
| 用户手册 | [USER_MANUAL.md](../USER_MANUAL.md) *(已归档)* |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-03-26*