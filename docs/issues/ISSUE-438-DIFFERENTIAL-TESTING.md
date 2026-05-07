# ISSUE-438: Differential Testing 框架 (BP2-3)

## 基本信息

| 字段 | 内容 |
|------|------|
| **Issue ID** | ISSUE-438 |
| **标题** | Differential Testing 框架 (BP2-3) |
| **优先级** | P0 |
| **类型** | Feature / Testing |
| **创建日期** | 2026-05-07 |
| **目标版本** | v3.0.0 |
| **状态** | In Progress |
| **Gate** | BP2-3: `cargo test -p sqlrustgo-sql-corpus` |

---

## 一、背景

### 1.1 问题

当前 SQLRustGo 缺乏与标准 SQL 引擎 (MySQL 5.7) 的结果比对能力，无法量化兼容度。

### 1.2 目标

建立 SQLRustGo 与 MySQL 5.7 的自动结果比对框架，真正知道兼容多少，不是猜测。

---

## 二、架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│              Differential Testing Framework                    │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │   SQLRustGo  │    │    MySQL    │    │  Comparator │    │
│  │    Runner    │    │   5.7 Runner │    │             │    │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘    │
│         │                   │                   │            │
│         └───────────────────┼───────────────────┘            │
│                             ▼                                │
│                    ┌─────────────────┐                       │
│                    │  DiffResult     │                       │
│                    │  (Match/Mismatch│                       │
│                    │   /Error)       │                       │
│                    └─────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心组件

| 组件 | 职责 |
|------|------|
| `SqlEngine` trait | SQL 引擎抽象接口 |
| `SqlRustGoRunner` | SQLRustGo 执行器 |
| `MySqlRunner` | MySQL 5.7 执行器 |
| `ResultComparator` | 结果比对 (排序、NULL 处理) |
| `DifferentialCorpus` | 差分测试入口 |

---

## 三、实现计划

### Phase 1: 核心框架

- [ ] 定义 `SqlEngine` trait
- [ ] 实现 `SqlRustGoRunner`
- [ ] 实现 `MySqlRunner` (MySQL 可用时)
- [ ] 实现 `ResultComparator`
- [ ] 实现 `DifferentialCorpus`

### Phase 2: SQL Corpus 扩展

- [ ] 基础 SQL (5,000) - 现有 DML/DDL 扩充
- [ ] 边界条件 (5,000) - 类型溢出、精度问题
- [ ] NULL 处理 (1,000) - NULL 比较、COALESCE
- [ ] 事务 (2,000) - 并发、隔离级别

### Phase 3: 集成与门禁

- [ ] 集成到 `cargo test -p sqlrustgo-sql-corpus`
- [ ] 生成兼容性报告
- [ ] 设置 ≥85% 通过率门禁

---

## 四、验收条件

| 指标 | 目标 |
|------|------|
| SQL cases 数量 | ≥10,000 |
| MySQL 5.7 结果比对通过率 | ≥85% |
| 执行时间 | < 30 分钟 (10,000 cases) |

---

## 五、相关文件

| 文件 | 说明 |
|------|------|
| `crates/sql-corpus/src/lib.rs` | 现有 SQL Corpus 实现 |
| `crates/sql-corpus/src/differential.rs` | 差分测试框架 (新建) |
| `sql_corpus/DIFFERENTIAL/` | 差分测试用例 |

---

## 六、修改历史

| 日期 | 修改人 | 修改内容 |
|------|--------|----------|
| 2026-05-08 | Claude | 初始创建 |
