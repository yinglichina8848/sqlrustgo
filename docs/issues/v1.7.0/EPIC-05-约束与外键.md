# [Epic-05] 约束与外键

## 概述

实现 FOREIGN KEY 约束和约束检查系统，是数据库设计实验的基础。

**优先级**: P0
**来源**: 原 v1.8

---

## Issues

### FK-01: FOREIGN KEY 约束实现

**优先级**: P0

**描述**: 实现外键约束的解析和存储

**Acceptance Criteria**:
- [ ] `FOREIGN KEY (user_id) REFERENCES users(id)` 正确解析
- [ ] 外键定义存储在 Catalog
- [ ] `SHOW CREATE TABLE` 显示外键信息

### FK-02: 约束检查

**优先级**: P0

**描述**: 实现 INSERT/UPDATE/DELETE 时的外键约束检查

**Acceptance Criteria**:
- [ ] 插入子表时验证父表存在
- [ ] 更新/删除时正确处理级联
- [ ] 违反约束时返回 MySQL 风格错误

---

## 实现步骤

1. **Parser 扩展**
   - 添加 FOREIGN KEY 语法解析
   - 添加 REFERENCES 子句解析

2. **Catalog 扩展**
   - 在 TableSchema 中添加外键信息
   - 实现外键元数据存储

3. **Executor 扩展**
   - 实现外键约束检查算子
   - 处理 ON DELETE/UPDATE 级联

---

## 关键文件

| 文件 | 用途 |
|------|------|
| `crates/parser/src/parser.rs` | 外键语法解析 |
| `crates/catalog/src/schema.rs` | 外键元数据 |
| `crates/executor/src/executor.rs` | 外键检查算子 |

---

**关联 Issue**: FK-01, FK-02
