# SQLRustGo v2.0.0 开发 Issue 总表

## AI 开发线依赖图

```
Phase →      P1                           P2                          P3                           P4                           P5
AI ↓
────────────────────────────────────────────────────────────────────────────────────────────────────────────
🟦 OpenCode A
#942 WAL回放  ──► #953 主从复制 ──► #944 2PC/分布式
#987 PageChecksum
#963 内存管理
#964 批量写入
#974 Batch Insert优化 ────────────┐
#972 WAL Group Commit ───────────┤
                                   │
🟩 OpenCode B
#988 Catalog系统 ─────────────────┤
#989 EXPLAIN                       │
#955 窗口函数实现                  │
#956 RBAC权限                      │
#945 安全/审计                    │
#946 基础性能 ◄────────────────────┤
                                   │
🟧 Claude A
#954 ParallelExecutor (#976) ────► #944 Sharding/2PC/分布式查询优化
#946 向量化/CBO ◄──────────────────┘
                                   │
🟥 Claude B
#753 ColumnChunk ─► #754 ColumnSegment ─► #755 ColumnarStorage ─► #757 ColumnarScan ─► #756 ProjectionPushdown ─► #758 Parquet
#952 性能基准/Epic-12列存储 ──────► #946 基础性能
```

### 依赖说明
- **Phase2 高可用 (#953)** ← Phase1 存储稳定性 (#942,#987,#963,#964)
- **Phase3 分布式 (#944)** ← Phase2 HA (#953,#966)
- **Epic-12 列存储 (#753-#758)** ← Phase1 ParallelExecutor (#954,#976)
- **Phase5 基础性能 (#946)** ← 聚合各 AI 输出
- **Phase5 性能优化** 可跨 AI 并行

### AI 颜色对应
- 🟦 **OpenCode A**: 存储/写入/HA
- 🟩 **OpenCode B**: Catalog/EXPLAIN/安全
- 🟧 **Claude A**: 并行查询/分布式/向量化
- 🟥 **Claude B**: Epic-12 列存储/Parquet/性能基准

---

## Issue 状态汇总

### Phase 1: 存储稳定性 (WAL/Catalog/MVCC/EXPLAIN)

| Issue | 标题 | 状态 |
|-------|------|------|
| #941 | v2.0.0 开发总控 | OPEN |
| #942 | Phase 1: 存储稳定性 | CLOSED ✅ |
| #952 | 性能基准工具搭建 - sysbench | CLOSED ✅ |
| #953 | 主从复制 - Binlog/故障转移 | CLOSED ✅ |
| #954 | 并行查询框架 - ParallelExecutor | OPEN |
| #963 | 内存管理优化 - Arena/Pool | CLOSED ✅ |
| #964 | 批量写入优化 - INSERT batch | CLOSED ✅ |
| #965 | WAL 组提交优化 | CLOSED ✅ |
| #966 | 主从复制原型 - Binlog/故障转移 | CLOSED ✅ |
| #975 | 任务调度器 - TaskScheduler | CLOSED ✅ |
| #976 | 并行执行器 - ParallelExecutor | CLOSED ✅ |
| #987 | Page Checksum 完整集成 | CLOSED ✅ |
| #988 | Catalog 系统完整集成 | CLOSED ✅ |
| #989 | EXPLAIN 算子覆盖扩展 | CLOSED ✅ |

### Phase 2: 高可用

| Issue | 标题 | 状态 |
|-------|------|------|
| #943 | Phase 2: 高可用 - 主从复制/备份/故障转移 | CLOSED ✅ |
| #955 | 窗口函数实现 - ROW_NUMBER/RANK/SUM OVER | CLOSED ✅ |
| #956 | RBAC 权限系统 - 用户/角色/GRANT | CLOSED ✅ |

### Phase 3: 分布式能力

| Issue | 标题 | 状态 |
|-------|------|------|
| #944 | Phase 3: 分布式能力 - Sharding/分布式事务 | CLOSED ✅ |

### Phase 4: 安全与治理

| Issue | 标题 | 状态 |
|-------|------|------|
| #945 | Phase 4: 安全与治理 - RBAC/SSL/审计 | CLOSED ✅ |
| #885 | 高可用与数据可靠性 - 主从复制/备份 | OPEN |
| #886 | 安全与权限管理 - RBAC/SSL | OPEN |

### Phase 5: 性能优化

| Issue | 标题 | 状态 |
|-------|------|------|
| #946 | Phase 5: 性能优化 - 向量化/CBO/列式存储 | CLOSED ✅ |

### Epic-12: 列式存储

| Issue | 标题 | 状态 |
|-------|------|------|
| #753 | ColumnChunk 数据结构 | CLOSED ✅ |
| #754 | ColumnSegment 磁盘布局 | CLOSED ✅ |
| #755 | ColumnarStorage 存储引擎 | CLOSED ✅ |
| #756 | Projection Pushdown 优化器 | CLOSED ✅ |
| #757 | ColumnarScan 执行器节点 | CLOSED ✅ |
| #758 | Parquet 导入导出 | CLOSED ✅ |

### Epic-16: v1.9.x → v2.0 迁移

| Issue | 标题 | 状态 |
|-------|------|------|
| #840 | 全面评估报告 & 后续任务 | CLOSED ✅ |
| #848 | 数据库内核工程化强化计划 | CLOSED ✅ |
| #887 | DeepSeek 审核整改计划 | CLOSED ✅ |
| #974 | Batch insert optimization | CLOSED ✅ |
| #972 | WAL Group Commit | CLOSED ✅ |

---

## AI 开发线 Issue

| Issue | 开发线 | AI工具 | 状态 |
|-------|--------|--------|------|
| #994 | AI Line1: 存储与高可用 | OpenCode A | OPEN |
| #995 | AI Line2: Catalog与安全 | OpenCode B | OPEN |
| #996 | AI Line3: 分布式事务与查询优化 | Claude A | OPEN |
| #997 | AI Line4: 并行执行与性能调优 | Claude B | OPEN |
| #998 | AI Line5: 性能优化与基准测试 | Claude/DeepSeek | OPEN |

---

## 统计

| Phase | 总计 | CLOSED | OPEN |
|-------|------|--------|------|
| Phase 1 | 14 | 13 | 1 |
| Phase 2 | 3 | 3 | 0 |
| Phase 3 | 1 | 1 | 0 |
| Phase 4 | 3 | 1 | 2 |
| Phase 5 | 1 | 1 | 0 |
| Epic-12 | 6 | 6 | 0 |
| **总计** | **28** | **25 (89%)** | **3** |

---

*最后更新: 2026-03-29*