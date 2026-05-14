# v3.1.0 OO 执行链路演进路线图

> **版本**: v3.1.0
> **日期**: 2026-05-12
> **Issue**: #661

---

## 一、v3.0.0 OO 文档体系

| 目录 | 文档数 | 覆盖内容 |
|------|--------|---------|
| `oo/dml/` | 1 | DML 执行链路 |
| `oo/ddl/` | 3 | DDL/ALTER/INDEX |
| `oo/dcl/` | 1 | DCL 执行 |
| `oo/join/` | 1 | JOIN 算法 |
| `oo/query/` | 3 | 子查询/窗口函数/递归 CTE |
| `oo/setops/` | 1 | 集合运算 |
| `oo/transaction/` | 2 | 事务管理/MVCC |
| `oo/wal/` | 1 | WAL 协议 |
| `oo/recovery/` | 1 | 崩溃恢复 |
| `oo/distributed/` | 1 | 分布式同步 |
| `oo/advanced/` | 2 | 触发器/存储过程 |
| `oo/cbo/` | 3 | CBO 代价模型/设计/Join 排序 |
| `oo/bptree/` | 1 | B+Tree 设计 |
| **总计** | **21** | — |

---

## 二、v3.1.0 新增 OO 文档

### 2.1 已完成

| 文档 | 状态 | PR/Issue |
|------|------|----------|
| `oo/AUDIT_CHAIN_INTEGRATION.md` | ✅ 已完成 | #625 |
| `oo/MERGE_EXECUTION.md` | ✅ 已完成 | #613 |
| `oo/CBO_INTEGRATION.md` | ✅ 已完成 | #616 |

### 2.2 进行中

| 文档 | 状态 | Issue |
|------|------|-------|
| `oo/GAP_LOCKING.md` | ⏳ 进行中 | #607 |
| `oo/CLUSTERED_INDEX.md` | ⏳ 进行中 | #607 |
| `oo/STORAGE_ENCRYPTION.md` | ⏳ 进行中 | #607 |

### 2.3 待创建

| 文档 | 优先级 | Issue |
|------|--------|-------|
| `oo/OO_ROADMAP.md` | P0 | #661 |

---

## 三、v3.1.0 vs v3.0.0 OO 文档对比

### 3.1 新增链路

| v3.1.0 新增 | 描述 | v3.0.0 对应 |
|-------------|------|--------------|
| MERGE 执行链路 | UPSERT 语句完整执行路径 | 无 |
| Gap Locking | Next-Key Lock / Gap Lock 实现 | 无 |
| 聚簇索引 | 索引与表数据物理对齐 | 无 |
| 存储加密 | AES-256-GCM 透明加密 | 无 |
| 审计链集成 | WAL + 审计链联动 | WAL 独立 |

### 3.2 增强链路

| v3.1.0 增强 | 增强内容 | v3.0.0 基础 |
|-------------|---------|-------------|
| CBO 集成 | 代价模型接入 planner | CBO_COST_MODEL.md (未激活) |
| MVCC 可见性 | TLA+ 规格 + 反例测试 | MVCC_IMPLEMENTATION.md |

---

## 四、v3.2.0 规划

### 4.1 待实现功能

| 功能 | 文档 | 优先级 |
|------|------|--------|
| 事件调度器 | `oo/EVENT_SCHEDULER.md` | P2 |
| 全文索引 | `oo/FTS_EXECUTION.md` | P1 |
| 空间数据类型 | `oo/SPATIAL_EXECUTION.md` | P3 |
| 存储过程游标/异常 | `oo/CURSOR_EXECUTION.md` | P2 |
| INFORMATION_SCHEMA | `oo/INFO_SCHEMA_EXECUTION.md` | P2 |

### 4.2 性能优化

| 功能 | 文档 | 优先级 |
|------|------|--------|
| P99 延迟优化 | `oo/PERFORMANCE_OPTIMIZATION.md` | P1 |
| 并行查询执行 | `oo/PARALLEL_EXECUTION.md` | P2 |

---

## 五、OO 文档创建标准

### 5.1 必须包含

每个 OO 文档必须包含：

1. **执行链路图**：ASCII 流程图 (Parser → Planner → Optimizer → Executor → Storage)
2. **各层职责**：每层的关键数据结构和算法
3. **关键代码路径**：核心函数和 trait
4. **边界情况**：错误处理和异常场景
5. **测试覆盖**：现有测试和缺口

### 5.2 命名规范

```
oo/<category>/<FEATURE_EXECUTION.md
```

| category | 用途 |
|----------|------|
| `dml/` | DML 语句执行 |
| `ddl/` | DDL 语句执行 |
| `query/` | 查询类型执行 |
| `storage/` | 存储引擎 |
| `transaction/` | 事务控制 |
| `network/` | 网络协议 |

---

## 六、进度追踪

### 6.1 v3.1.0 目标

- [x] `oo/AUDIT_CHAIN_INTEGRATION.md` - 审计链集成
- [x] `oo/MERGE_EXECUTION.md` - MERGE 执行链路
- [x] `oo/CBO_INTEGRATION.md` - CBO 集成
- [x] `oo/GAP_LOCKING.md` - Gap Lock 实现 (PR #775)
- [x] `oo/CLUSTERED_INDEX.md` - 聚簇索引 (63 测试)
- [x] `oo/STORAGE_ENCRYPTION.md` - 存储加密 (security crate)

### 6.2 v3.2.0 目标

- [ ] `oo/FTS_EXECUTION.md` - 全文索引
- [ ] `oo/EVENT_SCHEDULER.md` - 事件调度器
- [ ] `oo/PERFORMANCE_OPTIMIZATION.md` - 性能优化

---

## 七、参考

- `docs/releases/v3.0.0/oo/SQL_EXECUTION_MATRIX.md` - 执行矩阵
- `docs/releases/v3.1.0/COVERAGE_GAP_REMEDIATION_PLAN.md` - 覆盖缺口整改计划
- `docs/releases/v3.1.0/OO_DOCUMENT_ANALYSIS.md` - OO 文档分析