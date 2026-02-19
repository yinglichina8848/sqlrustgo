# SQLRustGo 2.0 规划目录

> 版本：v1.1
> 日期：2026-02-19
> 目标：从"项目代码"升级为"数据库内核架构"

---

## 一、目录结构

```
docs/v2.0/
├── README.md                      # 本文档
├── WHITEPAPER.md                  # 2.0 白皮书
├── SQLRUSTGO_2_0_ROADMAP.md       # 2.0 总体路线图
├── TRAE-2.0对话记录.md            # 2.0 规划对话记录
│
├── 成熟度评估/
│   ├── MATURITY_MODEL.md          # L0-L4 成熟度模型
│   ├── MATURITY_SCORECARD.md      # 架构成熟度打分表
│   ├── GROWTH_ROADMAP.md          # 成长路线图
│   └── ARCHITECTURE_GOVERNANCE.md # 架构治理蓝图
│
├── 架构设计/
│   ├── PLUGIN_ARCHITECTURE.md     # 插件化执行架构
│   ├── PLAN_DATA_STRUCTURES.md    # LogicalPlan/PhysicalPlan 设计
│   ├── PLUGIN_EXECUTOR_DESIGN.md  # 插件化执行引擎原型
│   ├── PLUGIN_REGISTRY.md         # 插件注册机制
│   └── PLUGIN_SYSTEM_CODE.md      # 插件系统真实代码结构
│
├── 性能优化/
│   ├── VECTORIZED_EXECUTION.md    # 向量化执行模型
│   ├── VECTORIZED_EXPRESSION.md   # 向量化表达式执行完整设计
│   ├── PERFORMANCE_ANALYSIS_50K.md# 5万行压力测试分析
│   ├── CBO_DESIGN.md              # 成本优化器设计
│   └── CBO_ALGORITHM.md           # CBO 详细算法（含 Join DP 公式）
│
└── 重构计划/
    ├── L3_UPGRADE_PLAN.md         # L3 升级计划
    ├── ARCHITECTURE_REFACTORING.md# 模块边界重构
    ├── TECH_DEBT_ANALYSIS.md      # 技术债分析
    ├── ARCHITECTURE_RISK_MODEL.md # 架构风险评分
    └── REFACTORING_PRIORITY.md    # 重构优先级
```

---

## 二、文档分类

### 2.1 白皮书与路线图

| 文档 | 说明 |
|:-----|:-----|
| [WHITEPAPER.md](WHITEPAPER.md) | SQLRustGo 2.0 白皮书 |
| [SQLRUSTGO_2_0_ROADMAP.md](SQLRUSTGO_2_0_ROADMAP.md) | 2.0 总体路线图 |
| [TRAE-2.0对话记录.md](TRAE-2.0对话记录.md) | 2.0 规划对话记录 |

### 2.2 成熟度评估

| 文档 | 说明 |
|:-----|:-----|
| [MATURITY_MODEL.md](成熟度评估/MATURITY_MODEL.md) | L0-L4 五级成熟度定义 |
| [MATURITY_SCORECARD.md](成熟度评估/MATURITY_SCORECARD.md) | 架构成熟度打分表（L1-L5） |
| [GROWTH_ROADMAP.md](成熟度评估/GROWTH_ROADMAP.md) | 个人项目 → 企业级产品路线图 |
| [ARCHITECTURE_GOVERNANCE.md](成熟度评估/ARCHITECTURE_GOVERNANCE.md) | 分支保护、权限控制、长期演进 |

### 2.3 架构设计

| 文档 | 说明 |
|:-----|:-----|
| [PLUGIN_ARCHITECTURE.md](架构设计/PLUGIN_ARCHITECTURE.md) | 完整插件化执行架构图 |
| [PLAN_DATA_STRUCTURES.md](架构设计/PLAN_DATA_STRUCTURES.md) | LogicalPlan / PhysicalPlan 数据结构 |
| [PLUGIN_EXECUTOR_DESIGN.md](架构设计/PLUGIN_EXECUTOR_DESIGN.md) | 可插拔执行引擎原型 |
| [PLUGIN_REGISTRY.md](架构设计/PLUGIN_REGISTRY.md) | 插件注册中心实现 |
| [PLUGIN_SYSTEM_CODE.md](架构设计/PLUGIN_SYSTEM_CODE.md) | 插件系统真实代码结构 |

### 2.4 性能优化

| 文档 | 说明 |
|:-----|:-----|
| [VECTORIZED_EXECUTION.md](性能优化/VECTORIZED_EXECUTION.md) | 向量化执行模型设计 |
| [VECTORIZED_EXPRESSION.md](性能优化/VECTORIZED_EXPRESSION.md) | 向量化表达式执行完整设计 |
| [PERFORMANCE_ANALYSIS_50K.md](性能优化/PERFORMANCE_ANALYSIS_50K.md) | 5万行规模压力测试分析 |
| [CBO_DESIGN.md](性能优化/CBO_DESIGN.md) | 成本优化器（CBO）设计 |
| [CBO_ALGORITHM.md](性能优化/CBO_ALGORITHM.md) | CBO 详细算法（含 Join DP 公式） |

### 2.5 重构计划

| 文档 | 说明 |
|:-----|:-----|
| [L3_UPGRADE_PLAN.md](重构计划/L3_UPGRADE_PLAN.md) | L2 → L3 升级详细计划 |
| [ARCHITECTURE_REFACTORING.md](重构计划/ARCHITECTURE_REFACTORING.md) | 模块边界重构建议 |
| [TECH_DEBT_ANALYSIS.md](重构计划/TECH_DEBT_ANALYSIS.md) | 技术债风险分析报告 |
| [ARCHITECTURE_RISK_MODEL.md](重构计划/ARCHITECTURE_RISK_MODEL.md) | 架构风险评分模型 |
| [REFACTORING_PRIORITY.md](重构计划/REFACTORING_PRIORITY.md) | 模块重构优先级排序 |

---

## 三、当前状态

### 3.1 成熟度评估

```
当前等级：L2 工程化级
当前风险：3.15（中度风险）
目标等级：L3 产品级
目标风险：1.35（轻度风险）
```

### 3.2 关键指标

| 指标 | 当前 | 目标 |
|:-----|:-----|:-----|
| 测试覆盖率 | ~76% | 85% |
| 架构风险 | 3.15 | 1.35 |
| 模块耦合度 | 中 | 低 |
| API 稳定性 | 不稳定 | 冻结 |

---

## 四、升级路径

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          升级路径                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Phase 1（1-2个月）                                                        │
│   ├── LogicalPlan 重构                                                      │
│   ├── PhysicalPlan trait 化                                                 │
│   ├── Executor 插件化                                                       │
│   └── HashJoin 实现                                                         │
│                                                                              │
│   Phase 2（2-3个月）                                                        │
│   ├── 向量化执行                                                            │
│   ├── 批处理表达式                                                          │
│   ├── 基础统计信息                                                          │
│   └── 简化 CBO                                                              │
│                                                                              │
│   Phase 3（3-6个月）                                                        │
│   ├── 完整 CBO                                                              │
│   ├── Join reorder                                                          │
│   ├── Memory pool                                                           │
│   └── Spill to disk                                                         │
│                                                                              │
│   Phase 4（6-12个月）                                                       │
│   ├── 分布式执行                                                            │
│   ├── 事务支持                                                              │
│   └── 高可用                                                                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 五、阅读顺序

推荐阅读顺序：

1. **了解现状**：[成熟度评估/MATURITY_MODEL.md](成熟度评估/MATURITY_MODEL.md) → [成熟度评估/MATURITY_SCORECARD.md](成熟度评估/MATURITY_SCORECARD.md) → [重构计划/ARCHITECTURE_RISK_MODEL.md](重构计划/ARCHITECTURE_RISK_MODEL.md)
2. **理解目标**：[WHITEPAPER.md](WHITEPAPER.md) → [SQLRUSTGO_2_0_ROADMAP.md](SQLRUSTGO_2_0_ROADMAP.md) → [成熟度评估/GROWTH_ROADMAP.md](成熟度评估/GROWTH_ROADMAP.md)
3. **学习架构**：[架构设计/PLUGIN_ARCHITECTURE.md](架构设计/PLUGIN_ARCHITECTURE.md) → [架构设计/PLAN_DATA_STRUCTURES.md](架构设计/PLAN_DATA_STRUCTURES.md) → [架构设计/PLUGIN_SYSTEM_CODE.md](架构设计/PLUGIN_SYSTEM_CODE.md)
4. **规划重构**：[重构计划/L3_UPGRADE_PLAN.md](重构计划/L3_UPGRADE_PLAN.md) → [重构计划/REFACTORING_PRIORITY.md](重构计划/REFACTORING_PRIORITY.md)
5. **深入优化**：[性能优化/VECTORIZED_EXPRESSION.md](性能优化/VECTORIZED_EXPRESSION.md) → [性能优化/CBO_ALGORITHM.md](性能优化/CBO_ALGORITHM.md)

---

## 六、对标分析

| 模块 | sqlrustgo | DataFusion |
|:-----|:----------|:-----------|
| Parser | ✅ | ✅ |
| LogicalPlan | 部分 | ✅ |
| CBO | ❌ | 部分 |
| 向量化 | ❌ | ✅ |
| 插件 | 基础 | ✅ |
| 分布式 | ❌ | ✅ |

---

## 七、关键原则

```
1. 模块独立 - 每个模块可独立开发、测试
2. 接口优先 - 先定义接口，再实现
3. 依赖反转 - 高层不依赖低层，都依赖抽象
4. 避免循环依赖 - 使用依赖注入解耦
5. 每个模块可单测 - 模块边界清晰
```

---

*本文档由 TRAE (GLM-5.0) 创建*
