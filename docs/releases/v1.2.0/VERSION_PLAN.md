# SQLRustGo v1.2.0 版本计划

> 版本：v1.2.0
> 制定日期：2026-03-04
> 制定人：yinglichina8848
> 目标：性能优化，支持 100万行级数据处理

---

## 一、版本概述

### 1.1 版本目标

| 项目 | 值 |
|------|-----|
| **版本号** | v1.2.0 |
| **目标成熟度** | L3+ 产品级 |
| **核心目标** | 性能优化，支持 100万行级数据 |
| **预计时间** | v1.1.0 GA 后 1 月 |

### 1.2 前置依赖

- ✅ #115 (v1.1.0 发布门禁) 已完成
- ✅ v1.1.0 已发布到 main

---

## 二、开发轨道

### 轨道 A: 内核性能优化

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          轨道 A: 内核性能优化                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Week 1-2: 向量化基础                                                      │
│   ├── V-001: RecordBatch 结构定义                                           │
│   ├── V-002: ColumnarArray trait 实现                                       │
│   └── V-003: 向量化表达式                                                   │
│                                                                              │
│   Week 3-4: 向量化执行器                                                    │
│   ├── V-004: 执行器重构支持批量处理                                         │
│   ├── V-005: 向量化 Filter 实现                                             │
│   ├── V-006: 向量化 Projection 实现                                         │
│   └── V-007: 向量化聚合实现                                                 │
│                                                                              │
│   Week 5-6: 统计信息                                                        │
│   ├── S-001: TableStats 结构设计                                            │
│   ├── S-002: ColumnStats 结构设计                                           │
│   ├── S-003: 统计信息收集器                                                 │
│   ├── S-004: 统计信息持久化                                                 │
│   ├── S-005: ANALYZE 命令实现                                               │
│   └── S-006: 统计信息查询接口                                               │
│                                                                              │
│   Week 7-8: 简化 CBO                                                        │
│   ├── C-001: CostModel 结构设计                                             │
│   ├── C-002: 基础成本估算                                                   │
│   ├── C-003: Join 选择优化                                                  │
│   ├── C-004: 索引选择优化                                                   │
│   ├── C-005: 优化器集成                                                     │
│   └── C-006: 成本估算测试                                                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 轨道 B: 网络层增强 (v1.2.1)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          轨道 B: 网络层增强                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Week 1-2: 异步服务器 (已完成 v1.1.0)                                       │
│   ├── N-001 ~ N-007: 基础 C/S 架构 ✅                                        │
│   └── 目标：基础 Client-Server ✅                                            │
│                                                                              │
│   Week 3-4: 功能完善                                                        │
│   ├── N-011: 异步服务器实现 ✅                                               │
│   ├── N-012: 连接池实现 ✅                                                   │
│   ├── N-013: 会话管理完善                                                   │
│   ├── N-014: 交互模式 (REPL)                                                │
│   └── N-015: 配置文件支持                                                   │
│                                                                              │
│   Week 5-6: 生产就绪                                                        │
│   ├── N-019: 认证机制完善                                                   │
│   ├── N-020: SSL/TLS 支持                                                   │
│   ├── N-021: 性能测试和优化                                                 │
│   └── N-022: 文档编写                                                       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 三、任务分配

### 3.1 任务矩阵

| ID | 任务 | 负责人 | 预估时间 | 优先级 | 依赖 |
|----|------|--------|----------|--------|------|
| **向量化** |||||||
| V-001 | RecordBatch 结构定义 | openheart | 4h | P0 | - |
| V-002 | ColumnarArray trait 实现 | openheart | 6h | P0 | V-001 |
| V-003 | 向量化表达式实现 | openheart | 8h | P0 | V-002 |
| V-004 | 执行器重构支持批量处理 | heartopen | 12h | P0 | V-003 |
| V-005 | 向量化 Filter 实现 | heartopen | 6h | P1 | V-004 |
| V-006 | 向量化 Projection 实现 | heartopen | 6h | P1 | V-004 |
| V-007 | 向量化聚合实现 | heartopen | 8h | P1 | V-004 |
| **统计信息** |||||||
| S-001 | TableStats 结构设计 | openheart | 4h | P0 | - |
| S-002 | ColumnStats 结构设计 | openheart | 4h | P0 | S-001 |
| S-003 | 统计信息收集器实现 | openheart | 8h | P1 | S-002 |
| S-004 | 统计信息持久化 | openheart | 6h | P1 | S-003 |
| S-005 | ANALYZE 命令实现 | heartopen | 4h | P2 | S-004 |
| S-006 | 统计信息查询接口 | heartopen | 4h | P2 | S-004 |
| **CBO** |||||||
| C-001 | CostModel 结构设计 | openheart | 4h | P0 | S-006 |
| C-002 | 基础成本估算实现 | openheart | 8h | P1 | C-001 |
| C-003 | Join 选择优化 | heartopen | 8h | P1 | C-002 |
| C-004 | 索引选择优化 | heartopen | 6h | P1 | C-002 |
| C-005 | 优化器集成 | heartopen | 6h | P1 | C-003, C-004 |
| C-006 | 成本估算测试 | maintainer | 4h | P1 | C-005 |
| **网络增强** |||||||
| N-013 | 会话管理完善 | heartopen | 4h | P2 | - |
| N-014 | 交互模式 (REPL) | heartopen | 6h | P2 | N-013 |
| N-015 | 配置文件支持 | openheart | 4h | P2 | - |
| N-019 | 认证机制完善 | heartopen | 4h | P2 | N-013 |
| N-020 | SSL/TLS 支持 | openheart | 6h | P3 | N-019 |
| **文档** |||||||
| D-001 | 性能测试报告 | maintainer | 4h | P1 | V-007 |
| D-002 | API 文档更新 | maintainer | 4h | P1 | - |
| D-003 | 升级指南 | maintainer | 4h | P2 | - |
| D-004 | Release Notes | yinglichina8848 | 2h | P0 | - |

### 3.2 负责人分工

| 负责人 | 角色 | 任务范围 |
|--------|------|----------|
| **openheart** | 架构开发 | 向量化基础、统计信息、CBO 设计 |
| **heartopen** | 功能开发 | 向量化执行器、网络增强 |
| **maintainer** | 审核 | 测试、文档、代码审核 |
| **yinglichina8848** | 调度 | 计划制定、发布控制 |

---

## 四、技术设计

### 4.1 RecordBatch 结构

```rust
pub struct RecordBatch {
    schema: Arc<Schema>,
    columns: Vec<ArrayRef>,
    row_count: usize,
}

pub type ArrayRef = Arc<dyn Array>;

pub trait Array: Send + Sync {
    fn data_type(&self) -> &DataType;
    fn len(&self) -> usize;
    fn is_null(&self, index: usize) -> bool;
    fn slice(&self, offset: usize, length: usize) -> ArrayRef;
}
```

### 4.2 统计信息结构

```rust
pub struct TableStats {
    pub row_count: usize,
    pub total_bytes: usize,
    pub column_stats: HashMap<String, ColumnStats>,
}

pub struct ColumnStats {
    pub distinct_count: usize,
    pub null_count: usize,
    pub min_value: Option<ScalarValue>,
    pub max_value: Option<ScalarValue>,
    pub avg_width: Option<f64>,
}
```

### 4.3 成本模型

```rust
pub struct CostModel {
    pub seq_scan_cost: f64,
    pub idx_scan_cost: f64,
    pub filter_cost: f64,
    pub join_cost: f64,
    pub aggregate_cost: f64,
}

impl CostModel {
    pub fn estimate(&self, plan: &LogicalPlan, stats: &TableStats) -> Cost {
        match plan {
            LogicalPlan::TableScan { .. } => {
                Cost::from(self.seq_scan_cost * stats.row_count as f64)
            }
            LogicalPlan::Filter { input, .. } => {
                let input_cost = self.estimate(input, stats);
                input_cost + Cost::from(self.filter_cost * stats.row_count as f64)
            }
            // ...
        }
    }
}
```

---

## 五、里程碑

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          v1.2.0 里程碑                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   v1.2.0-draft ─────────────────────────────────────────────────────────►   │
│   │                                                                          │
│   ├── Week 1-2: 向量化基础                                                  │
│   │   └── V-001 ~ V-003: RecordBatch + ColumnarArray                        │
│   │                                                                          │
│   ├── Week 3-4: 向量化执行器                                                │
│   │   └── V-004 ~ V-007: 执行器重构 + 向量化算子                             │
│   │                                                                          │
│   ├── Week 5-6: 统计信息                                                    │
│   │   └── S-001 ~ S-006: 统计信息收集与持久化                                │
│   │                                                                          │
│   ├── Week 7-8: 简化 CBO                                                    │
│   │   └── C-001 ~ C-006: 成本模型与优化器                                   │
│   │                                                                          │
│   └── Week 9-10: 测试与文档                                                 │
│       └── D-001 ~ D-004: 文档 + Release                                     │
│                                                                              │
│   v1.2.0 GA ─────────────────────────────────────────────────────────────►   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 六、验收标准

### 6.1 功能验收

| 验收项 | 标准 |
|--------|------|
| 向量化执行 | RecordBatch + 向量化算子完整实现 |
| 统计信息 | TableStats + ColumnStats 收集正确 |
| CBO | 基础成本优化生效 |
| 测试通过 | 所有测试通过 |

### 6.2 性能验收

| 指标 | 目标 |
|------|------|
| 100万行数据处理 | < 1s |
| 简单查询延迟 | < 100ms |
| 内存使用 | 合理范围 |

### 6.3 质量验收

| 指标 | 目标 |
|------|------|
| 测试覆盖率 | ≥ 90% |
| Clippy | 无警告 |
| 文档 | 完整 |

---

## 七、风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 向量化重构影响现有功能 | 高 | 中 | 分支开发 + 充分测试 |
| 性能目标未达成 | 中 | 低 | 基准测试 + 迭代优化 |
| CBO 实现复杂度超预期 | 中 | 中 | 简化初始版本 |
| 时间延期 | 中 | 中 | 优先级排序 + MVP |

---

## 八、发布计划

### 8.1 版本流程

```
v1.2.0-draft → v1.2.0-alpha → v1.2.0-beta → v1.2.0-rc → v1.2.0
```

### 8.2 时间表

| 版本 | 预计时间 | 说明 |
|------|----------|------|
| v1.2.0-draft | Week 2 | 向量化基础完成 |
| v1.2.0-alpha | Week 4 | 向量化执行器完成 |
| v1.2.0-beta | Week 6 | 统计信息完成 |
| v1.2.0-rc | Week 8 | CBO 完成 |
| v1.2.0 | Week 10 | 正式发布 |

---

## 九、关联 Issue

- 父 Issue: #88 (SQLRustGo 2.0 总体开发计划)
- 前置 Issue: #115 (v1.1.0 发布门禁)
- 子 Issue:
  - #103 Storage trait 化
  - #104 Optimizer 规则系统实现
  - #105 向量化执行实现
  - #107 技术债务清理

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-04 | 初始版本计划 |

---

*本文档由 yinglichina8848 制定*
