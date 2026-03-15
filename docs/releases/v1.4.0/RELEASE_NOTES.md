# SQLRustGo v1.4.0 Release Notes

> **版本**: v1.4.0
> **发布日期**: 待定
> **代号**: SQL Engine 完整化
> **当前阶段**: Alpha

---

## 一、版本概述

### 1.1 发布类型

| 项目 | 值 |
|------|------|
| 版本号 | v1.4.0 |
| 发布类型 | SQL Engine 完整化 |
| 目标分支 | release/v1.4.0 |
| 开发分支 | develop/v1.4.0 |
| 前置版本 | v1.3.0 (GA) |

### 1.2 核心特性

v1.4.0 是 SQL Engine 完整化版本，实现完整 SQL 查询能力：

- **SQL 能力增强**: JOIN/GROUP BY/ORDER BY/LIMIT/子查询
- **Expression Engine**: 统一的表达式求值系统
- **Logical Plan**: 逻辑计划层，与物理计划分离
- **基础优化**: 谓词下推、投影裁剪、常量折叠

---

## 二、版本目标

### 2.1 架构升级

```
v1.3 (L2) → v1.4 (L3 Mini DBMS)
```

### 2.2 技术架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         v1.4.0 架构                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  SQL Input                                                        │
│       │                                                           │
│       ▼                                                           │
│  ┌─────────┐     ┌──────────────┐     ┌─────────────────┐     │
│  │ Parser  │────►│ LogicalPlan  │────►│  PhysicalPlan   │     │
│  │  扩展   │     │              │     │   (Executor)   │     │
│  └─────────┘     └──────────────┘     └─────────────────┘     │
│       │                │                        │               │
│       │                ▼                        ▼               │
│       │         ┌──────────────┐     ┌─────────────────┐      │
│       └────────►│ Expression   │────►│   Volcano       │      │
│                │   Engine      │     │   Execution     │      │
│                └──────────────┘     └─────────────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、功能变更

### 3.1 新增功能

#### Parser 扩展

- **JOIN 支持**: INNER/LEFT/RIGHT/CROSS JOIN
- **GROUP BY 支持**: 分组查询
- **HAVING 子句**: 分组过滤
- **ORDER BY 支持**: 排序查询
- **LIMIT/OFFSET 支持**: 结果限制
- **子查询支持**: SCALAR/IN/EXISTS

#### Expression Engine

- **Expression trait**: 表达式 trait 定义
- **BinaryExpr**: 二元表达式 (AND/OR/=/</>/<=/>=)
- **ColumnRef**: 列引用表达式
- **Literal**: 常量表达式
- **FunctionExpr**: 内置函数

#### Logical Plan

- **LogicalPlan trait**: 逻辑计划 trait
- **LogicalProjection**: 逻辑投影
- **LogicalFilter**: 逻辑过滤
- **LogicalJoin**: 逻辑连接
- **LogicalAggregate**: 逻辑聚合

#### Executor 完善

- **Sort Executor**: 排序执行器
- **Limit Executor**: 限制执行器
- **Aggregate Executor**: 聚合执行器

#### 基础优化

- **谓词下推**: Predicate Pushdown
- **投影裁剪**: Projection Pushdown
- **常量折叠**: Constant Folding

---

## 四、SQL 兼容性

### 4.1 支持的 SQL 语法

```sql
-- JOIN
SELECT * FROM t1 JOIN t2 ON t1.id = t2.id
SELECT * FROM t1 LEFT JOIN t2 ON t1.id = t2.id

-- GROUP BY
SELECT a, COUNT(*), SUM(b) FROM t GROUP BY a
SELECT a, COUNT(*) FROM t GROUP BY a HAVING COUNT(*) > 1

-- ORDER BY / LIMIT
SELECT * FROM t ORDER BY a LIMIT 10
SELECT * FROM t ORDER BY a LIMIT 10 OFFSET 20

-- 子查询
SELECT * FROM t WHERE a IN (SELECT id FROM t2)
SELECT * FROM t WHERE a > (SELECT MAX(b) FROM t2)
```

---

## 五、里程碑

| 阶段 | 日期 | 说明 |
|------|------|------|
| Alpha | 待定 | 核心功能可用 |
| Beta | 待定 | 完整测试通过 |
| RC | 待定 | 发布候选 |
| GA | 待定 | 正式发布 |

---

## 六、升级说明

### 6.1 从 v1.3.0 升级

v1.4.0 包含以下不兼容变更：

- Parser 接口扩展
- Expression trait 变更
- LogicalPlan 新增

### 6.2 依赖更新

请参考 Cargo.toml 更新依赖。

---

## 七、已知问题

- 子查询优化仍在完善中
- 部分复杂 JOIN 场景可能性能不佳

---

## 八、相关文档

- [VERSION_PLAN.md](./VERSION_PLAN.md)
- [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md)
- [v1.3.0 Release Notes](../v1.3.0/RELEASE_NOTES.md)

---

## 九、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-15 | 初始版本 |

---

**文档状态**: Alpha  
**制定日期**: 2026-03-15  
**制定人**: yinglichina8848
