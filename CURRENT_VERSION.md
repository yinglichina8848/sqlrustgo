# 当前版本状态

develop/v3.2.0

## 阶段信息

- **阶段**: RC → GA 收敛阶段
- **当前里程碑**: v3.2.0 RC
- **开始日期**: 2026-05-18
- **开发分支**: develop/v3.2.0
- **目标**: GA 门禁通过

## 版本概述

v3.2.0 聚焦于 Trust Convergence（可信收敛），确保系统满足 GMP（良好制造规范）工业标准。

**核心目标**:
- MySQL 协议完整兼容
- 性能稳定无回归
- 审计链完整性验证
- Crash Recovery 验证
- GMP Long-Run 稳定性

## RC → GA 核心原则

> **禁止架构扩散，聚焦可信收敛**

| 类别 | 允许 | 不允许 |
|------|------|--------|
| Bug 修复 | ✅ | - |
| 性能恢复 | ✅ | - |
| 覆盖率补全 | ✅ | - |
| Recovery/稳定性验证 | ✅ | - |
| GMP 深度验证 | ✅ | - |
| 新架构 | ❌ | - |
| 新平台 | ❌ | - |
| 新 UI 系统 | ❌ | - |
| 新协议层 | ❌ | - |
| 分布式大改 | ❌ | - |

## v3.2.0 已完成工作

### P0 Issues

| # | 内容 | PR | 状态 |
|---|------|-----|------|
| #1156 | UPDATE/DELETE WHERE 子句修复 + 性能回归调查 | #1174 | ✅ closed |
| #1159 | Audit Chain Validator 增强 | #1180 | ✅ merged |
| #1157 | WAL Crash Recovery 测试 | #1168 | ✅ merged |
| #1158 | Crash Recovery 验证 | #1166 | ✅ merged |

### 性能数据（qps_benchmark_test）

| 操作 | v3.2.0 实测 | v3.0.0 基线 | 状态 |
|------|------------|------------|------|
| UPDATE | 109,988 QPS | 43,121 QPS | ✅ +155% |
| DELETE | 134,312 QPS | 64,896 QPS | ✅ +107% |
| INSERT | 73,261 QPS | 28,698 QPS | ✅ +155% |

**结论**: 没有发现 -89% 性能回归。v3.2.0 性能实际上比 v3.0.0 快 2-3 倍。

### GMP 审计链增强

| 验证项 | 状态 |
|--------|------|
| hash continuity | ✅ |
| timestamp ordering | ✅ 新增 |
| signature validity | ✅ |
| orphan correction | ✅ 新增 |
| workflow linkage | ✅ |
| provenance completeness | ✅ |

## 门禁状态

详见 [docs/governance/gate_spec_v310.md](docs/governance/gate_spec_v310.md)（适用于 v3.1.0+）

## 下一步工作

| 优先级 | 内容 | Issue |
|--------|------|-------|
| P0 | GMP Long-Run Stability (72h) | #1160 |
| P0 | MySQL 协议兼容性验证 | - |
| P1 | Evidence Export | - |
| P1 | Compliance-as-Code Lite | - |
