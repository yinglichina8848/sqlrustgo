# 架构模块边界重构建议

> 版本：v1.0
> 日期：2026-02-18

---

## 一、当前风险（常见问题）

在早期项目中通常会出现：

| 问题 | 说明 |
|:-----|:-----|
| parser 直接操作 storage | 绕过执行层 |
| executor 含有大量业务逻辑 | 职责不清 |
| planner 与 executor 强耦合 | 难以优化 |
| 全局状态滥用 | 难以测试 |

---

## 二、理想模块边界模型

```
        +--------+
        |  API   |  ← 公共接口层
        +--------+
            ↓
        +--------+
        | Parser |  ← SQL 解析
        +--------+
            ↓
        +--------+
        | Planner|  ← 查询规划
        +--------+
            ↓
        +--------+
        |Executor|  ← 执行引擎
        +--------+
            ↓
        +--------+
        |Storage |  ← 存储引擎
        +--------+
```

---

## 三、关键原则

### 3.1 依赖单向

**禁止**：
```
❌ storage → planner
❌ executor → parser
```

**正确**：
```
✅ parser → planner → executor → storage
```

### 3.2 通过 Trait 解耦

```rust
pub trait Storage {
    fn read(&self, key: &str) -> Result<Row>;
    fn write(&mut self, key: &str, value: Row) -> Result<()>;
}

// executor 只依赖 trait，不依赖具体实现
pub struct Executor<S: Storage> {
    storage: S,
}
```

### 3.3 消除全局变量

**禁止**：
```rust
❌ static mut GLOBAL_CONTEXT: Context;
```

**引入**：
```rust
✅ pub struct Context {
    // ...
}
```

### 3.4 引入中间表示（IR）

**planner 输出**：
```rust
pub struct LogicalPlan {
    // 逻辑执行计划
}
```

**executor 接收**：
```rust
pub struct PhysicalPlan {
    // 物理执行计划
}
```

**避免直接传 AST**，使用 IR 解耦。

---

## 四、模块职责定义

### 4.1 Parser 模块

**职责**：
- SQL 词法分析
- SQL 语法分析
- 生成 AST

**不负责**：
- 执行逻辑
- 存储操作

### 4.2 Planner 模块

**职责**：
- AST → LogicalPlan
- 查询优化
- 生成 PhysicalPlan

**不负责**：
- 具体执行
- 存储访问

### 4.3 Executor 模块

**职责**：
- 执行 PhysicalPlan
- 调用 Storage trait
- 返回结果

**不负责**：
- SQL 解析
- 查询优化

### 4.4 Storage 模块

**职责**：
- 数据持久化
- 索引管理
- 事务支持

**不负责**：
- SQL 逻辑
- 执行计划

---

## 五、重构步骤

### Step 1：绘制当前依赖图

```bash
cargo modules dependencies
```

### Step 2：识别问题依赖

- 循环依赖
- 反向依赖
- 跨层调用

### Step 3：定义 Trait 抽象

```rust
// core/traits.rs
pub trait Storage { ... }
pub trait Executor { ... }
pub trait Planner { ... }
```

### Step 4：逐步重构

1. 先重构最底层（storage）
2. 再重构中间层（executor, planner）
3. 最后重构顶层（parser, api）

---

## 六、验收标准

| 检查项 | 标准 |
|:-------|:-----|
| 循环依赖 | 0 个 |
| 反向依赖 | 0 个 |
| 跨层调用 | 0 个 |
| Trait 抽象 | 完成 |
| 单元测试 | 通过 |

---

*本文档由 TRAE (GLM-5.0) 创建*
