# v3.2.0 GA 遗留问题追踪

> **版本**: v1.0
> **日期**: 2026-05-18
> **维护人**: hermes-agent
> **范围**: v3.2.0 GA 遗留问题 → v3.3.0 修复目标
> **依据**: Issue #1196, #1197, #1198 及 GA_READINESS_GAP_ANALYSIS.md

---

## 一、执行摘要

v3.2.0 GA Gate 通过率 41/46 (89.1%)，有 5 项未通过，其中 2 项为 GA 阻塞项（P0），3 项为非阻塞项。所有未通过项均有 Issue 追踪并分配至 v3.3.0 milestone。

### 遗留问题汇总

| 优先级 | Issue | 问题 | 影响 | 目标版本 |
|--------|-------|------|------|----------|
| 🔴 P0 | #1196/#1197 | executor 覆盖率 70.7% < 85% | GA 门禁失败 | v3.3.0 RC |
| 🔴 P0 | (待建) | MySQL Protocol 握手失败 | GA 门禁失败 | v3.3.0 RC |
| 🟡 P1 | #1198 | TPC-H SF=1 数据缺失 | 无法验证 | v3.3.0 RC |
| 🟡 P2 | #1198 | 72h 稳定性测试未完成 | 稳定性未确认 | v3.3.0 GA |
| 🟡 P3 | — | Coverage 数据矛盾（85.81% vs 68.8%） | 测量标准不统一 | v3.3.0 Alpha |

---

## 二、🔴 P0 遗留项（阻塞 v3.3.0 GA）

### 2.1 L-001: executor 模块覆盖率不达标

| 属性 | 值 |
|------|-----|
| **Issue** | #1196, #1197 |
| **标签** | `GA-blocker`, `P0`, `v3.3.0` |
| **当前状态** | executor 覆盖率 70.70%，目标 ≥85% |
| **差距** | -14.3%（需新增约 1,294 行覆盖） |

#### 根因分析

**主要贡献者**:

| 文件 | 覆盖率 | 总行数 | 未覆盖行数 |
|------|---------|--------|-----------|
| `stored_proc.rs` | **41.8%** | 3,001 | **1,748** |
| `merge.rs` | 21.0% | 319 | 252 |

**核心问题**: `stored_proc.rs` 实现完整的 SQL/PSM 存储过程语言（CTE、递归查询、游标管理、异常处理），但 1,748 行代码因架构设计问题（过长函数 + 深度嵌套控制流）无法被现有测试触达。现有 417 个 executor 测试 + 100+ 集成测试已无法进一步提升覆盖。

#### 整改方案

**Phase 1 — 模块拆分（v3.3.0 Alpha）**

```
crates/executor/src/stored_proc/
├── mod.rs           # 主入口（保持 API 兼容）
├── expression.rs    # 表达式求值（可独立测试）
├── cursor.rs        # 游标管理
├── handler.rs       # 异常处理
└── cte.rs          # CTE/递归
```

**Phase 2 — 逐模块覆盖（v3.3.0 Beta/RC）**

| 子模块 | 目标覆盖率 | 具体任务 |
|--------|-----------|----------|
| `expression.rs` | 95%+ | 为每个表达式类型分支编写测试 |
| `cursor.rs` | 90%+ | 游标状态转换测试 |
| `handler.rs` | 85%+ | 异常处理路径测试 |
| `cte.rs` | 80%+ | 递归 CTE 执行测试 |
| 集成测试 | — | 主要执行路径覆盖 |

#### 验收标准

- [ ] `cargo llvm-cov test -p sqlrustgo-executor --lib` 显示 ≥85%
- [ ] `cargo test --all-features` 全部通过
- [ ] 无新增 `unsafe` 代码
- [ ] 性能回归测试通过（`cargo bench`）

#### 技术债务清理

| 函数 | 当前行数 | 目标行数 |
|------|----------|----------|
| `expression_to_value` | 244 | <80 |
| `evaluate_row_expression` | 226 | <80 |
| `execute_statement` | 314 | <100 |

---

### 2.2 L-002: MySQL Protocol 握手失败

| 属性 | 值 |
|------|-----|
| **Issue** | (待建) |
| **标签** | `GA-blocker`, `P0`, `v3.3.0` |
| **当前状态** | `mysql::Conn::new()` 返回 `DriverError { Could not setup connection }` |
| **根因** | sqlrustgo-mysql-server 的 MySQL 协议握手实现不完整 |

#### 根因分析

握手包交换流程（ClientHandshake → ServerGreeting → AuthSwitch）中的 `auth plugin` 兼容性或握手状态机存在问题。参考文档：`docs/releases/v3.1.0/MYSQL_PROTOCOL_OPTIMIZATION.md`。

#### 整改方案

1. 调试 handshake 包交换（抓包分析 Client ↔ Server 交互）
2. 检查 `auth plugin` 兼容性（`mysql_native_password` vs `caching_sha2_password`）
3. 参考 MySQL 5.7/8.0 协议规范修正状态机

#### 验收标准

- [ ] `mysql::Conn::new()` 连接成功
- [ ] `mysql_protocol_handshake_test` 全部通过
- [ ] MySQL 客户端可正常连接 sqlrustgo-server

---

## 三、🟡 P1 遗留项

### 3.1 L-003: TPC-H SF=1 数据缺失

| 属性 | 值 |
|------|-----|
| **Issue** | #1198（部分追踪） |
| **标签** | `GA-blocker`, `P1`, `v3.3.0` |
| **当前状态** | `tpch_data/` 目录不存在，无法运行 TPC-H SF=1 |
| **影响** | GA Gate G8 (TPC-H SF=1 22/22) 无法验证 |

#### 整改方案

```bash
# 生成 TPC-H SF=1 数据
bash scripts/gate/setup_tpch_env.sh --sf 1

# 执行验证
bash scripts/gate/check_tpch.sh --sf1
```

#### 验收标准

- [ ] `tpch_data/` 目录存在且包含 SF=1 数据
- [ ] `check_tpch.sh --sf1` 输出 22/22 PASS
- [ ] 无 OOM 错误

---

### 3.2 L-004: 72 小时稳定性测试未完成

| 属性 | 值 |
|------|-----|
| **Issue** | #1198 |
| **标签** | `P2`, `v3.3.0` |
| **当前状态** | 16h 测试已完成；24h 测试进行中（58%）；48h/72h 未开始 |

#### 当前进度

| 测试 | 状态 | 结果 |
|------|------|------|
| `test_sustained_write_16h` | ✅ 完成 | 100,589,900 次插入，1746.35 ops/sec |
| `test_concurrent_read_write_16h` | ✅ 完成 | 527,478 写入 / 2,096,784 读取 |
| `test_sustained_write_24h` | 🔄 进行中 | ~13.9h 已运行，~10.1h 剩余 |
| `test_concurrent_read_write_24h` | 🔄 进行中 | 同上 |
| `test_sustained_write_48h` | ⏳ 待执行 | — |
| `test_sustained_write_72h` | ⏳ 待执行 | — |
| `test_concurrent_read_write_48h` | ⏳ 待执行 | — |
| `test_concurrent_read_write_72h` | ⏳ 待执行 | — |

#### 已知警告（需修复）

| 警告 | 严重度 | 修复方向 |
|------|--------|----------|
| `ExecutionEngine::new` 已废弃 | 🟡 | 改用 `new_with_config()` 或 `EngineConfig::builder()` |
| `create_memory_engine()` 未使用 (dead_code) | ℹ️ | 清理或移到 `#[allow(dead_code)]` |

#### 验收标准

- [ ] 24h 测试完成且无崩溃、无内存泄漏
- [ ] 48h 测试完成且无崩溃、无内存泄漏
- [ ] 72h 测试完成且无崩溃、无内存泄漏
- [ ] `deprecation warnings` 已修复

---

## 四、🟡 P3 遗留项

### 4.1 L-005: Coverage 数据矛盾

| 属性 | 值 |
|------|-----|
| **Issue** | (待建) |
| **标签** | `P3`, `v3.3.0` |

#### 问题描述

三个文档对同一测量给出三个不同数字：

| 文档 | 报告数值 | 日期 |
|------|---------|------|
| GA_GATE_CHECKLIST.md | 85.81% | 2026-05-18 |
| RC_TO_GA_FINAL_REPORT.md | 68.8% | 2026-05-17 |
| GA_READINESS_GAP_ANALYSIS.md | ⏳ 测量中 | 2026-05-17 |

#### 根因

可能的原因：
1. **测量范围不同**: L1_CRATES vs 全量 crate
2. **测量时间点不同**: RC Gate vs GA Gate
3. **llvm-cov 版本差异**: 不同版本结果略有差异
4. **数据书写错误**: GA_GATE_CHECKLIST.md 填错了数字

#### 整改方案

1. 统一覆盖率测量标准（明确 L1_CRATES 定义）
2. 在 `gate_spec_v320.md` 中明确覆盖率测量命令
3. 建立覆盖率基准快照（`coverage_baseline_v320.json`）

---

## 五、v3.3.0 修复追踪表

| 遗留 ID | Issue | 优先级 | 负责人 | 目标阶段 | 状态 |
|---------|-------|--------|--------|----------|------|
| L-001 | #1196/#1197 | P0 | — | v3.3.0 RC | 🔴 Open |
| L-002 | (待建) | P0 | — | v3.3.0 RC | 🔴 Open |
| L-003 | #1198 | P1 | — | v3.3.0 RC | 🟡 Open |
| L-004 | #1198 | P2 | — | v3.3.0 GA | 🟡 Open |
| L-005 | (待建) | P3 | — | v3.3.0 Alpha | 🟡 Open |

---

## 六、修复 PR 记录

| 遗留 ID | PR | 内容 | 状态 |
|---------|-----|------|------|
| — | — | — | — |

（修复 PR 建立后在此记录）

---

*本文档由 hermes-agent 生成*
*最后更新: 2026-05-18*
