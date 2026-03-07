# SQLRustGo v1.3.0 发布门禁检查清单

> 版本：v1.3.0
> 日期：2026-03-05
> 发布类型：规划中
> 目标成熟度：L4 企业级

---

## 一、发布概览

### 1.1 版本信息

| 项目 | 值 |
|------|-----|
| **版本号** | v1.3.0 |
| **发布类型** | 规划中 |
| **目标分支** | release/v1.3.0 |
| **开发分支** | develop-v1.3.0 |
| **前置版本** | v1.2.0 (GA) |
| **目标成熟度** | L4 企业级 |

### 1.2 版本目标

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          v1.3.0 核心目标                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   🎯 架构升级：L3 → L4 企业级                                                │
│                                                                              │
│   ✅ 插件系统完整实现                                                        │
│   ✅ CBO 成本优化器完善                                                      │
│   ✅ Join 算法演进 (SortMergeJoin)                                           │
│   ✅ 事务隔离级别 + MVCC 基础                                                │
│   ✅ 性能监控指标系统                                                        │
│   ✅ 健康检查端点                                                            │
│   ✅ Prometheus 指标暴露                                                     │
│   ✅ 测试覆盖率 ≥ 90%                                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、门禁检查清单

### 2.1 🔴 必须项

#### A. 代码质量门禁

| ID | 检查项 | 状态 | 说明 | 检查结果 |
|----|--------|------|------|----------|
| A-01 | 编译通过 | ⏳ | `cargo build --all` 无错误 | - |
| A-02 | 测试通过 | ⏳ | `cargo test --all` 全部通过 | - |
| A-03 | Clippy 检查 | ⏳ | `cargo clippy -- -D warnings` 无警告 | - |
| A-04 | 格式检查 | ⏳ | `cargo fmt --all -- --check` 通过 | - |
| A-05 | 无 unwrap/panic | ⏳ | 核心代码无 unwrap/panic 调用 | - |
| A-06 | 错误处理完整 | ⏳ | 使用 SqlResult<T> 统一错误处理 | - |

#### B. 测试覆盖门禁

| ID | 检查项 | 状态 | 当前值 | 目标值 | 说明 |
|----|--------|------|--------|--------|------|
| B-01 | 行覆盖率 | ⏳ | - | ≥90% | - |
| B-02 | 函数覆盖率 | ⏳ | - | ≥85% | - |
| B-03 | 区域覆盖率 | ⏳ | - | ≥85% | - |
| B-04 | 核心模块覆盖率 | ⏳ | - | ≥80% | plugin/executor/transaction |
| B-05 | 新增代码覆盖率 | ⏳ | - | ≥80% | v1.3.0 新增代码 |

#### C. 功能完整性门禁

| ID | 检查项 | 状态 | 说明 | Issue/PR |
|----|--------|------|------|----------|
| C-01 | 插件系统 | ⏳ | Plugin trait + 加载器 | #106 |
| C-02 | CBO 完善 | ⏳ | 成本模型 + 优化器 | #109 |
| C-03 | SortMergeJoin | ⏳ | 新 Join 算法 | #110 |
| C-04 | 事务隔离级别 | ⏳ | Read Committed/Repeatable Read | - |
| C-05 | MVCC 基础 | ⏳ | 快照隔离 | - |
| C-06 | 性能监控 | ⏳ | Metrics 系统 | - |
| C-07 | 健康检查 | ⏳ | Health 端点 | - |

---

### 2.2 🟠 重要项

#### D. 可观测性门禁

| ID | 检查项 | 状态 | 说明 |
|----|--------|------|------|
| D-01 | /health/live 端点 | ⏳ | 存活探针 |
| D-02 | /health/ready 端点 | ⏳ | 就绪探针 |
| D-03 | /health 端点 | ⏳ | 综合健康检查 |
| D-04 | /metrics 端点 | ⏳ | Prometheus 格式 |
| D-05 | 核心指标 ≥ 20 个 | ⏳ | BufferPool/Executor/Network |
| D-06 | Grafana Dashboard | ⏳ | 可视化模板 |

#### E. 性能门禁

| ID | 检查项 | 状态 | 说明 |
|----|--------|------|------|
| E-01 | 性能基准测试 | ⏳ | 继承 v1.2.0 基准 |
| E-02 | Join 性能测试 | ⏳ | SortMergeJoin vs HashJoin |
| E-03 | 事务性能测试 | ⏳ | 并发事务吞吐量 |
| E-04 | 无性能退化 | ⏳ | 与 v1.2.0 对比 |

#### F. 文档门禁

| ID | 检查项 | 状态 | 说明 |
|----|--------|------|------|
| F-01 | Release Notes | ⏳ | 版本发布说明 |
| F-02 | CHANGELOG 更新 | ⏳ | 变更日志 |
| F-03 | API 文档 | ⏳ | 公共 API 文档注释 |
| F-04 | 可观测性指南 | ⏳ | 监控和健康检查使用说明 |
| F-05 | 升级指南 | ⏳ | v1.2.0 → v1.3.0 迁移指南 |

#### G. 安全门禁

| ID | 检查项 | 状态 | 说明 |
|----|--------|------|------|
| G-01 | 依赖审计 | ⏳ | `cargo audit` 通过 |
| G-02 | 安全扫描 | ⏳ | 无高危安全问题 |
| G-03 | 敏感信息检查 | ⏳ | 无密钥/凭证泄露 |

---

### 2.3 🟡 建议项

#### H. 工程化门禁

| ID | 检查项 | 状态 | 说明 |
|----|--------|------|------|
| H-01 | CI 流程完整 | ⏳ | GitHub Actions 配置完整 |
| H-02 | 分支保护配置 | ⏳ | develop-v1.3.0 保护规则 |
| H-03 | 代码所有者 | ⏳ | CODEOWNERS 文件更新 |
| H-04 | Issue 关联 | ⏳ | 所有 PR 关联 Issue |
| H-05 | Commit 规范 | ⏳ | 遵循 Conventional Commits |

---

## 三、可观测性验收标准

### 3.1 健康检查端点

| 端点 | 请求 | 预期响应 | 状态码 |
|------|------|----------|--------|
| `/health/live` | GET | `{"status": "alive"}` | 200 |
| `/health/ready` | GET | `{"status": "ready", "checks": {...}}` | 200/503 |
| `/health` | GET | 详细健康报告 | 200/503 |

### 3.2 性能指标

| 指标类别 | 指标名称 | 类型 | 说明 |
|----------|----------|------|------|
| BufferPool | sqlrustgo_buffer_pool_hits | Counter | 缓存命中次数 |
| BufferPool | sqlrustgo_buffer_pool_misses | Counter | 缓存未命中次数 |
| BufferPool | sqlrustgo_buffer_pool_evictions | Counter | 页面淘汰次数 |
| Executor | sqlrustgo_queries_total | Counter | 查询总数 |
| Executor | sqlrustgo_queries_failed | Counter | 失败查询数 |
| Executor | sqlrustgo_query_duration_seconds | Histogram | 查询延迟 |
| Network | sqlrustgo_connections_active | Gauge | 活跃连接数 |
| Network | sqlrustgo_connections_total | Counter | 连接总数 |

---

## 四、发布流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          v1.3.0 发布流程                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Phase 1: 开发阶段                                                          │
│   ├── 1.1 插件系统实现                                                      │
│   ├── 1.2 CBO 完善                                                          │
│   ├── 1.3 Join 算法演进                                                     │
│   ├── 1.4 事务增强                                                          │
│   ├── 1.5 可观测性系统                                                      │
│   └── 1.6 测试与文档                                                        │
│                                                                              │
│   Phase 2: 验证阶段                                                          │
│   ├── 2.1 执行完整测试套件                                                  │
│   ├── 2.2 执行性能基准测试                                                  │
│   ├── 2.3 执行安全审计                                                      │
│   └── 2.4 可观测性验收                                                      │
│                                                                              │
│   Phase 3: 发布阶段                                                          │
│   ├── 3.1 创建 v1.3.0 Tag                                                   │
│   ├── 3.2 发布 GitHub Release                                               │
│   └── 3.3 合并到 main 分支                                                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 五、检查命令

```bash
# 代码质量
cargo build --all
cargo test --all
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check

# 测试覆盖率
cargo llvm-cov --all-features

# 安全审计
cargo audit
cargo outdated

# 性能测试
cargo bench

# 健康检查验证
curl http://localhost:3306/health/live
curl http://localhost:3306/health/ready
curl http://localhost:3306/health
curl http://localhost:3306/metrics
```

---

## 六、相关文档

- [版本计划](./VERSION_PLAN.md)
- [v1.2.0 发布门禁](../v1.2.0/RELEASE_GATE_CHECKLIST.md)
- [健康检查规范](../v1.1.0/HEALTH_CHECK_SPECIFICATION.md)
- [监控规范](../v1.1.0/MONITORING_SPECIFICATION.md)

---

*本文档由 yinglichina8848 创建*
*最后更新: 2026-03-05*
