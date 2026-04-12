# [Epic-06] MySQL 兼容语法

## 概述

实现 MySQL 兼容语法，让 SQLRustGo 可替代 MySQL 进行教学。

**优先级**: P0
**来源**: 原 v1.8

---

## Issues

### SYN-01: AUTO_INCREMENT 支持

**优先级**: P0

**描述**: 实现 AUTO_INCREMENT 自增主键

**Acceptance Criteria**:
- [ ] `CREATE TABLE t (id INT AUTO_INCREMENT PRIMARY KEY)` 正确解析
- [ ] INSERT 时自动生成自增 ID
- [ ] `LAST_INSERT_ID()` 函数正确

---

### SYN-02: LIMIT offset, count

**优先级**: P0

**描述**: 支持 MySQL 的 LIMIT offset, count 语法

**Acceptance Criteria**:
- [ ] `SELECT * FROM t LIMIT 10, 20` 正确执行
- [ ] 与 `LIMIT 20 OFFSET 10` 等价

---

### SYN-03: SHOW TABLES / SHOW COLUMNS

**优先级**: P0

**描述**: 实现 MySQL 风格的 SHOW 命令

**Acceptance Criteria**:
- [ ] `SHOW TABLES` 输出所有表名
- [ ] `SHOW COLUMNS FROM t` 输出列信息
- [ ] `SHOW INDEX FROM t` 输出索引信息

---

### SYN-04: DESCRIBE table_name

**优先级**: P0

**描述**: 实现 DESCRIBE 命令

**Acceptance Criteria**:
- [ ] `DESCRIBE orders` 输出表结构
- [ ] 与 `SHOW COLUMNS FROM` 等价

---

### SYN-05: 常用函数

**优先级**: P1

**描述**: 实现常用 MySQL 函数

**Acceptance Criteria**:
- [ ] `NOW()` 返回当前时间
- [ ] `COUNT(*)` 正确计数
- [ ] `DATE_FORMAT(date, format)` 格式化日期

---

## 实现步骤

1. **Parser 扩展**
   - 添加 AUTO_INCREMENT 关键字
   - 添加 SHOW/DESCRIBE 语法

2. **Planner 扩展**
   - 添加自增 ID 生成逻辑
   - 添加 SHOW 计划的生成

3. **Executor 扩展**
   - 实现 SHOW 命令执行器
   - 实现自增计数器

---

## 关键文件

| 文件 | 用途 |
|------|------|
| `crates/parser/src/parser.rs` | 语法解析 |
| `crates/planner/src/planner.rs` | 计划生成 |
| `crates/executor/src/executor.rs` | 命令执行 |

---

**关联 Issue**: SYN-01, SYN-02, SYN-03, SYN-04, SYN-05
