# 派生表（Inline View）执行支持设计

## 概述

实现 SQLRustGo 执行器对派生表（Inline View）的支持，使 Q13、Q22 等使用内联视图的 TPC-H 查询能够完整执行。

## 问题分析

当前执行器处理 Q13 时：
1. 解析器正确解析了 `FROM (SELECT ...) AS c_orders` 到 `derived_tables` 字段
2. 但执行器**完全忽略了** `derived_tables` 字段
3. 执行器直接尝试扫描空表名，导致失败

## 解决方案：物化派生表

### 执行流程

```
1. ExecutionEngine::execute(SELECT with derived_tables)
           │
           ▼
2. 检测 derived_tables 是否为空
           │
           ▼
3. 物化派生表
   for each derived_table:
     a. 创建临时表 __derived_N
     b. 递归执行子查询
     c. 将结果写入临时表
           │
           ▼
4. 替换派生表引用
   - 在 select.tables 中用临时表名替换别名
   - 例如：c_orders -> __derived_0
           │
           ▼
5. 正常执行主查询
           │
           ▼
6. 清理临时表（查询结束后）
```

### 数据结构

```rust
// 临时表映射
struct TempTableMapping {
    alias: String,        // 派生表别名，如 "c_orders"
    temp_name: String,    // 临时表名，如 "__derived_0"
    schema: Vec<ColumnDefinition>,
}
```

## 实现步骤

### Step 1: 修改 StorageEngine trait
- 添加 `create_temp_table` 方法
- 添加 `drop_temp_table` 方法

### Step 2: 修改 ExecutionEngine
- 检测 derived_tables
- 递归执行子查询
- 物化结果到临时表
- 替换表名引用
- 执行主查询
- 清理临时表

### Step 3: 测试验证
- Q13 端到端测试
- Q22 端到端测试

## 文件变更

| 文件 | 修改内容 |
|------|---------|
| `src/lib.rs` | 修改 Statement::Select 处理逻辑 |
| `crates/storage/src/engine.rs` | StorageEngine trait 添加临时表方法 |

## 错误处理

- 子查询执行失败：返回错误
- 临时表创建失败：返回错误
- 临时表清理失败：记录日志，不阻塞执行

## 性能考虑

- 临时表使用内存存储
- 查询结束后立即清理
- 大结果集派生表可能内存压力大（后续优化）
