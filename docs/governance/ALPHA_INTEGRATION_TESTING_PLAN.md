# v3.0.0 Alpha 阶段：全面集成与测试计划

> **创建时间**: 2026-05-06
> **阶段**: v3.0.0-Alpha
> **类型**: 集成/测试总控
> **关联 Issue**: #370

---

## 一、背景

Issue #370 定义了 v3.0.0 Alpha 阶段的核心任务：

**已确认完成**：
- ✅ 优化器 P0 激活（ConstantFolding / PredicatePushdown / ProjectionPruning 桥接到 crates/optimizer 真实实现）
- ✅ SQL Corpus 100%
- ✅ 22/22 TPC-H 可运行

**待完成**：
- P0: SQL 兼容性完整验证
- P0: 执行引擎正确性验证（HashJoin / SortMergeJoin）
- P0: 事务隔离级别验证（Read Committed / Snapshot Isolation）
- A-HYG: 单元测试、覆盖率

---

## 二、A-Gate 门禁定义（基于 Issue #370）

### 2.1 门禁分类矩阵

| 门禁编号 | 类别 | P0 必须项 | P1 期望项 |
|----------|------|-----------|-----------|
| A-OPT | 优化器激活 | ConstantFolding / PredicatePushdown / ProjectionPruning 真实调用 | CboOptimizer 集成 |
| A-SQL | SQL 兼容性 | IN / DISTINCT / CASE / COALESCE 语法执行 | 窗口函数 / CTE / UNION |
| A-EXEC | 执行引擎 | HashJoin / SortMergeJoin 正确执行 | 向量化执行 |
| A-TX | 事务隔离 | Read Committed / Snapshot Isolation 正确 | SSI Serializable |
| A-HYG | 代码质量 | 编译 / Clippy / 格式化 / 文档链接 / 安全 | 覆盖率 ≥50% |

### 2.2 A-OPT: 优化器激活门禁

| ID | 检查项 | 验证方法 | 通过标准 |
|----|--------|----------|----------|
| A-OPT-1 | ConstantFolding 激活 | `SELECT 1+2+3` | 返回常量折叠结果 `6` |
| A-OPT-2 | PredicatePushdown 激活 | `EXPLAIN SELECT * FROM t WHERE c>10` | filter 在 scan 之下 |
| A-OPT-3 | ProjectionPruning 激活 | `SELECT c FROM t WHERE id=1` | 只读取 c 列，无冗余列 |
| A-OPT-4 | 优化器桥接验证 | `cargo test optimizer_bridge` | 测试通过 |

### 2.3 A-SQL: SQL 兼容性门禁

| ID | 检查项 | 验证方法 | 通过标准 |
|----|--------|----------|----------|
| A-SQL-1 | IN 语法 | `SELECT * FROM t WHERE id IN (1,2,3)` | 返回匹配行 |
| A-SQL-2 | DISTINCT | `SELECT DISTINCT c FROM t` | 去重正确 |
| A-SQL-3 | CASE 表达式 | `SELECT CASE WHEN c>0 THEN 1 ELSE 0 END FROM t` | 分支正确 |
| A-SQL-4 | COALESCE | `SELECT COALESCE(NULL, NULL, c) FROM t` | 返回 c 值 |
| A-SQL-5 | IN 子查询 | `SELECT * FROM t WHERE c IN (SELECT c FROM t2)` | 子查询正确 |
| A-SQL-6 | EXISTS 子查询 | `SELECT * FROM t WHERE EXISTS (SELECT 1 FROM t2)` | 布尔值正确 |

### 2.4 A-EXEC: 执行引擎门禁

| ID | 检查项 | 验证方法 | 通过标准 |
|----|--------|----------|----------|
| A-EXEC-1 | HashJoin | `SELECT * FROM t1 JOIN t2 ON t1.id=t2.id` | 结果正确无丢失 |
| A-EXEC-2 | SortMergeJoin | 大数据量 JOIN | 与 HashJoin 结果一致 |
| A-EXEC-3 | 聚合函数 | `SELECT COUNT(*),SUM(c),AVG(c) FROM t` | 数值正确 |
| A-EXEC-4 | GROUP BY | `SELECT c,COUNT(*) FROM t GROUP BY c` | 分组正确 |
| A-EXEC-5 | ORDER BY | `SELECT * FROM t ORDER BY c DESC` | 排序正确 |
| A-EXEC-6 | LIMIT/OFFSET | `SELECT * FROM t LIMIT 10 OFFSET 5` | 返回正确子集 |

### 2.5 A-TX: 事务隔离门禁

| ID | 检查项 | 验证方法 | 通过标准 |
|----|--------|----------|----------|
| A-TX-1 | Read Committed | 并发 UPDATE 串行化 | 后提交者阻塞或覆盖 |
| A-TX-2 | Snapshot Isolation | 事务内多次读取 | 结果一致 |
| A-TX-3 | 并发写冲突 | 两事务同时修改同记录 | 只有一个成功或报错 |
| A-TX-4 | 回滚正确性 | 事务回滚 | 数据恢复到事务前 |

### 2.6 A-HYG: 代码质量门禁

| ID | 检查项 | 命令/方法 | 通过标准 |
|----|--------|----------|----------|
| A-HYG-1 | 编译 | `cargo build --all-features --workspace` | 无错误 |
| A-HYG-2 | 单元测试 | `cargo test --all-features --workspace` | ≥80% 通过 |
| A-HYG-3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 |
| A-HYG-4 | 格式化 | `cargo fmt --all -- --check` | 无 diff |
| A-HYG-5 | 文档链接 | `bash scripts/gate/check_docs_links.sh` | 无死链 |
| A-HYG-6 | 覆盖率 | `cargo llvm-cov --all-features` | 整体 ≥50% |
| A-HYG-7 | 安全扫描 | `cargo audit` | 无高危漏洞 |

---

## 三、当前 A-HYG 执行状态

| ID | 检查项 | 命令 | 结果 | 备注 |
|----|--------|------|------|------|
| A-HYG-1 | 编译 | `cargo build --all-features` | ✅ | 55.23s, 3 warnings |
| A-HYG-2 | 单元测试 | `cargo test --all-features` | ⏳ | 待执行 |
| A-HYG-3 | Clippy | `cargo clippy --all-features -- -D warnings` | ✅ | 0 warnings |
| A-HYG-4 | 格式化 | `cargo fmt --all -- --check` | ✅ | 无 diff |
| A-HYG-5 | 文档链接 | `bash scripts/gate/check_docs_links.sh` | ✅ | All links valid |
| A-HYG-6 | 覆盖率 | `cargo llvm-cov --all-features` | ⏳ | 待执行 |
| A-HYG-7 | 安全扫描 | `cargo audit` | ✅ | 0 高危, 7 advisory (allowed) |

---

## 四、分阶段测试计划

### Phase 1: 优化器激活验证 (Alpha-1)

```bash
cargo test --all-features optimizer_bridge
cargo test --all-features predicate_pushdown
cargo test --all-features constant_folding
```

### Phase 2: SQL 兼容性全面测试 (Alpha-2)

```bash
cargo run --bin sql-corpus -- --suites standard
# 目标: IN/DISTINCT/CASE/COALESCE 100% pass
```

### Phase 3: 执行引擎验证 (Alpha-3)

```bash
cargo test --all-features hash_join
cargo test --all-features sort_merge_join
cargo test --all-features aggregate
```

### Phase 4: 事务与并发 (Alpha-4)

```bash
cargo test --all-features transaction
cargo test --all-features concurrent
```

---

## 五、覆盖率目标

| 模块 | Alpha 目标 | 说明 |
|------|-------------|------|
| executor | ≥45% | 核心执行引擎 |
| optimizer | ≥40% | CBO 和规则优化 |
| storage | ≥15% | 存储层基础 |
| catalog | ≥50% | 元数据管理 |
| parser | ≥50% | SQL 解析 |
| **整体** | **≥50%** | 项目总覆盖率 |

---

## 六、Nomad 集群状态

| 节点 | ID | 状态 | Docker |
|------|-----|------|--------|
| HP Z6G4 | `98a7c88c` | **ready** | ✅ 29.4.2 |
| 250 MacMini | `cc67e32a` | **ready** | ✅ 29.4.2 |

---

## 七、Issue 跟踪

| Issue | 类型 |
|--------|------|
| #353 | v3.0.0 开发总控 |
| #370 | v3.0.0 Alpha 全面集成与测试 (本文档关联) |

---

*文档版本: 2.0*  
*创建: Hermes Agent*  
*最后更新: 2026-05-06*
