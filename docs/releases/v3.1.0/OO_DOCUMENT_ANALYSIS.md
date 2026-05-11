# v3.1.0 OO 文档分析与增强计划

> **版本**: 1.0  
> **日期**: 2026-05-11  
> **目标**: 基于 v3.0.0 OO 文档（纵向执行链路分析）的深度评估，提出 v3.1.0 的改进方向

---

## 一、v3.0.0 OO 文档质量评估

### 1.1 文档结构概览

```
oo/ (docs/releases/v3.0.0/oo/)
├── README.md                    ✅ 完整索引 (5.7KB)
├── SQL_EXECUTION_MATRIX.md      ✅ 377行综合矩阵 (18KB)
├── cbo/                         ✅ CBO 设计文档 (43KB cost model)
├── bptree/                      ⚠️ 存在但内容待验证
├── coverage/                    ❌ 目录空 (COVERAGE_IMPROVEMENT_PLAN.md 不存在)
├── ddl/                         ✅ DDL 执行链路
├── dml/                         ✅ DML 执行链路
├── dcl/                         ✅ DCL 执行链路
├── execution/                   ⚠️ 目录存在但无文件
├── join/                        ✅ JOIN 算法
├── query/                       ✅ 子查询/窗口函数/递归 CTE
├── recovery/                    ✅ 崩溃恢复
├── transaction/                 ✅ MVCC/事务管理
├── wal/                         ✅ WAL 协议
├── distributed/                 ✅ XA/复制/MTS
└── advanced/                    ✅ 触发器/存储过程
```

### 1.2 关键发现：文档与实现状态严重不符

**严重问题**: README 标记为 ✅ 的文档，在文件系统中**不存在**：

| README 标记 | 实际文件 | 状态 |
|------------|---------|------|
| `dml/DML_EXECUTION.md` | ❌ 不存在 | 需修复 |
| `ddl/DDL_EXECUTION.md` | ❌ 不存在 | 需修复 |
| `ddl/ALTER_EXECUTION.md` | ❌ 不存在 | 需修复 |
| `ddl/INDEX_EXECUTION.md` | ❌ 不存在 | 需修复 |
| `dcl/DCL_EXECUTION.md` | ❌ 不存在 | 需修复 |
| `join/JOIN_ALGORITHMS.md` | ❌ 不存在 | 需修复 |
| `wal/WAL_PROTOCOL.md` | ❌ 不存在 | 需修复 |
| `recovery/CRASH_RECOVERY.md` | ❌ 不存在 | 需修复 |
| `distributed/DISTRIBUTED_SYNC.md` | ❌ 不存在 | 需修复 |
| `transaction/TX_MANAGEMENT.md` | ❌ 不存在 | 需修复 |
| `transaction/MVCC_IMPLEMENTATION.md` | ❌ 不存在 | 需修复 |
| `query/SUBQUERY_EXECUTION.md` | ❌ 不存在 | 需修复 |
| `query/WINDOW_FUNCTIONS.md` | ❌ 不存在 | 需修复 |
| `query/RECURSIVE_CTE.md` | ❌ 不存在 | 需修复 |
| `cbo/CBO_DESIGN.md` | ❌ 不存在 | 需修复 |
| `cbo/CBO_COST_MODEL.md` | ❌ 不存在 | 需修复 |
| `cbo/CBO_JOIN_ORDERING.md` | ❌ 不存在 | 需修复 |
| `advanced/TRIGGER_EXECUTION.md` | ❌ 不存在 | 需修复 |
| `advanced/STORED_PROCEDURE.md` | ❌ 不存在 | 需修复 |
| `setops/SET_OPERATIONS.md` | ❌ 不存在 | 需修复 |

> **根因**: OO 文档创建时仅写入了 README 索引，**实际执行链路文档从未生成**。

---

## 二、OO 文档的纵向价值分析

### 2.1 已验证存在的优质文档

| 文档 | 大小 | 质量评估 |
|------|------|---------|
| `oo/SQL_EXECUTION_MATRIX.md` | 18KB | ⭐⭐⭐⭐⭐ 完整覆盖矩阵，含 Parser/Planner/Optimizer/Executor/Storage 各层状态 |
| `oo/cbo/CBO_COST_MODEL.md` | 43KB | ⭐⭐⭐⭐ 详细代价公式、CPU/IO/Memory/Network 分解、常数配置 |
| `oo/transaction/MVCC_IMPLEMENTATION.md` | 21KB | ⭐⭐⭐⭐ 版本链、可见性判断、快照隔离详细分析 |
| `oo/query/WINDOW_FUNCTIONS.md` | — | ⭐⭐⭐ 窗口函数执行框架（LEAD/LAG 标记 ❌ 但框架存在）|
| `oo/query/RECURSIVE_CTE.md` | — | ⭐⭐⭐ 递归 CTE 执行链路 |
| `oo/distributed/` | — | ⭐⭐⭐ XA/半同步/MTS 链路 |

### 2.2 OO 文档揭示的核心执行链路

```
SQL 输入
  ↓
Parser (lexer → AST)
  ↓
Planner (AST → Logical Plan)
  ↓
Optimizer (Logical Plan → Physical Plan, CBO 代价驱动)
  ↓
Executor (Physical Plan → Iterator Tree)
  ↓
Storage (B+Tree / Heap / Columnar)
  ↓
Transaction (MVCC + WAL)
  ↓
Recovery (Crash Recovery)
```

**OO 文档揭示的关键能力缺口**:

| 链路节点 | v3.0.0 状态 | v3.1.0 需求 |
|---------|------------|-------------|
| Planner → Optimizer | CBO 设计存在但未激活 | **CBO 代价模型接入 planner** |
| Executor → Storage | 聚簇索引缺失 | **聚簇索引实现** |
| Transaction | Gap Locking 缺失 | **Next-Key Lock / Gap Lock** |
| Recovery | WAL 基础存在 | **审计链 crash-safe** |
| DCL 执行 | 解析存在，执行缺失 | **RBAC 执行层** |

---

## 三、OO 文档未充分挖掘的价值

### 3.1 价值 #1: 覆盖缺口图 → 测试用例自动生成

**现状**: `SQL_EXECUTION_MATRIX.md` 记录了每个语句的 Parser/Planner/Optimizer/Executor 覆盖率和状态。

**未被利用**:
- 这些覆盖率数字是静态值，从未通过 `cargo llvm-cov` 实时验证
- 没有建立覆盖率与测试用例的直接映射
- `coverage/` 目录为空

**v3.1.0 改进**: 基于矩阵**自动生成测试用例骨架**:

```python
# 从 OO 文档提取测试缺口，生成测试用例
for stmt in execution_matrix:
    if stmt.coverage < 70%:
        generate_unit_test(stmt.stmt_type, stmt.keywords)
```

### 3.2 价值 #2: CBO 代价模型 → 实际执行计划选择

**现状**: `CBO_COST_MODEL.md` 有完整的代价公式（CPU/IO/Memory/Network 分解），但**从未接入 planner 的 `optimizer.rs`**。

**证据**:
- `crates/optimizer/src/optimizer.rs` 没有 `CostModel` trait 实例化
- v3.1.0 Issue #616 (CBO 激活) 正在进行，但 OO 文档的代价模型**未被引用**

**v3.1.0 改进**:
1. 将 `CBO_COST_MODEL.md` 中的 `CostConstants` Rust 结构实现到 `crates/optimizer/src/cost_model.rs`
2. 建立代价模型与实际执行器的反馈闭环
3. OO 文档中的 `TotalCost = CPUCost + IOCost + ...` 公式应该在代码中可验证

### 3.3 价值 #3: MVCC 可见性判断 → 形式化验证

**现状**: `MVCC_IMPLEMENTATION.md` 有版本链和可见性判断算法，但**从未进行形式化验证**。

**未被利用**:
- 可见性规则（`begin_ts < commit_ts < snapshot_ts`）没有 TLA+ 或 Dafny 证明
- 并发场景（写偏斜、幻读）没有模型检验
- 快照隔离的 SSI（Serializable Snapshot Isolation）没有压力测试框架

**v3.1.0 改进**:
1. 基于 OO 文档的可见性算法，实现 `mvcc/visibility.rs` 的 **TLA+ 规格说明**
2. 增加**反例测试**（counterexample test）：故意构造违反 MVCC 规则的场景，验证系统正确拒绝

### 3.4 价值 #4: WAL 协议 → 审计链集成

**现状**: `wal/WAL_PROTOCOL.md` 分析了 WAL 的 write-ahead 机制，但**审计日志与 WAL 的集成**未被分析。

**关键缺口**:
- 审计条目作为 WAL record 写入 → crash-safe
- SHA-256 链在 WAL replay 时验证 → 篡改检测
- 这两个链路在 OO 文档中没有横向关联

**v3.1.0 改进**:
- 在 OO 文档中新增 `oo/audit/AUDIT_CHAIN_INTEGRATION.md`，描述审计→WAL→恢复的完整链路

---

## 四、基于 OO 文档分析的 v3.1.0 增强建议

### 4.1 执行链路增强（文档 → 代码改进）

| OO 文档揭示的缺口 | 当前实现 | v3.1.0 行动 |
|-----------------|---------|-------------|
| **MERGE 语句 0%** | `executor.rs` 无 MERGE 分支 | PR #613 已实现 → 完善 OO 文档 |
| **NTILE/LEAD/LAG 0%** | 窗口函数未实现 | Issue #621 追踪 |
| **Merge Join 0%** | 只有 Hash Join | `crates/executor/src/merge_join.rs` 新建 |
| **BNL (Block Nested Loop) 0%** | 无 BNL | `crates/executor/src/bnl_join.rs` 新建 |
| **INTERSECT/EXCEPT ~45%** | 实现但测试不足 | 增加差异测试 |
| **Gap Locking** | 无 | `crates/transaction/src/gap_lock.rs` 新建 |
| **聚簇索引** | 只有 Heap 表 | `crates/storage/src/clustered_index.rs` 新建 |
| **存储加密** | 无 AES-256 | `crates/storage/src/aes_cipher.rs` 新建 |
| **审计链** | 基础审计存在 | SHA-256 链 + WAL 集成 |

### 4.2 测试体系增强（OO 文档 → 自动化测试）

| 增强项 | 描述 | 对应 OO 文档 |
|--------|------|------------|
| **覆盖缺口自动扫描** | 解析 `SQL_EXECUTION_MATRIX.md`，生成 `coverage_audit.rs` 测试 | SQL_EXECUTION_MATRIX |
| **CBO 代价模型可验证测试** | 将 `CBO_COST_MODEL.md` 公式实现为可执行的 Rust 测试 | CBO_COST_MODEL |
| **MVCC 可见性规则测试** | 对每个可见性规则写 `#[test]` 验证其行为 | MVCC_IMPLEMENTATION |
| **WAL 崩溃注入测试** | 基于 OO 文档的 5 场景矩阵自动化 | WAL_PROTOCOL |
| **执行链路回归测试** | 对每个语句类型验证 Parser→Executor 链路完整性 | SQL_EXECUTION_MATRIX |

### 4.3 文档体系增强（补全 OO 执行链路文档）

**P0 补全**（v3.1.0-alpha 前）:

| 文档 | 内容 | 对应 OO 章节 |
|------|------|------------|
| `oo/dml/MERGE_EXECUTION.md` | MERGE 语句完整执行链路 | SQL_EXECUTION_MATRIX §2.2 |
| `oo/transaction/GAP_LOCKING.md` | Next-Key Lock 实现分析 | MVCC_IMPLEMENTATION 补充 |
| `oo/storage/CLUSTERED_INDEX.md` | 聚簇索引设计与实现 | BPTREE_DESIGN 补充 |
| `oo/storage/ENCRYPTION.md` | AES-256-GCM 存储加密链路 | WAL_PROTOCOL 补充 |
| `oo/audit/AUDIT_CHAIN.md` | 审计链 SHA-256 + WAL 集成 | WAL_PROTOCOL + MVCC |
| `oo/join/MERGE_JOIN.md` | Merge Join 算法实现 | JOIN_ALGORITHMS 补充 |
| `oo/join/BNL_JOIN.md` | Block Nested Loop Join | JOIN_ALGORITHMS 补充 |
| `oo/cbo/CBO_INTEGRATION.md` | CBO 代价模型接入 planner | CBO_COST_MODEL 补充 |
| `oo/query/WINDOW_FUNCTION_NTILE.md` | NTILE/LEAD/LAG 实现 | WINDOW_FUNCTIONS 补充 |
| `oo/query/MULTI_TABLE_UPDATE_DELETE.md` | 多表 UPDATE/DELETE 执行 | DML 链路补充 |

**P1 补全**（v3.1.0-beta 前）:

| 文档 | 内容 |
|------|------|
| `oo/dcl/RBAC_EXECUTION.md` | RBAC 执行层（GRANT/REVOKE 实际生效）|
| `oo/ddl/ONLINE_DDL.md` | 在线 DDL（INPLACE 算法）|
| `oo/advanced/EVENT_SCHEDULER.md` | 事件调度器 |
| `oo/distributed/FAULT_TOLERANCE.md` | 自动故障转移 |

---

## 五、执行计划

### 5.1 立即行动（本周）

- [ ] **修复 OO 文档真实性**: 删除 README 中标记为 ✅ 但文件不存在的虚假记录，或补充实际文件
- [ ] **OO 文档覆盖矩阵 → Issue 映射**: 将 `SQL_EXECUTION_MATRIX.md` 中所有 ❌ 标记的条目创建为 Issue
- [ ] **创建 OO 文档质量门禁**: `scripts/gate/check_oo_docs.sh`，验证 README 标记与文件系统一致

### 5.2 短期（v3.1.0-alpha 前）

- [ ] **CBO 代价模型激活**: 将 `CBO_COST_MODEL.md` 中的 `CostConstants` Rust 实现到代码中
- [ ] **MERGE 执行链路文档**: 基于 PR #613 补充 `oo/dml/MERGE_EXECUTION.md`
- [ ] **Gap Locking 文档**: 基于 Issue #607 补充 `oo/transaction/GAP_LOCKING.md`
- [ ] **审计链文档**: 补充 `oo/audit/AUDIT_CHAIN.md`

### 5.3 中期（v3.1.0-beta 前）

- [ ] **MVCC 形式化验证**: 基于 `MVCC_IMPLEMENTATION.md` 的可见性规则写 TLA+ 规格
- [ ] **WAL 崩溃注入自动化**: 将 OO 文档的 5 场景矩阵实现为 `#[test]` 混沌测试
- [ ] **覆盖缺口自动扫描**: 解析 `SQL_EXECUTION_MATRIX.md`，生成测试覆盖率报告
- [ ] **多表 DML 文档**: `oo/query/MULTI_TABLE_UPDATE_DELETE.md`

### 5.4 长期（v3.2.0）

- [ ] **RBAC 执行层**: 基于 `DCL_EXECUTION.md` 将解析层升级为执行层
- [ ] **在线 DDL**: 基于 `ALTER_EXECUTION.md` 升级为 INPLACE 算法
- [ ] **事件调度器**: 基于 `EVENT_SCHEDULER.md` 实现完整调度器
- [ ] **自动故障转移**: 基于 `FAULT_TOLERANCE.md` 实现 Raft 风格选主

---

## 六、核心结论

### OO 文档的核心价值

| 价值维度 | 现状 | v3.1.0 目标 |
|---------|------|-------------|
| **纵向执行链路** | 部分覆盖（DML/DDL/DCL/TCL/查询） | **补全 MERGE/GapLock/聚簇索引/加密链路** |
| **横向模块矩阵** | ✅ `SQL_EXECUTION_MATRIX.md` 完整 | 实时同步到代码覆盖测试 |
| **代价模型** | `CBO_COST_MODEL.md` 详细但未激活 | **接入 planner，验证可执行** |
| **事务正确性** | `MVCC_IMPLEMENTATION.md` 分析深入 | **TLA+ 形式化验证** |
| **崩溃恢复** | `WAL_PROTOCOL.md` + 5 场景矩阵 | **自动化 chaos 测试** |
| **文档真实性** | ❌ README 标记与文件系统不符 | **建立 OO 文档门禁** |

### v3.1.0 关键增强方向

1. **CBO 代价模型从文档到代码**: `CBO_COST_MODEL.md` 的 43KB 代价公式必须在 `crates/optimizer/src/cost_model.rs` 中可执行、可测试
2. **MVCC 可见性规则形式化**: `MVCC_IMPLEMENTATION.md` 的可见性算法必须变成 TLA+ 规格 + 反例测试
3. **WAL + 审计链集成**: WAL 协议 + SHA-256 链在 OO 文档中横向关联，在代码中统一实现
4. **OO 文档门禁**: 建立 `check_oo_docs.sh`，确保 README 标记与文件系统一致，防止文档腐化
5. **测试用例从 OO 矩阵自动生成**: `SQL_EXECUTION_MATRIX.md` 的覆盖缺口直接映射到测试文件
