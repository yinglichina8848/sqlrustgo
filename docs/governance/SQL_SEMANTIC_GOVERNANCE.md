# SQLRustGo SQL 语义治理指南

> **版本**: 1.0
> **更新日期**: 2026-04-24
> **维护人**: yinglichina8848
> **适用范围**: SQLRustGo 全链路 SQL 语义正确性（Parser → Planner → Executor → Storage）
> **前置文档**: `docs/governance/CONTRIBUTING.md`

---

## 1. 核心问题：系统缺乏统一的 SQL 语义执行模型

### 1.1 表现

- 多点实现（eval_predicate 分散在 5+ 个模块）
- 行为不一致（NULL = NULL 在 JOIN 中匹配，在 WHERE 中不匹配）
- 难以验证正确性（无语义级测试矩阵）

### 1.2 本质

> ❗ 数据库不是普通程序，SQL 语义正确性 = 数学正确性

---

## 2. 分层治理模型

```
Layer 1: Parser（语法能力 — 能表达 SQL）
Layer 2: Planner（逻辑计划 — 生成正确计划）
Layer 3: Executor（执行语义 — 语义正确执行）
Layer 4: Storage（数据一致性 — 数据正确）
```

| 层 | 目标 | 当前状态 |
|---|------|----------|
| Parser | 能表达 SQL | ⚠️ IS NULL 为变通实现 |
| Planner | 生成正确计划 | ✅ Volcano 模型正确 |
| Executor | 语义正确执行 | ❌ 多处语义缺陷 |
| Storage | 数据正确 | ✅ NULL 填充正确 |

---

## 3. Executor 层语义缺陷清单（已确认）

### 3.1 🔴 高严重度

| 编号 | 问题 | 位置 | SQL 标准 |
|------|------|------|----------|
| E-1 | `NULL = NULL` 返回 `true` | `execution_engine.rs` evaluate_binary_comparison | 应返回 UNKNOWN（WHERE 中视为 false） |
| E-2 | `execute_select_with_join` 未应用 WHERE 子句 | `execution_engine.rs:615-725` | FROM → JOIN → WHERE |
| E-3 | `COUNT(col)` 不跳过 NULL | `execution_engine.rs:548` | COUNT(col) 应排除 NULL |

### 3.2 🟠 中严重度

| 编号 | 问题 | 位置 | SQL 标准 |
|------|------|------|----------|
| E-4 | `SUM` 全 NULL 时返回 `0` 而非 `NULL` | `execution_engine.rs:549-561` | 全 NULL → NULL |
| E-5 | JOIN 中 NULL 键值用 `format!("{:?}")` 匹配 | `execution_engine.rs:659` | NULL 键不应匹配 |
| E-6 | 聚合函数只处理 Integer，不支持 Float | `execution_engine.rs:549-608` | 应支持所有数值类型 |
| E-7 | NOT / AND / OR 语义不完整 | `execution_engine.rs` | 需三值逻辑 |

### 3.3 🟡 低严重度

| 编号 | 问题 | 位置 |
|------|------|------|
| E-8 | 向量搜索 eval_predicate 不支持 IS NULL | `sql_vector_hybrid.rs` |
| E-9 | 图查询 evaluate_predicate 不遵循 NULL 语义 | `cypher/executor.rs` |

---

## 4. Parser 层语义缺陷清单

| 编号 | 问题 | 位置 | 说明 |
|------|------|------|------|
| P-1 | `IS` 未定义为独立 Token | `token.rs` / `lexer.rs` | 被解析为 Identifier |
| P-2 | IS NULL 无结构化 AST 节点 | `parser.rs:1609-1680` | 作为 BinaryOp("IS", Literal("NULL")) 变通 |
| P-3 | `col = NULL` 未被拒绝 | `parser.rs` | SQL 标准中 `= NULL` 非法，应提示使用 IS NULL |

---

## 5. 治理方法

### 方法 1：语义矩阵

对每个模块验证：`功能 × 边界 × 组合`

| 功能 | 边界 | 组合 |
|------|------|------|
| JOIN | NULL | JOIN + WHERE |
| Filter | NULL | Filter + Aggregate |
| Aggregate | NULL | GROUP BY + HAVING |
| Comparison | NULL | = / != / IS NULL |

### 方法 2：关键路径审查

SQL 执行中最容易出错的路径，按优先级：

1. JOIN（NULL 匹配 + 执行顺序）
2. WHERE（NULL 过滤 + 三值逻辑）
3. GROUP BY（NULL 分组语义）
4. Aggregate（COUNT/SUM/AVG 的 NULL 处理）
5. Subquery（未来）

### 方法 3：语义护城河测试

每个模块必须有语义级测试，锁定 SQL 标准行为：

| 类型 | SQL | 预期 | 测试目的 |
|------|-----|------|----------|
| JOIN + NULL | `SELECT * FROM t1 LEFT JOIN t2 ON t1.id = t2.id WHERE t2.id IS NULL` | 仅返回未匹配行 | JOIN NULL + WHERE 顺序 |
| Filter NULL | `SELECT * FROM t WHERE col = NULL` | 空结果（或语法错误） | = NULL 非法语义 |
| IS NULL | `SELECT * FROM t WHERE col IS NULL` | 返回 NULL 行 | IS NULL 基础语义 |
| COUNT | `SELECT COUNT(col), COUNT(*) FROM t` | COUNT(col) < COUNT(*) | COUNT NULL 排除 |
| SUM NULL | `SELECT SUM(col) FROM t`（全 NULL） | NULL | SUM 全 NULL 语义 |

### 方法 4：单一语义入口

当前状态（分散）：

```
execution_engine.rs  → evaluate_where_clause()
execution_engine.rs  → evaluate_binary_comparison()
storage/predicate.rs → Predicate::eval()
columnar/storage.rs  → eval_predicate_for_scan()
vector/hybrid.rs     → eval_predicate()
graph/cypher.rs      → evaluate_predicate()
```

目标状态（收敛）：

```
eval_predicate(expr) -> bool     ← 所有谓词统一入口
eval_aggregate(...) -> Value     ← 所有聚合统一入口
eval_join_key(...) -> bool       ← JOIN 键匹配统一入口
eval_expression(...) -> Value    ← 表达式求值统一入口
```

### 方法 5：阶段化演进

```
Phase 1: bool + NULL 折叠（UNKNOWN → FALSE，仅 WHERE）
Phase 2: Option<bool>（区分 TRUE / FALSE / UNKNOWN）
Phase 3: TriBool（完整 SQL 三值逻辑）
```

---

## 6. 全局 SQL 审查清单

### 🔴 Executor 层

- [ ] 所有 predicate 是否统一入口？
- [ ] NULL = NULL 是否返回 false（WHERE 中）？
- [ ] JOIN 是否遵守 SQL 语义（NULL 键不匹配）？
- [ ] WHERE 是否在 JOIN 后执行？
- [ ] COUNT(col) 是否跳过 NULL？
- [ ] SUM 全 NULL 是否返回 NULL？

### 🟠 Parser 层

- [ ] IS NULL 是否有独立 AST 节点？
- [ ] IS NOT NULL 是否支持？
- [ ] `col = NULL` 是否被拒绝或警告？
- [ ] NOT / AND / OR 是否完整支持？

### 🟡 Planner 层

- [ ] 是否保持语义不变？
- [ ] 是否错误重排 JOIN + WHERE？

### 🔵 Aggregate 层

- [ ] COUNT(col) vs COUNT(*) 是否区分？
- [ ] NULL 是否被正确忽略？
- [ ] GROUP BY key 中 NULL 是否正确分组？
- [ ] HAVING 中 NULL 语义是否正确？

---

## 7. 当前语义覆盖

| 能力 | 状态 | 说明 |
|------|------|------|
| NULL 比较 (= / !=) | ❌ | NULL = NULL 错误返回 true |
| JOIN NULL | ❌ | NULL 键错误匹配 |
| WHERE 过滤 | ⚠️ | JOIN 场景未应用 WHERE |
| IS NULL / IS NOT NULL | ✅ | 变通实现但功能正确 |
| LEFT/RIGHT/FULL JOIN NULL 填充 | ✅ | 正确 |
| COUNT(*) | ✅ | 正确 |
| COUNT(col) | ❌ | 不跳过 NULL |
| SUM (全 NULL) | ❌ | 返回 0 而非 NULL |
| AVG (全 NULL) | ✅ | 返回 NULL |
| MIN / MAX (NULL) | ✅ | 跳过 NULL，全 NULL 返回 NULL |
| NOT / AND / OR | ⚠️ | 未实现三值逻辑 |

---

## 8. 演进路线

| Phase | 能力 | 目标 |
|-------|------|------|
| Phase 1 | bool + NULL 折叠 | UNKNOWN → FALSE（仅 WHERE），修复 E-1/E-2/E-3 |
| Phase 2 | Option\<bool\> | 区分 TRUE / FALSE / UNKNOWN，修复 E-4/E-5/E-7 |
| Phase 3 | TriBool | 完整 SQL 三值逻辑，修复 P-1/P-2/P-3 |

### 优先级排序

| 优先级 | 任务 | 依赖 |
|--------|------|------|
| 🔥 P1 | Aggregate + NULL 语义（E-3/E-4） | 无 |
| 🔥 P1 | JOIN WHERE 执行顺序（E-2） | 无 |
| 🔥 P1 | NULL = NULL 修复（E-1） | 无 |
| 🔥 P2 | NOT / AND / OR 三值逻辑（E-7） | Phase 2 |
| 🔥 P2 | Parser IS NULL AST 节点（P-1/P-2） | Phase 2 |
| 🔥 P3 | Planner 语义正确性验证 | Phase 2 |

---

## 9. Prompt 模板（AI 辅助治理）

### 目标型 Prompt

```text
请对 SQLRustGo 的 <模块> 进行语义审查（semantic audit），目标：

1. 找出所有违反 SQL 标准语义的实现
2. 识别 NULL / JOIN / FILTER / AGGREGATE 相关问题
3. 输出：
   - 问题列表（按严重性排序）
   - 代码位置
   - 修复建议
   - 是否需要测试补充
```

### 深度分析 Prompt

```text
分析 SQLRustGo 中 <模块> 的执行路径：

- 输入：SQL → AST → Plan → Execution
- 找出所有可能的语义不一致点

重点关注：
- NULL 处理
- JOIN 语义
- WHERE 执行顺序
- 聚合函数行为

输出：
1. 执行路径图
2. 未覆盖的语义分支
3. 高风险 bug 点（Top 10）
```

### 测试生成 Prompt

```text
基于 SQL 标准语义，为 <模块> 设计一组"语义护城河测试"，要求：

- 覆盖 NULL / JOIN / FILTER / AGGREGATE
- 优先组合场景（JOIN + WHERE 等）
- 每个测试说明它锁定的语义

输出：
- SQL
- 输入数据
- 预期结果
- 测试目的
```

### 架构收敛 Prompt

```text
当前 <模块> 存在多处语义分散实现，请：

1. 识别所有重复/分散的语义逻辑
2. 设计单一语义入口（如 eval_xxx）
3. 给出渐进式重构方案（Phase 1/2/3）
```

---

## 10. 本次改造意义

> 从"功能驱动开发"升级为"语义驱动开发"

传统开发：功能实现 → 测试通过 → 发布

语义驱动：语义规范 → 语义测试 → 实现 → 语义验证

---

**创建日期**: 2026-04-24
**最后更新**: 2026-04-24
