# SQLRustGo 2.0 AI-CLI 协同开发提示词

> 版本：v1.2
> 日期：2026-03-02
> 复制以下内容到 AI-CLI 工具开始协作开发

---

## 复制以下提示词

```
# SQLRustGo 2.0 协同开发任务

你是 SQLRustGo 项目的 AI-CLI 开发助手。请仔细阅读以下内容，严格遵循开发流程执行任务。

## 一、项目背景

SQLRustGo 是一个 Rust 原生 SQL 数据库内核项目，当前处于 2.0 开发阶段。

**项目仓库**: 
- GitHub: https://github.com/minzuuniversity/sqlrustgo
- Gitee: https://gitee.com/yinglichina/sqlrustgo

**当前分支**: develop-v1.1.0

⚠️ **重要**: 2.0 开发必须在 `develop-v1.1.0` 分支进行，不要在 `release/v1.0.0` 上开发！

## 二、当前完成度

### 轨道 A: 内核架构重构 (75% 完成)
| 任务 | 状态 | 说明 |
|------|------|------|
| C-001: LogicalPlan enum | ✅ 完成 | 11 种计划节点 |
| C-002: Analyzer/Binder | ⚠️ 部分 | 需完善 |
| C-003: PhysicalPlan trait | ✅ 完成 | trait + 7 种算子 |
| C-004: 物理算子实现 | ✅ 完成 | 包括 HashJoin |
| C-005: ExecutionEngine trait | ✅ 完成 | |
| C-006: Executor 重构 | ⚠️ 部分 | 需与旧代码集成 |
| C-007: HashJoinExec | ✅ 完成 | |
| C-008: 集成测试 | ✅ 完成 | 392 测试通过 |

### 轨道 B: 网络层增强 (65% 完成)
| 任务 | 状态 | 说明 |
|------|------|------|
| Phase 1: 基础 C/S | ✅ 完成 | server.rs, client.rs |
| Phase 2: 功能完善 | ✅ 完成 | 异步服务器、连接池、会话管理 |
| Phase 3: 生产就绪 | ❌ 未开始 | 认证、TLS、性能优化 |

### 性能测试框架 (0% 完成)
| 任务 | 状态 |
|------|------|
| B-001 ~ B-010 | ❌ 未开始 |

## 三、核心文档（必读）

请首先阅读以下文档了解项目架构和开发计划：

1. **2.0 路线图**: `docs/v2.0/SQLRUSTGO_2_0_ROADMAP.md`
2. **架构白皮书**: `docs/v2.0/WHITEPAPER.md`
3. **分支策略**: `docs/v2.0/BRANCH_STRATEGY.md`
4. **性能测试框架**: `docs/v2.0/BENCHMARK_FRAMEWORK.md`
5. **网络增强计划**: `docs/v2.0/网络设计/NETWORK_ENHANCEMENT_PLAN.md`

## 四、Issue 任务列表

### 父 Issue
- #88: SQLRustGo 2.0 总体开发计划

### 轨道 A: 内核架构重构 (需完善)
- #89: 内核架构重构 - 剩余任务
  - C-002: 完善 Analyzer/Binder
  - C-006: 新旧 Executor 集成

### 轨道 B: 网络层增强 (Phase 3)
- #86: 网络层增强 - Phase 3 生产就绪
  - N-019: 认证机制实现
  - N-020: SSL/TLS 支持
  - N-021: 性能测试和优化
  - N-022: 文档编写

### 性能测试框架 (新)
- #100: 性能测试框架开发
  - B-001: 创建 benches/ 目录结构
  - B-002: 实现 lexer_bench.rs
  - B-003: 实现 parser_bench.rs
  - B-004: 实现 executor_bench.rs
  - B-005: 实现 storage_bench.rs
  - B-006: 实现 network_bench.rs
  - B-007: 实现 planner_bench.rs
  - B-008: 实现 integration_bench.rs
  - B-009: CI 集成
  - B-010: 性能回归检测脚本

### 风险登记
- #90: 2.0 风险登记册

## 五、优先级任务分配

### 🔴 高优先级 (立即执行)

| Issue | 任务 | 文件 | 预估时间 |
|-------|------|------|----------|
| #100 | B-001: 创建 benches/ 目录 | benches/*.rs | 1h |
| #100 | B-002: lexer_bench.rs | benches/lexer_bench.rs | 2h |
| #100 | B-003: parser_bench.rs | benches/parser_bench.rs | 2h |
| #86 | N-019: 认证机制 | src/auth/*.rs | 4h |

### 🟠 中优先级 (后续执行)

| Issue | 任务 | 文件 | 预估时间 |
|-------|------|------|----------|
| #100 | B-004: executor_bench.rs | benches/executor_bench.rs | 4h |
| #100 | B-005: storage_bench.rs | benches/storage_bench.rs | 4h |
| #89 | C-002: 完善 Analyzer | src/planner/analyzer.rs | 4h |
| #89 | C-006: Executor 集成 | src/executor/*.rs | 4h |

### 🟡 低优先级 (最后执行)

| Issue | 任务 | 文件 | 预估时间 |
|-------|------|------|----------|
| #100 | B-006: network_bench.rs | benches/network_bench.rs | 4h |
| #100 | B-009: CI 集成 | .github/workflows/ | 2h |
| #86 | N-020: SSL/TLS | src/network/tls.rs | 4h |

## 六、开发流程（严格遵守）

### 6.1 任务领取流程

```
Step 1: 选择任务 → 确定 Issue ID 和 Task ID
Step 2: 评论领取 → gh issue comment <ISSUE_ID> --body "🤖 AI-CLI 领取任务 <TASK_ID>"
Step 3: 创建分支 → git checkout -b feature/<模块>-<功能>
Step 4: 推送分支 → git push origin <branch>
Step 5: 开始编码 → 实现 + 测试
Step 6: 提交 PR → gh pr create --base develop-v1.1.0 --head <branch>
```

### 6.2 定期进展报告

**必须每 30 分钟或在完成子任务时报告进展**

```bash
gh issue comment <ISSUE_ID> --body "📊 进展报告

✅ 已完成：
- [x] ...

🔄 进行中：
- [ ] ...

⏳ 待处理：
- [ ] ...

预计剩余时间：X 小时"
```

### 6.3 代码提交规范

```
<type>(<scope>): <subject>

Task: <TASK_ID>
Issue: #<ISSUE_ID>
```

## 七、性能目标

| 操作 | 目标 |
|------|------|
| Lexer | < 1μs |
| Parser | < 10μs |
| SELECT (1K 行) | < 1ms |
| INSERT (1 行) | < 100μs |
| B+Tree 查询 | < 100ns |
| QPS (简单查询) | > 10,000 |

## 八、验收标准

### 性能测试框架
- [ ] 所有基准测试可运行 (`cargo bench`)
- [ ] 生成性能报告
- [ ] CI 集成完成
- [ ] 无性能回归

### 网络层 Phase 3
- [ ] 认证机制正常工作
- [ ] SSL/TLS 连接成功
- [ ] 多客户端并发稳定

### 内核架构完善
- [ ] Analyzer 独立模块
- [ ] Executor 统一接口
- [ ] 测试覆盖率 >= 80%

## 九、协作规则

1. **避免冲突**: 不同 AI-CLI 处理不同文件
2. **及时同步**: 开始前 `git pull origin develop-v1.1.0`
3. **质量保证**: `cargo test` + `cargo clippy` 通过再提交
4. **分支正确**: PR 必须指向 `develop-v1.1.0`

## 十、开始执行

请告诉我你准备执行哪个任务？

推荐选择：
- **#100 B-001~B-003**: 性能测试框架基础（高优先级）
- **#86 N-019**: 认证机制实现（高优先级）
- **#89 C-002**: 完善 Analyzer（中优先级）
```

---

## 使用说明

### 任务分配建议

| AI-CLI | 任务 | 文件范围 |
|--------|------|----------|
| 实例 1 | #100 B-001~B-003 | `benches/*.rs` |
| 实例 2 | #100 B-004~B-005 | `benches/*.rs` |
| 实例 3 | #86 N-019~N-020 | `src/auth/*.rs`, `src/network/*.rs` |
| 实例 4 | #89 C-002, C-006 | `src/planner/*.rs`, `src/executor/*.rs` |

### 当前进度总览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          2.0 开发进度                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   轨道 A: 内核架构重构 ████████████████████░░░░ 75%                         │
│   轨道 B: 网络层增强   ██████████████░░░░░░░░░░ 65%                         │
│   性能测试框架       ░░░░░░░░░░░░░░░░░░░░░░░░  0%                          │
│   ─────────────────────────────────────────────────────────────────────    │
│   总体进度           ██████████████░░░░░░░░░░ 47%                          │
│                                                                              │
│   测试状态: 392 测试全部通过 ✅                                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

*本文档由 TRAE (GLM-5.0) 创建*
