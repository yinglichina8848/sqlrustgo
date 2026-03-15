# SQLRustGo v1.4.0 发布门禁检查清单

> **版本**: v1.4.0
> **制定日期**: 2026-03-15
> **当前阶段**: 🚧 **Alpha** - SQL Engine 完整化
> **发布类型**: SQL 查询能力增强版
> **目标成熟度**: L3 (Mini DBMS)

---

## 一、发布概览

### 1.1 版本信息

| 项目 | 值 |
|------|------|
| 版本号 | v1.4.0 |
| 发布类型 | SQL Engine 完整化 |
| 目标分支 | release/v1.4.0 |
| 开发分支 | develop/v1.4.0 |
| 前置版本 | v1.3.0 (GA) |
| 目标成熟度 | L3 (Mini DBMS) |

### 1.2 版本目标

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          v1.4.0 核心目标                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   🎯 架构升级：L2 → L3 Mini DBMS                                           │
│                                                                              │
│   ✅ SQL 能力增强 (JOIN/GROUP BY/ORDER BY/LIMIT/子查询)                    │
│   ✅ Expression Engine (表达式系统)                                         │
│   ✅ Logical Plan (逻辑计划层)                                             │
│   ✅ 基础优化 (谓词下推/投影裁剪/常量折叠)                                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、门禁检查清单

### 2.1 🔴 必须项

#### A. 代码质量门禁

| ID | 检查项 | 状态 | 说明 | 检查结果 |
|----|--------|------|------|----------|
| A-01 | 编译通过 | ⏳ | cargo build --all 无错误 | - |
| A-02 | 测试通过 | ⏳ | cargo test --all 全部通过 | - |
| A-03 | Clippy 检查 | ⏳ | cargo clippy -- -D warnings 无警告 | - |
| A-04 | 格式检查 | ⏳ | cargo fmt --all -- --check 通过 | - |
| A-05 | 无 unwrap/panic | ⏳ | 核心代码无 unwrap/panic 调用 | - |
| A-06 | 错误处理完整 | ⏳ | 使用 SqlResult<T> 统一错误处理 | - |

#### B. 测试覆盖门禁

> 更新日期: 2026-03-15

| ID | 检查项 | 状态 | 当前值 | 目标值 | 说明 |
|----|--------|------|--------|--------|------|
| B-01 | 整体行覆盖率 | ⏳ | - | ≥75% | - |
| B-02 | Expression 行覆盖率 | ⏳ | - | ≥70% | - |
| B-03 | Planner 行覆盖率 | ⏳ | - | ≥75% | - |
| B-04 | Optimizer 行覆盖率 | ⏳ | - | ≥70% | - |

> 注: 测试方法 `cargo llvm-cov --workspace --all-features`，不含测试代码

#### C. 功能完整性门禁

> 更新日期: 2026-03-15

| ID | 检查项 | 状态 | 说明 | Issue/PR |
|----|--------|------|------|----------|
| **Parser 扩展** |||||
| C-01 | JOIN 支持 | ⏳ | INNER/LEFT/RIGHT/CROSS JOIN | - |
| C-02 | GROUP BY 支持 | ⏳ | 分组查询 | - |
| C-03 | HAVING 子句 | ⏳ | 分组过滤 | - |
| C-04 | ORDER BY 支持 | ⏳ | 排序查询 | - |
| C-05 | LIMIT/OFFSET 支持 | ⏳ | 结果限制 | - |
| C-06 | 子查询支持 | ⏳ | SCALAR/IN/EXISTS | - |
| **Expression Engine** |||||
| C-07 | Expression trait | ⏳ | 表达式 trait 定义 | - |
| C-08 | BinaryExpr | ⏳ | 二元表达式 | - |
| C-09 | ColumnRef 表达式 | ⏳ | 列引用 | - |
| C-10 | Literal 表达式 | ⏳ | 常量表达式 | - |
| C-11 | FunctionExpr | ⏳ | 内置函数 | - |
| **Logical Plan** |||||
| C-12 | LogicalPlan trait | ⏳ | 逻辑计划 trait | - |
| C-13 | LogicalProjection | ⏳ | 逻辑投影 | - |
| C-14 | LogicalFilter | ⏳ | 逻辑过滤 | - |
| C-15 | LogicalJoin | ⏳ | 逻辑连接 | - |
| C-16 | LogicalAggregate | ⏳ | 逻辑聚合 | - |
| **Executor 完善** |||||
| C-17 | Sort Executor | ⏳ | 排序执行器 | - |
| C-18 | Limit Executor | ⏳ | 限制执行器 | - |
| C-19 | Aggregate Executor | ⏳ | 聚合执行器 | - |
| **基础优化** |||||
| C-20 | 谓词下推 | ⏳ | Predicate Pushdown | - |
| C-21 | 投影裁剪 | ⏳ | Projection Pushdown | - |
| C-22 | 常量折叠 | ⏳ | Constant Folding | - |

### 2.2 🟠 重要项

#### D. 可观测性门禁

| ID | 检查项 | 状态 | 说明 |
|----|--------|------|------|
| D-01 | EXPLAIN ANALYZE | ⏳ | 执行计划分析 |
| D-02 | 慢查询日志 | ⏳ | >1s 阈值记录 |
| D-03 | Query Profiler | ⏳ | 查询性能分析 |

#### E. 性能门禁

| ID | 检查项 | 状态 | 说明 |
|----|--------|------|------|
| E-01 | 性能基准测试 | ⏳ | 继承 v1.3.0 基准 |
| E-02 | 无严重性能退化 | ⏳ | 对比 v1.3.0 |

#### F. 文档门禁

| ID | 检查项 | 状态 | 说明 |
|----|--------|------|------|
| F-01 | Release Notes | ⏳ | [RELEASE_NOTES.md](./RELEASE_NOTES.md) |
| F-02 | CHANGELOG 更新 | ⏳ | CHANGELOG.md 添加 v1.4.0 |
| F-03 | API 文档注释 | ⏳ | 公共 API 文档 |
| F-04 | SQL 兼容性文档 | ⏳ | 支持的 SQL 语法说明 |

#### G. 安全门禁

| ID | 检查项 | 状态 | 说明 |
|----|--------|------|------|
| G-01 | 依赖审计 | ⏳ | cargo audit |
| G-02 | 敏感信息检查 | ⏳ | 无密钥泄露 |

### 2.3 🟡 建议项

#### H. 工程化门禁

| ID | 检查项 | 状态 | 说明 |
|----|--------|------|------|
| H-01 | CI 流程完整 | ⏳ | GitHub Actions |
| H-02 | 分支保护配置 | ⏳ | develop/v1.4.0 |
| H-03 | 代码所有者 | ⏳ | CODEOWNERS |
| H-04 | Issue 关联 | ⏳ | PR 关联 Issue |
| H-05 | Commit 规范 | ⏳ | Conventional Commits |

---

## 三、发布流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          v1.4.0 发布流程                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Phase 1: 开发阶段 (进行中)                                               │
│   ├── Parser 扩展                                                           │
│   ├── Expression Engine                                                    │
│   ├── Logical Plan                                                         │
│   ├── Executor 完善                                                        │
│   └── 基础优化                                                             │
│                                                                              │
│   Phase 2: Alpha 验证 (当前)                                               │
│   ├── 2.1 核心功能可用                                                    │
│   ├── 2.2 基本测试通过                                                    │
│   └── 2.3 基础门禁检查                                                    │
│                                                                              │
│   Phase 3: Beta 验证                                                       │
│   ├── 3.1 完整测试套件                                                    │
│   ├── 3.2 覆盖率检查                                                      │
│   ├── 3.3 安全审计                                                        │
│   └── 3.4 文档完善                                                        │
│                                                                              │
│   Phase 4: RC 验证                                                         │
│   ├── 4.1 回归测试                                                        │
│   ├── 4.2 性能基准                                                        │
│   └── 4.3 发布准备                                                         │
│                                                                              │
│   Phase 5: 发布                                                            │
│   ├── 5.1 创建 v1.4.0 Tag                                                │
│   ├── 5.2 发布 GitHub Release                                             │
│   └── 5.3 合并到 main 分支                                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 四、检查命令

```bash
# 代码质量
cargo build --all
cargo test --all
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check

# 测试覆盖率 (使用 llvm-cov)
cargo llvm-cov --workspace --all-features

# 安全审计
cargo audit
```

---

## 五、SQL 能力目标

### 5.1 v1.4 完成后可执行的 SQL

```sql
-- 必须支持
SELECT * FROM t1 JOIN t2 ON t1.id = t2.id
SELECT a, COUNT(*), SUM(b) FROM t GROUP BY a HAVING COUNT(*) > 1
SELECT * FROM t ORDER BY a LIMIT 10 OFFSET 20
SELECT * FROM t WHERE a IN (SELECT id FROM t2)
SELECT * FROM t WHERE a > (SELECT MAX(b) FROM t2)
```

---

## 六、相关文档

- [VERSION_PLAN.md](./VERSION_PLAN.md)
- [RELEASE_NOTES.md](./RELEASE_NOTES.md)
- [v1.3.0 发布门禁](../v1.3.0/RELEASE_GATE_CHECKLIST.md)

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-15 | 初始版本 |

---

**文档状态**: Alpha  
**制定日期**: 2026-03-15  
**制定人**: yinglichina8848
