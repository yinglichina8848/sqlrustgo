# v1.3.0 开发指南

> **版本**: v1.3.0
> **代号**: Observability Engine
> **制定日期**: 2026-03-05

---

## 一、版本概述

v1.3.0 是一个重要的里程碑版本，专注于**可观测性系统**建设，同时完善插件系统和 CBO。

### 1.1 版本目标

| 项目 | 值 |
|------|-----|
| **版本号** | v1.3.0 |
| **代号** | Observability Engine |
| **目标成熟度** | L4 企业级 |
| **核心目标** | 商用级内核，完整 CBO，可观测性 |
| **预计时间** | v1.2.0 GA 后 2 个月 |

### 1.2 主要功能

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 健康检查 | /health/live, /health/ready, /health 端点 | P0 |
| 性能监控 | Metrics trait + 各组件指标采集 | P0 |
| 指标暴露 | Prometheus 格式 + /metrics 端点 | P1 |
| 插件系统 | Plugin trait + 动态加载 | P0 |
| CBO 完善 | 成本模型 + 统计信息集成 | P0 |
| Join 算法 | SortMergeJoin + NestedLoopJoin | P1 |
| 事务增强 | 隔离级别 + MVCC + 锁管理 | P0 |

---

## 二、快速开始

### 2.1 分支管理

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到开发分支
git checkout develop-v1.3.0

# 构建
cargo build --release
```

### 2.2 启动服务

```bash
# 启动服务
cargo run --release

# 检查健康状态
curl http://localhost:5433/health/live
curl http://localhost:5433/health/ready
curl http://localhost:5433/health

# 获取指标
curl http://localhost:5433/metrics
```

---

## 三、开发文档

| 文档 | 说明 |
|------|------|
| [VERSION_PLAN.md](./VERSION_PLAN.md) | 版本计划 |
| [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md) | 门禁检查清单 |
| [TASK_MATRIX.md](./TASK_MATRIX.md) | 任务分解矩阵 |
| [AI_CLI_GUIDE.md](./AI_CLI_GUIDE.md) | AI-CLI 协作指南 |

---

## 四、开发里程碑

### 4.1 时间线

```
Week 1-2:  插件系统 + 性能监控基础
Week 3-4:  CBO 完善 + 健康检查
Week 5-6:  Join 算法 + 指标暴露
Week 7-8:  事务增强
Week 9-10: 测试 + 文档 + 发布
```

### 4.2 版本发布

| 版本 | 时间 | 里程碑 |
|------|------|--------|
| v1.3.0-draft | Week 2 | 插件系统 + 监控基础完成 |
| v1.3.0-alpha | Week 4 | CBO + 健康检查完成 |
| v1.3.0-beta | Week 6 | Join 算法 + 指标暴露完成 |
| v1.3.0-rc | Week 8 | 事务增强完成 |
| v1.3.0 GA | Week 10 | 正式发布 |

---

## 五、核心功能详解

### 5.1 健康检查系统

#### 端点

| 端点 | 用途 | 延迟要求 |
|------|------|----------|
| /health/live | 存活探针 | < 10ms |
| /health/ready | 就绪探针 | < 100ms |
| /health | 综合健康 | < 100ms |

#### 响应示例

```json
// /health/live
{"status": "alive", "version": "1.3.0", "timestamp": "2026-03-05T10:00:00Z"}

// /health/ready
{
  "status": "ready",
  "checks": {
    "storage": {"status": "healthy", "latency_ms": 5},
    "memory": {"status": "healthy", "usage_percent": 45.2},
    "connections": {"status": "healthy", "active": 10, "max": 100}
  }
}
```

### 5.2 性能监控系统

#### 指标类别

| 类别 | 指标数量 | 示例 |
|------|----------|------|
| Buffer Pool | ≥5 | hits, misses, evictions |
| Executor | ≥5 | queries_total, duration, rows |
| Network | ≥4 | connections, bytes |
| Storage | ≥4 | read_bytes, write_bytes |

#### Prometheus 格式

```
# HELP sqlrustgo_buffer_pool_hits_total Total buffer pool cache hits
# TYPE sqlrustgo_buffer_pool_hits_total counter
sqlrustgo_buffer_pool_hits_total 12345

# HELP sqlrustgo_query_duration_seconds Query execution duration
# TYPE sqlrustgo_query_duration_seconds histogram
sqlrustgo_query_duration_seconds_bucket{le="0.001"} 100
sqlrustgo_query_duration_seconds_bucket{le="0.01"} 500
sqlrustgo_query_duration_seconds_sum 45.6
sqlrustgo_query_duration_seconds_count 1000
```

---

## 六、贡献指南

### 6.1 开发流程

1. **选择任务**: 从 TASK_MATRIX.md 选择待认领任务
2. **声明领取**: 在 Issue 中评论领取任务
3. **创建分支**: 从 `develop-v1.3.0` 创建 `fix/v1.3.0-*` 分支
4. **开发**: 遵循编码规范进行开发
5. **提交**: 创建 PR，提交到 `develop-v1.3.0`
6. **审核**: 等待 1 人审核后合并

### 6.2 编码规范

```bash
# 必须通过所有检查
cargo build --release
cargo test --all
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

### 6.3 审核要求

| 分支 | 审核人数 |
|------|----------|
| develop-v1.3.0 | 1 人 |
| release/v1.3.0-* (draft/alpha/beta) | 1 人 |
| release/v1.3.0-* (rc/GA) | 2 人 |

---

## 七、验收标准

### 7.1 功能验收

| 功能 | 验收标准 |
|------|----------|
| 健康检查 | 三个端点全部可用，返回正确格式 |
| 性能监控 | 指标采集正确，数量 ≥ 20 |
| 指标暴露 | Prometheus 格式正确，/metrics 端点可用 |
| 插件系统 | Plugin trait + 加载器完整实现 |
| CBO | 成本优化生效 |
| Join | SortMergeJoin + NestedLoopJoin 可用 |
| 事务 | 隔离级别 + MVCC 基础实现 |

### 7.2 质量验收

| 指标 | 目标 |
|------|------|
| 测试覆盖率 | ≥ 90% |
| Clippy | 零警告 |
| /health/live 延迟 | < 10ms |
| /metrics 延迟 | < 50ms |

---

## 八、版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.3.0-draft | - | 初始版本 |

---

## 九、相关链接

- [GitHub 仓库](https://github.com/minzuuniversity/sqlrustgo)
- [v1.2.0 Release Notes](../v1.2.0/RELEASE_NOTES.md)
- [v1.1.0 Release Notes](../v1.1.0/RELEASE_NOTES.md)

---

*本文档由 AI 助手生成*
