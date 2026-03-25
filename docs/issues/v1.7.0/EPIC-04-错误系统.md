# [Epic-04] 错误系统

## 概述

实现 MySQL 风格的错误信息系统，为 v1.8 教学兼容打基础。

**资源占比**: 10%
**优先级**: P2

---

## Issues

### [ERR-01] Unknown column 错误

**优先级**: P1
**工作量**: 30 行

**描述**: 实现 MySQL 风格的 Unknown column 错误

**Acceptance Criteria**:
- [x] 引用不存在的列时返回错误
- [x] 错误信息包含列名和表名

**错误格式**:
```
ERROR 1054 (42S22): Unknown column 'xxx' in 'field list'
```

**实现**:
- 在 `crates/planner/src/lib.rs` 添加 `validate_columns` 方法
- 在 `crates/executor/src/local_executor.rs` 的 `execute_projection` 和 `execute_filter` 中调用验证

---

### [ERR-02] Table not found 错误

**优先级**: P1
**工作量**: 30 行

**描述**: 实现 MySQL 风格的表不存在错误

**Acceptance Criteria**:
- [x] 引用不存在的表时返回错误
- [x] 错误信息包含表名

**错误格式**:
```
ERROR 1146 (42S02): Table 'xxx' doesn't exist
```

**实现**:
- 在 `crates/storage/src/engine.rs` 的 `get_table_info` 中实现

---

### [ERR-03] Duplicate key 错误

**优先级**: P2
**工作量**: 40 行

**描述**: 实现 MySQL 风格的重复键错误

**Acceptance Criteria**:
- [x] 违反 UNIQUE 约束时返回错误
- [x] 错误信息包含键值

**错误格式**:
```
ERROR 1062 (23000): Duplicate entry 'xxx' for key 'yyy'
```

**实现**:
- 在 `crates/storage/src/engine.rs` 的 `ColumnDefinition` 添加 `is_unique` 字段
- 在 `MemoryStorage::insert` 中检查 UNIQUE 约束

---

## 实现步骤

1. **错误类型定义**
   - 在 `crates/types/src/error.rs` 添加错误变体
   - 实现 `SqlError` trait

2. **错误码映射**
   - 定义 MySQL 风格错误码
   - 实现错误格式化

3. **错误抛出点**
   - 在 Parser/Planner/Executor 中添加检查
   - 抛出对应错误

---

## 关键文件

| 文件 | 用途 |
|------|------|
| `crates/types/src/error.rs` | 错误类型定义 |
| `crates/executor/src/executor.rs` | 错误抛出 |
| `crates/planner/src/planner.rs` | 语义检查 |

---

## 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 错误信息本地化 | 低 | 初期仅支持英文 |
| 错误码冲突 | 低 | 复用现有错误系统 |

---

**关联 Issue**: ERR-01, ERR-02, ERR-03
**总工作量**: ~100 行