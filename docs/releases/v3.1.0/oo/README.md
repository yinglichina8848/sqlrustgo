# v3.1.0 OO 执行链路文档

> **版本**: v3.1.0  
> **日期**: 2026-05-11  
> **说明**: v3.1.0 新增执行链路文档，补充 v3.0.0 OO 文档未覆盖的部分

---

## 新增文档

| 文档 | 大小 | 描述 | 对应 Issue |
|------|------|------|-----------|
| `oo/README.md` | 本文件 | 索引 | — |
| `OO_ROADMAP.md` | 8KB | OO 文档演进路线图 | #628 |
| `oo/MERGE_EXECUTION.md` | 12KB | MERGE 语句完整执行链路 | #613 |
| `oo/GAP_LOCKING.md` | 15KB | Next-Key Lock / Gap Lock 实现 | #607 |
| `oo/CLUSTERED_INDEX.md` | 18KB | 聚簇索引设计与 B+Tree 联动 | #607 |
| `oo/STORAGE_ENCRYPTION.md` | 12KB | AES-256-GCM 存储加密链路 | #607 |
| `oo/AUDIT_CHAIN.md` | 10KB | 审计链 SHA-256 + WAL 集成 | #607 |

---

## 文档状态

```
v3.1.0 OO/
├── README.md                        ← 本索引
├── OO_ROADMAP.md                    ← v3.1.0/v3.2.0 OO 演进路线
├── MERGE_EXECUTION.md              ← MERGE 执行链路 [规划中]
├── GAP_LOCKING.md                  ← Gap Lock 实现 [规划中]
├── CLUSTERED_INDEX.md              ← 聚簇索引 [规划中]
├── STORAGE_ENCRYPTION.md           ← 存储加密 [规划中]
└── AUDIT_CHAIN.md                  ← 审计链集成 [规划中]
```

---

## v3.1.0 OO 文档与 v3.0.0 的关系

| v3.0.0 OO 文档 | v3.1.0 补充/增强 |
|----------------|-------------------|
| `oo/dml/INSERT_EXECUTION.md` | `oo/MERGE_EXECUTION.md` 补充 |
| `oo/transaction/MVCC_IMPLEMENTATION.md` | `oo/GAP_LOCKING.md` 补充 Next-Key Lock |
| `oo/bptree/BPTREE_DESIGN.md` | `oo/CLUSTERED_INDEX.md` 补充聚簇索引 |
| `oo/wal/WAL_PROTOCOL.md` | `oo/AUDIT_CHAIN.md` 补充审计链集成 |
| `oo/security/` (不存在) | `oo/STORAGE_ENCRYPTION.md` 新增 |

---

## 进度

- [ ] `oo/OO_ROADMAP.md` — v3.1.0/v3.2.0 OO 演进路线
- [ ] `oo/MERGE_EXECUTION.md` — MERGE 语句执行链路
- [ ] `oo/GAP_LOCKING.md` — Next-Key Lock / Gap Lock 实现
- [ ] `oo/CLUSTERED_INDEX.md` — 聚簇索引设计
- [ ] `oo/STORAGE_ENCRYPTION.md` — AES-256-GCM 加密
- [ ] `oo/AUDIT_CHAIN.md` — 审计链 SHA-256 + WAL
