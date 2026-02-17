# Alpha 阶段门禁检查清单

## 概述

本文档定义 v1.0.0-alpha 阶段的门禁标准，用于确保代码质量和流程规范。

## 门禁检查项

### 1. 编译门禁

```bash
cargo build --all-features
```

- [ ] 编译成功
- [ ] 无警告

### 2. 测试门禁

```bash
cargo test --all-features
```

- [ ] 所有测试通过
- [ ] 测试数量 ≥ 100

### 3. 代码规范门禁

```bash
cargo clippy --all-features -- -D warnings
cargo fmt --check
```

- [ ] Clippy 无警告
- [ ] 代码格式化通过

### 4. 覆盖率门禁

```bash
cargo tarpaulin --message-format=short
```

| 模块 | 覆盖率要求 | 当前状态 |
|------|------------|----------|
| network | ≥ 40% | / |
| executor | ≥ 70% | / |
| parser | ≥ 70% | / |
| storage | ≥ 70% | / |
| **总体** | **≥ 75%** | / |

### 5. 文档门禁

- [ ] README.md 完整
- [ ] API 文档已更新
- [ ] CHANGELOG.md 已更新

### 6. PR 质量门禁

- [ ] PR 描述完整
- [ ] 包含测试计划
- [ ] 至少一个 reviewer 批准
- [ ] 无重大风险

## 门禁检查流程

```
开发者提交 PR
    ↓
CI 自动检查 (build/test/clippy/fmt)
    ↓
Reviewer 人工审查
    ↓
Coverage 检查
    ↓
Gatekeeper 最终确认
    ↓
合并到 baseline
```

## 门禁责任人

| 角色 | 职责 |
|------|------|
| Claude Code | 执行开发、修复问题 |
| OpenCode | 文档整理、证据收集 |
| Codex CLI | Gatekeeper、最终审批 |

## 门禁状态追踪

| Issue | 状态 | 日期 |
|-------|------|------|
| #17 (Phase 1) | 进行中 | 2026-02-17 |

## 常见问题

### Q: Coverage 不达标怎么办？
A: 补充测试用例，优先覆盖核心模块

### Q: Clippy 警告如何处理？
A: 使用 `cargo clippy --fix` 自动修复

### Q: PR 被拒绝后如何处理？
A: 根据反馈修复问题，重新提交

## 相关文档

- [并行开发协作指南](./2026-02-16-parallel-development-guide.md)
- [PR 工作流](./2026-02-16-pr-workflow.md)
- [测试覆盖率实现计划](./2026-02-16-test-coverage-impl-plan.md)

---

**维护者**: Codex CLI (Gatekeeper)  
**版本**: 1.0  
**日期**: 2026-02-18
