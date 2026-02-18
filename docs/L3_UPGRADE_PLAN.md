# SQLRustGo L3 升级详细执行计划

> 版本：v1.0
> 日期：2026-02-18
> 目标：从 工程化级（L2）→ 产品级（L3）
> 周期：3-6 个月

---

## 一、L3 必须达成的 6 个核心指标

| 维度 | 目标 | 当前状态 |
|:-----|:-----|:---------|
| **架构** | 模块边界清晰，无循环依赖 | ⚠️ 待评估 |
| **API** | 外部接口冻结策略 | ❌ 未建立 |
| **稳定性** | 单测覆盖率 ≥ 80% | ⚠️ ~76% |
| **性能** | 建立 Benchmark 框架 | ❌ 未建立 |
| **文档** | 使用者文档 + 架构文档 | ⚠️ 部分 |
| **兼容性** | 版本升级策略 | ❌ 未建立 |

---

## 二、阶段 1：架构稳定化（第 1 个月）

### 1.1 输出模块依赖图

**目标**：
- 明确 parser → planner → executor → storage 单向依赖
- 禁止反向调用

**工具建议**：
```bash
cargo tree
cargo modules
```

**输出物**：
- 模块依赖图文档
- 循环依赖报告

### 1.2 明确核心模块边界

**建议目录结构**：
```
src/
├── core/       # 核心类型和 trait
├── parser/     # SQL 解析器
├── planner/    # 查询规划器
├── executor/   # 执行引擎
├── storage/    # 存储引擎
├── api/        # 公共 API
├── tests/      # 集成测试
└── bench/      # 性能基准
```

**原则**：
- parser 不知道 executor
- executor 不知道 parser
- planner 不依赖 storage 具体实现（使用 trait）

---

## 三、阶段 2：接口稳定（第 2 个月）

### 3.1 冻结公共 API

**定义**：
- 对外 API 必须集中在 `/api`
- 内部模块禁止直接暴露

**建立**：
```rust
pub mod api;

// api/mod.rs
pub use crate::executor::ExecutionEngine;
pub use crate::storage::Storage;
pub use crate::types::*;
```

**文档**：
- README 中写明公共 API 稳定性策略

### 3.2 插件化接口设计

**为未来扩展预留**：
```rust
pub trait StorageEngine {
    fn create_table(&mut self, name: &str, schema: Schema) -> Result<()>;
    fn drop_table(&mut self, name: &str) -> Result<()>;
    fn insert(&mut self, table: &str, row: Row) -> Result<()>;
    fn scan(&self, table: &str) -> Result<Vec<Row>>;
}

pub trait Optimizer {
    fn optimize(&self, plan: LogicalPlan) -> LogicalPlan;
}

pub trait ExecutionStrategy {
    fn execute(&self, plan: PhysicalPlan) -> Result<ResultSet>;
}
```

---

## 四、阶段 3：测试与性能（第 3 个月）

### 4.1 引入 Benchmark 框架

**使用**：criterion

**建立**：
```
benches/
├── parser.rs
├── executor.rs
└── storage.rs
```

**定义 KPI**：
| 指标 | 说明 |
|:-----|:-----|
| 查询延迟 | SELECT 执行时间 |
| 内存使用 | 峰值内存 |
| 并发性能 | QPS |

### 4.2 测试覆盖率

**目标**：
```bash
cargo tarpaulin
```

**目标覆盖率**：≥ 80%

---

## 五、阶段 4：产品化标准（第 4-6 个月）

### 5.1 API 兼容策略

**定义**：
- minor 版本不破坏 API
- major 版本允许 breaking change

**写入**：`API_COMPATIBILITY.md`

### 5.2 文档站点

**推荐**：mdBook 或 Docusaurus

**文档必须包含**：
- 架构图
- 执行流程图
- 示例代码

---

## 六、下一步可执行行动清单

### 未来 30 天建议做：

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        30 天行动计划                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   第 1 周：                                                                 │
│   ├── [ ] 输出模块依赖图                                                    │
│   └── [ ] 识别循环依赖                                                      │
│                                                                              │
│   第 2 周：                                                                 │
│   ├── [ ] 抽象 storage trait                                                │
│   └── [ ] 引入 benchmark                                                    │
│                                                                              │
│   第 3 周：                                                                 │
│   ├── [ ] 提升测试覆盖率到 80%                                              │
│   └── [ ] 输出架构图文档                                                    │
│                                                                              │
│   第 4 周：                                                                 │
│   ├── [ ] 冻结公共 API                                                      │
│   └── [ ] 建立 API 兼容策略                                                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 七、核心升级思想

```
从：功能驱动开发
转向：架构驱动演进
```

---

*本文档由 TRAE (GLM-5.0) 创建*
