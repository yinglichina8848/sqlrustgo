# v3.1.0 OO 执行链路文档

> **版本**: v3.1.0
> **日期**: 2026-05-11
> **说明**: v3.1.0 新增执行链路文档，补充 v3.0.0 OO 文档未覆盖的部分

---

## 文档索引

### v3.1.0 新增文档

- [OO_ROADMAP.md](OO_ROADMAP.md) — OO 文档演进路线图
- [MERGE_EXECUTION.md](MERGE_EXECUTION.md) — MERGE 语句完整执行链路
- [GAP_LOCKING.md](GAP_LOCKING.md) — Next-Key Lock / Gap Lock 实现
- [CLUSTERED_INDEX.md](CLUSTERED_INDEX.md) — 聚簇索引设计与 B+Tree 联动
- [STORAGE_ENCRYPTION.md](STORAGE_ENCRYPTION.md) — AES-256-GCM 存储加密链路
- [COVERAGE_GAP_REMEDIATION_PLAN.md](COVERAGE_GAP_REMEDIATION_PLAN.md) — 覆盖缺口整改计划
- [CBO_INTEGRATION.md](CBO_INTEGRATION.md) — CBO 代价模型集成

### v3.1.0 新增安全文档

- [security/RBAC_EXECUTION.md](security/RBAC_EXECUTION.md) — RBAC 执行层文档

---

## 文档状态

```
v3.1.0 OO/
├── README.md                        ← 本索引
├── OO_ROADMAP.md                   ← v3.1.0/v3.2.0 OO 演进路线 ✅
├── MERGE_EXECUTION.md              ← MERGE 执行链路 ✅
├── GAP_LOCKING.md                  ← Gap Lock 实现 ✅
├── CLUSTERED_INDEX.md              ← 聚簇索引 ✅
├── STORAGE_ENCRYPTION.md           ← 存储加密 ✅
├── COVERAGE_GAP_REMEDIATION_PLAN.md ← 覆盖缺口整改 ✅
├── CBO_INTEGRATION.md             ← CBO 集成 ✅
└── security/
    └── RBAC_EXECUTION.md          ← RBAC 执行 ✅
```

---

## v3.1.0 OO 文档与 v3.0.0 的关系

| v3.0.0 OO 文档 | v3.1.0 补充/增强 |
|----------------|-------------------|
| `oo/dml/INSERT_EXECUTION.md` | `oo/MERGE_EXECUTION.md` 补充 |
| `oo/transaction/MVCC_IMPLEMENTATION.md` | `oo/GAP_LOCKING.md` 补充 Next-Key Lock |
| `oo/bptree/BPTREE_DESIGN.md` | `oo/CLUSTERED_INDEX.md` 补充聚簇索引 |
| `oo/security/` (不存在) | `oo/STORAGE_ENCRYPTION.md` 新增 |

---

## 进度

- [x] `oo/OO_ROADMAP.md` — v3.1.0/v3.2.0 OO 演进路线
- [x] `oo/MERGE_EXECUTION.md` — MERGE 语句执行链路
- [x] `oo/GAP_LOCKING.md` — Next-Key Lock / Gap Lock 实现
- [x] `oo/CLUSTERED_INDEX.md` — 聚簇索引设计
- [x] `oo/STORAGE_ENCRYPTION.md` — AES-256-GCM 加密
- [x] `oo/COVERAGE_GAP_REMEDIATION_PLAN.md` — 覆盖缺口整改
- [x] `oo/CBO_INTEGRATION.md` — CBO 集成
- [x] `oo/security/RBAC_EXECUTION.md` — RBAC 执行层文档
