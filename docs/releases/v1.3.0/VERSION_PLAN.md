# SQLRustGo v1.3.0 版本计划

> 版本：v1.3.0
> 制定日期：2026-03-05
> 制定人：yinglichina8848
> 目标：内核能力提升，实现商用级可观测性

---

## 一、版本概述

### 1.1 版本目标

| 项目 | 值 |
|------|-----|
| **版本号** | v1.3.0 |
| **目标成熟度** | L4 企业级 |
| **核心目标** | 商用级内核，完整 CBO，可观测性 |
| **预计时间** | v1.2.0 GA 后 2 月 |

### 1.2 前置依赖

- ✅ #117 (v1.2.0 开发计划) 必须完成
- ✅ v1.2.0 统计信息系统 (S-001 ~ S-006) 提供基础设施

---

## 二、开发轨道

### 轨道 A: 内核能力提升

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          轨道 A: 内核能力提升                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Week 1-2: 插件系统                                                        │
│   ├── P-001: Plugin trait 定义                                              │
│   ├── P-002: 插件加载器实现                                                 │
│   ├── P-003: 存储插件接口                                                   │
│   ├── P-004: 执行器插件接口                                                 │
│   └── P-005: 插件生命周期管理                                               │
│                                                                              │
│   Week 3-4: CBO 完善                                                        │
│   ├── C-001: 成本模型完善                                                   │
│   ├── C-002: 统计信息集成                                                   │
│   ├── C-003: Join 顺序优化                                                  │
│   ├── C-004: 索引选择优化                                                   │
│   └── C-005: 代价估算测试                                                   │
│                                                                              │
│   Week 5-6: Join 算法演进                                                   │
│   ├── J-001: SortMergeJoin 实现                                             │
│   ├── J-002: NestedLoopJoin 优化                                            │
│   ├── J-003: Join 选择策略                                                  │
│   └── J-004: Join 性能测试                                                  │
│                                                                              │
│   Week 7-8: 事务增强                                                        │
│   ├── T-001: 事务隔离级别                                                   │
│   ├── T-002: MVCC 基础实现                                                  │
│   ├── T-003: 锁管理器                                                       │
│   └── T-004: 死锁检测                                                       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 轨道 B: 可观测性系统 (新增)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          轨道 B: 可观测性系统                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Week 1-2: 性能指标监控                                                    │
│   ├── M-001: Metrics trait 定义                                             │
│   ├── M-002: BufferPoolMetrics 实现                                         │
│   ├── M-003: ExecutorMetrics 实现                                           │
│   ├── M-004: NetworkMetrics 实现                                            │
│   └── M-005: 指标聚合器                                                     │
│                                                                              │
│   Week 3-4: 健康检查系统                                                    │
│   ├── H-001: HealthChecker 实现                                             │
│   ├── H-002: /health/live 端点                                              │
│   ├── H-003: /health/ready 端点                                             │
│   ├── H-004: /health 综合端点                                               │
│   └── H-005: 组件健康检查器                                                 │
│                                                                              │
│   Week 5-6: 指标暴露与集成                                                  │
│   ├── E-001: Prometheus 格式暴露                                            │
│   ├── E-002: /metrics 端点                                                  │
│   ├── E-003: Grafana Dashboard 模板                                         │
│   └── E-004: 告警规则示例                                                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 轨道 C: 企业级特性 (v1.3.1+)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          轨道 C: 企业级特性                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Week 1-2: 内存管理                                                        │
│   ├── MM-001: Memory Pool 实现                                              │
│   ├── MM-002: 内存配额管理                                                  │
│   └── MM-003: Spill to Disk                                                 │
│                                                                              │
│   Week 3-4: 安全增强                                                        │
│   ├── S-001: 认证机制完善                                                   │
│   ├── S-002: SSL/TLS 支持                                                   │
│   └── S-003: 权限控制                                                       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 三、任务分配

### 3.1 任务矩阵

| ID | 任务 | 负责人 | 预估时间 | 优先级 | 依赖 |
|----|------|--------|----------|--------|------|
| **插件系统** |||||||
| P-001 | Plugin trait 定义 | openheart | 4h | P0 | - |
| P-002 | 插件加载器实现 | openheart | 6h | P0 | P-001 |
| P-003 | 存储插件接口 | openheart | 8h | P1 | P-002 |
| P-004 | 执行器插件接口 | heartopen | 8h | P1 | P-002 |
| P-005 | 插件生命周期管理 | openheart | 6h | P1 | P-003, P-004 |
| **CBO 完善** |||||||
| C-001 | 成本模型完善 | openheart | 6h | P0 | v1.2.0 S-006 |
| C-002 | 统计信息集成 | openheart | 4h | P0 | C-001 |
| C-003 | Join 顺序优化 | heartopen | 8h | P1 | C-002 |
| C-004 | 索引选择优化 | heartopen | 6h | P1 | C-002 |
| C-005 | 代价估算测试 | maintainer | 4h | P1 | C-003, C-004 |
| **Join 算法** |||||||
| J-001 | SortMergeJoin 实现 | heartopen | 12h | P0 | - |
| J-002 | NestedLoopJoin 优化 | heartopen | 6h | P1 | J-001 |
| J-003 | Join 选择策略 | openheart | 4h | P1 | J-001, J-002 |
| J-004 | Join 性能测试 | maintainer | 4h | P1 | J-003 |
| **事务增强** |||||||
| T-001 | 事务隔离级别 | openheart | 8h | P0 | - |
| T-002 | MVCC 基础实现 | openheart | 16h | P0 | T-001 |
| T-003 | 锁管理器 | heartopen | 8h | P0 | T-001 |
| T-004 | 死锁检测 | heartopen | 6h | P1 | T-003 |
| **性能监控** |||||||
| M-001 | Metrics trait 定义 | openheart | 4h | P0 | - |
| M-002 | BufferPoolMetrics 实现 | heartopen | 4h | P0 | M-001 |
| M-003 | ExecutorMetrics 实现 | heartopen | 4h | P0 | M-001 |
| M-004 | NetworkMetrics 实现 | heartopen | 4h | P1 | M-001 |
| M-005 | 指标聚合器 | openheart | 4h | P1 | M-002, M-003, M-004 |
| **健康检查** |||||||
| H-001 | HealthChecker 实现 | heartopen | 4h | P0 | M-005 |
| H-002 | /health/live 端点 | heartopen | 2h | P0 | H-001 |
| H-003 | /health/ready 端点 | heartopen | 2h | P0 | H-001 |
| H-004 | /health 综合端点 | heartopen | 4h | P1 | H-002, H-003 |
| H-005 | 组件健康检查器 | heartopen | 4h | P1 | H-001 |
| **指标暴露** |||||||
| E-001 | Prometheus 格式暴露 | openheart | 4h | P1 | M-005 |
| E-002 | /metrics 端点 | heartopen | 2h | P1 | E-001 |
| E-003 | Grafana Dashboard 模板 | maintainer | 4h | P2 | E-002 |
| E-004 | 告警规则示例 | maintainer | 2h | P2 | E-002 |
| **文档** |||||||
| D-001 | 可观测性指南 | maintainer | 4h | P1 | E-002 |
| D-002 | API 文档更新 | maintainer | 4h | P1 | - |
| D-003 | 升级指南 | maintainer | 4h | P2 | - |
| D-004 | Release Notes | yinglichina8848 | 2h | P0 | - |

### 3.2 负责人分工

| 负责人 | 角色 | 任务范围 |
|--------|------|----------|
| **openheart** | 架构开发 | 插件系统、CBO、事务、Metrics 设计 |
| **heartopen** | 功能开发 | Join 算法、健康检查、指标实现 |
| **maintainer** | 审核 | 测试、文档、Dashboard、告警规则 |
| **yinglichina8848** | 调度 | 计划制定、发布控制 |

---

## 四、技术设计

### 4.1 Metrics 系统

```rust
pub trait Metrics: Send + Sync {
    fn name(&self) -> &str;
    fn collect(&self) -> Vec<MetricSample>;
    fn reset(&self);
}

pub struct MetricSample {
    pub name: String,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
    pub timestamp: u64,
}

pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(HistogramData),
}

pub struct BufferPoolMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    page_reads: AtomicU64,
    page_writes: AtomicU64,
}

pub struct ExecutorMetrics {
    queries_total: AtomicU64,
    queries_failed: AtomicU64,
    query_duration_ns: AtomicU64,
    rows_processed: AtomicU64,
}
```

### 4.2 健康检查系统

```rust
pub struct HealthChecker {
    start_time: Instant,
    version: String,
    components: Vec<Box<dyn HealthComponent>>,
}

pub trait HealthComponent: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> ComponentHealth;
}

pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthChecker {
    pub fn check_live(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
    
    pub fn check_ready(&self) -> HealthReport {
        let checks: HashMap<String, ComponentHealth> = self.components
            .iter()
            .map(|c| (c.name().to_string(), c.check()))
            .collect();
        
        let status = if checks.values().all(|c| c.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if checks.values().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        };
        
        HealthReport { status, checks }
    }
}
```

### 4.3 Prometheus 暴露格式

```
# HELP sqlrustgo_buffer_pool_hits Total buffer pool cache hits
# TYPE sqlrustgo_buffer_pool_hits counter
sqlrustgo_buffer_pool_hits 12345

# HELP sqlrustgo_buffer_pool_misses Total buffer pool cache misses
# TYPE sqlrustgo_buffer_pool_misses counter
sqlrustgo_buffer_pool_misses 678

# HELP sqlrustgo_query_duration_seconds Query execution duration
# TYPE sqlrustgo_query_duration_seconds histogram
sqlrustgo_query_duration_seconds_bucket{le="0.001"} 100
sqlrustgo_query_duration_seconds_bucket{le="0.01"} 500
sqlrustgo_query_duration_seconds_bucket{le="0.1"} 900
sqlrustgo_query_duration_seconds_bucket{le="+Inf"} 1000
sqlrustgo_query_duration_seconds_sum 45.6
sqlrustgo_query_duration_seconds_count 1000
```

---

## 五、里程碑

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          v1.3.0 里程碑                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   v1.3.0-draft ─────────────────────────────────────────────────────────►   │
│   │                                                                          │
│   ├── Week 1-2: 插件系统 + 性能监控基础                                      │
│   │   └── P-001 ~ P-005, M-001 ~ M-003                                       │
│   │                                                                          │
│   ├── Week 3-4: CBO 完善 + 健康检查                                          │
│   │   └── C-001 ~ C-005, H-001 ~ H-005                                       │
│   │                                                                          │
│   ├── Week 5-6: Join 算法 + 指标暴露                                         │
│   │   └── J-001 ~ J-004, E-001 ~ E-004                                       │
│   │                                                                          │
│   ├── Week 7-8: 事务增强 + 集成测试                                          │
│   │   └── T-001 ~ T-004, 集成测试                                            │
│   │                                                                          │
│   └── Week 9-10: 测试与文档                                                 │
│       └── D-001 ~ D-004: 文档 + Release                                      │
│                                                                              │
│   v1.3.0 GA ─────────────────────────────────────────────────────────────►   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 六、验收标准

### 6.1 功能验收

| 验收项 | 标准 |
|--------|------|
| 插件系统 | Plugin trait + 加载器完整实现 |
| CBO | 成本优化生效，查询计划可验证 |
| Join 算法 | SortMergeJoin + NestedLoopJoin 可用 |
| 事务增强 | 隔离级别 + MVCC 基础实现 |
| 性能监控 | 指标收集 + Prometheus 暴露 |
| 健康检查 | 三个端点全部可用 |

### 6.2 可观测性验收

| 指标 | 目标 |
|------|------|
| /health/live | 返回 200 + {"status": "alive"} |
| /health/ready | 检查所有组件状态 |
| /health | 返回详细健康报告 |
| /metrics | Prometheus 格式指标 |
| 指标数量 | ≥ 20 个核心指标 |

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
| MVCC 实现复杂度超预期 | 高 | 中 | 简化初始版本，仅实现快照隔离 |
| 插件系统设计变更 | 中 | 低 | 参考 Apache Arrow DataFusion |
| 健康检查与现有代码冲突 | 低 | 低 | 独立模块，最小侵入 |

---

## 八、发布计划

### 8.1 版本流程

```
v1.3.0-draft → v1.3.0-alpha → v1.3.0-beta → v1.3.0-rc → v1.3.0
```

### 8.2 时间表

| 版本 | 预计时间 | 说明 |
|------|----------|------|
| v1.3.0-draft | Week 2 | 插件系统 + 监控基础 |
| v1.3.0-alpha | Week 4 | CBO + 健康检查 |
| v1.3.0-beta | Week 6 | Join 算法 + 指标暴露 |
| v1.3.0-rc | Week 8 | 事务增强完成 |
| v1.3.0 | Week 10 | 正式发布 |

---

## 九、关联 Issue

- 父 Issue: #88 (SQLRustGo 2.0 总体开发计划)
- 前置 Issue: #117 (v1.2.0 开发计划)
- 子 Issue:
  - #106 插件系统完整实现
  - #109 CBO 成本优化器实现
  - #110 Join 算法演进
  - #111 架构治理与长期演进规划
- 新增 Issue:
  - 可观测性系统实现 (待创建)

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-05 | 初始版本，新增可观测性轨道 |

---

*本文档由 yinglichina8848 制定*
