# SQLRustGo v2.0 架构设计文档

**版本**: v2.0
**代号**: Vector Engine + Cascades
**发布日期**: 2026-03-26

---

## 一、架构概述

v2.0 是 SQLRustGo 的**向量化引擎版本**，引入 Cascades 优化器和向量化执行。

### 核心特性

- **向量化执行**: SIMD 加速
- **Cascades 优化器**: 基于成本的优化
- **火山模型 + 向量化**: 混合执行
- **存储引擎**: Buffer Pool + B+Tree

---

## 二、系统架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                        SQLRustGo v2.0                       │
├─────────────────────────────────────────────────────────────────┤
│  Client Layer                                                 │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐                │
│  │  CLI/Repl │  │ HTTP API │  │ MySQL协议 │                │
│  └────────────┘  └────────────┘  └────────────┘                │
├─────────────────────────────────────────────────────────────────┤
│  Query Processing Layer                                       │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              SQL Parser                               │  │
│  └─────────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              Query Planner                          │  │
│  └─────────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              Cascades Optimizer (CBO)               │  │
│  │  - Rule-based transformations                      │  │
│  │  - Cost-based join reordering              │  │
│  │  - Index selection                   │  │
│  └─────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Execution Layer                                              │
│  ┌────────────────────┐  ┌────────────────────┐              │
│  │  Volcano Executor │  │  Vectorized Exec  │              │
│  │  - Iterator model │  │  - SIMD (AVX2)   │              │
│  └─────────────────���──┘  └────────────────────┘              │
├─────────────────────────────────────────────────────────────────┤
│  Storage Layer                                                 │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              Buffer Pool                          │  │
│  │  - LRU/CLOCK replacement              │  │
│  │  - Page management                 │  │
│  └─────────────────────────────────────────────────────────┘  │
│  ┌────────────────┐  ┌────────────────┐                       │
│  │ B+Tree Index │  │  Hash Index  │                       │
│  └────────────────┘  └────────────────┘                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、核心模块设计

### 3.1 Cascades 优化器

#### What (是什么)
Cascades 优化器实现基于成本的查询优化，通过枚举物理计划并选择最低成本计划。

#### Why (为什么)
选择最优执行计划，提高查询性能。

#### How (如何实现)

```
┌────────────────────────────────────────────────┐
│         Cascades Optimizer                       │
├────────────────────────────────────────────────┤
│  Rule Explorer                             │
│  - 逻辑 → 物理变换                       │
│  - 规则匹配                            │
├────────────────────────────────────────────────┤
│  Cost Model                              │
│  - I/O 成本                            │
│  - CPU 成本                             │
│  - 网络成本                             │
├────────────────────────────────────────────────┤
│  Plan Enumerator                        │
│  - 深度优先搜索                        │
│  - 上界剪枝                            │
│  - 最佳计划选择                       │
└────────────────────────────────────────────────┘
```

##### 成本模型

```rust
pub struct Cost {
    pub io_cost: f64,
    pub cpu_cost: f64,
    pub total_cost: f64,
}

impl Cost {
    pub fn calculate(&self, plan: &PhysicalPlan) -> Cost {
        let io = self.estimate_io(plan);
        let cpu = self.estimate_cpu(plan);
        Cost {
            io_cost: io,
            cpu_cost: cpu,
            total_cost: io * self.io_weight + cpu * self.cpu_weight,
        }
    }
}
```

### 3.2 向量化执行

#### What (是什么)
向量化执行使用 SIMD 指令一次处理多条数据。

#### Why (为什么)
提高吞吐量，利用 CPU 的并行能力。

#### How (如何实现)

```
Data Batch (1024 rows)
    │
    ▼ SIMD Processing
┌──────────────────────���──────────────┐
│     AVX-2 Registers                  │
│  ┌────┬────┬────┬────┐           │
│  │v0  │v1  │v2  │v3  │ ...       │
│  └────┴────┴────┴────┘           │
│  每次处理 4 个 32-bit float        │
└─────────────────────────────────────┘
```

##### SIMD 示例

```rust
#[cfg(target_arch = "x86_64")]
pub fn vectorized_sum(values: &[f32]) -> f32 {
    unsafe {
        let mut sum = _mm256_setzero_ps();
        for chunk in values.chunks(8) {
            let va = _mm256_loadu_ps(chunk.as_ptr());
            sum = _mm256_add_ps(sum, va);
        }
        // Horizontal sum
        let high = _mm256_extractf128_ps(sum, 1);
        let low = _mm256_castps256_ps128(sum);
        _mm_add_ss(_mm_add_ss(low, high))
    }
}
```

### 3.3 存储引擎

#### What (是什么)
Buffer Pool 管理页面，支持 LRU/CLOCK 淘汰策略。

#### Why (为什么)
减少磁盘 I/O，提高访问效率。

#### How (如何实现)

```
┌─────────────────────────────────────┐
│         Buffer Pool                  │
├─────────────────────────────────────┤
│  ┌─────────────────────────────┐ │
│  │     Page Table              │ │
│  │  page_id → Frame mapping │ │
│  └─────────────────────────────┘ │
│  ┌─────────────────────────────┐ │
│  │     LRU / CLOCK Lists       │ │
│  │  - Hot list                  │ │
│  │  - Cold list                 │ │
│  └─────────────────────────────┘ │
│  ┌─────────────────────────────┐ │
│  │     Replacer               │ │
│  │  - Pin / Unpin              │ │
│  │  - Select victim            │ │
│  └─────────────────────────────┘ │
└─────────────────────────────────────┘
```

---

## 四、Crate 结构

### 核心 Crate

| Crate | 描述 | 版本 |
|------|------|------|
| sqlrustgo-parser | SQL 解析 | 1.x |
| sqlrustgo-planner | 查询规划 | 1.x |
| sqlrustgo-optimizer | Cascades 优化器 | 2.0 新增 |
| sqlrustgo-executor | 执行引擎 | 1.x |
| sqlrustgo-storage | 存储引擎 | 1.x |
| sqlrustgo-types | 数据类型 | 1.x |

### 依赖关系

```
parser → planner → optimizer → executor → storage
                  ↓
               optimizer → storage (统计信息)
```

---

## 五、设计原则

### 5.1 优化器原则

| 算法 | 复杂度 | 说明 |
|------|--------|------|
| 规则优化 | O(n) | 谓词下推、列裁剪 |
| 连接重排 | O(n!) | 使用动态规划 |
| 索引选择 | O(n) | 基于成本选择 |

### 5.2 执行器原则

| 优化 | 效果 |
|------|------|
| SIMD | 4x 加速 (AVX2) |
| 向量化 | 减少函数调用 |
| 批处理 | 增加吞吐量 |

---

## 六、版本演进

### v1.x → v2.0

| 维度 | v1.x | v2.0 |
|------|------|------|
| 优化器 | RBO | Cascades |
| 执行 | Volcano | 向量化 |
| 存储 | 基础 | Buffer Pool |
| 索引 | B+Tree | B+Tree + Hash |

---

## 七、快速导航

| 模块 | 文档 |
|------|------|
| Cascades 设计 | [architecture/CASCADES_DESIGN.md](./architecture/CASCADES_DESIGN.md) |
| 用户手册 | [../USER_MANUAL.md](../USER_MANUAL.md) |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-03-26*