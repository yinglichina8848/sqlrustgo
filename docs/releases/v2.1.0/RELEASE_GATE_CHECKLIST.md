# SQLRustGo v2.1.0 发布门禁清单

**版本**: v2.1.0
**日期**: 2026-04-02
**状态**: RC1 ✅

---

## 一、门禁概述

本清单定义 v2.1.0 发布的门禁检查标准，确保所有核心功能和性能指标达标。

---

## 二、门禁检查阶段

### 2.1 开发阶段门禁（Daily）

| 检查项 | 命令 | 通过标准 | 状态 |
|--------|------|----------|------|
| 代码编译 | `cargo build --release` | 无错误 | ✅ |
| 单元测试 | `cargo test --lib` | 100% 通过 | ✅ |
| 代码格式 | `cargo fmt --check` | 无警告 | ✅ |
| 代码规范 | `cargo clippy` | 无 error | ✅ (warnings) |

### 2.2 集成阶段门禁（Weekly）

| 检查项 | 命令 | 通过标准 | 状态 |
|--------|------|----------|------|
| 集成测试 | `cargo test --test '*_test'` | 100% 通过 | ✅ |
| TPC-H 测试 | `cargo test --test tpch_test` | 22/22 通过 | ✅ |
| 回归测试 | `cargo test --test regression_test` | 1039/1039 通过 | ✅ |
| 覆盖率 | `cargo tarpaulin` | ≥80% | ⏳ 待验证 |

### 2.3 发布阶段门禁（Release）

| 检查项 | 命令 | 通过标准 | 状态 |
|--------|------|----------|------|
| 完整测试 | `cargo test --workspace` | 100% 通过 | ⏳ |
| 性能测试 | `cargo bench` | 达标 | ⏳ |
| 文档检查 | `cargo doc --no-deps` | 无警告 | ⏳ |

---

## 三、功能门禁检查

### 3.1 P0 功能（必须通过）

| 功能 | 测试命令 | 通过标准 | 状态 |
|------|----------|----------|------|
| SQL 解析 | `cargo test -p sqlrustgo-parser` | 100% | ✅ |
| 存储引擎 | `cargo test -p sqlrustgo-storage` | 100% | ✅ |
| 执行器 | `cargo test -p sqlrustgo-executor` | 100% | ✅ |
| MockStorage 移除 | `cargo test --test regression_test` | 全部通过 | ✅ |

### 3.2 P1 功能（建议通过）

| 功能 | 测试命令 | 通过标准 | 状态 |
|------|----------|----------|------|
| Prometheus 指标 | `cargo test server_health_test` | 通过 | ✅ |
| 慢查询日志 | `cargo test slow_query_log_test` | 通过 | ✅ |
| KILL 语句 | `cargo test mysql_compatibility_test` | 通过 | ✅ |
| Physical Backup | `cargo test physical_backup_test` | 通过 | ✅ |
| 备份保留策略 | `cargo test physical_backup_test` | prune 通过 | ✅ |

### 3.3 P2 功能（可选通过）

| 功能 | 测试命令 | 通过标准 | 状态 |
|------|----------|----------|------|
| UUID 类型 | `cargo test types_value_test` | 通过 | ✅ |
| ARRAY 类型 | `cargo test types_value_test` | 通过 | ✅ |
| ENUM 类型 | `cargo test types_value_test` | 通过 | ✅ |
| TPC-H BETWEEN | `cargo test tpch_test` | 通过 | ✅ |
| TPC-H IN | `cargo test tpch_test` | 通过 | ✅ |

---

## 四、性能门禁检查

### 4.1 性能基准

| 指标 | 目标 | 测试命令 | 状态 |
|------|------|----------|------|
| QPS (50并发) | ≥1000 | `cargo bench --bench tpch_benchmark` | ⏳ |
| P50 延迟 | <50ms | 基准测试输出 | ⏳ |
| P99 延迟 | <100ms | 基准测试输出 | ⏳ |

### 4.2 内存使用

| 指标 | 目标 | 状态 |
|------|------|------|
| 空闲内存 | <500MB | ⏳ |
| 峰值内存 | <2GB | ⏳ |

---

## 五、安全门禁检查

| 检查项 | 命令/方法 | 通过标准 | 状态 |
|--------|-----------|----------|------|
| SQL 防火墙 | 日志审查 | 无异常 | ✅ |
| 权限系统 | `cargo test auth_rbac_test` | 通过 | ✅ |
| 敏感数据 | 代码审查 | 无泄露 | ✅ |

---

## 六、文档门禁检查

| 检查项 | 状态 |
|--------|------|
| RELEASE_NOTES.md | ✅ |
| CHANGELOG.md | ✅ |
| USER_MANUAL.md | ✅ |
| TEST_MANUAL.md | ✅ |
| API_DOCUMENTATION.md | ✅ |
| MIGRATION_GUIDE.md | ✅ |

---

## 七、Issue 关闭状态

| Issue | 描述 | 状态 |
|-------|------|------|
| #1198 | 备份保留策略 | ✅ 已关闭 |
| #1018 | Physical Backup CLI | ✅ 已关闭 |
| #1210 | TPC-H Phase 1 | ✅ 已合并 |
| #1128 | UUID/ARRAY/ENUM | ✅ 已合并 |
| #1207 | MockStorage stub 修复 | ✅ 已关闭 |

### 开放 Issue

| Issue | 描述 | 状态 |
|-------|------|------|
| #1231 | TPC-H 合规性测试缺失 | ⚠️ P1 |
| #1137 | 测试覆盖率提升 - 目标 80% | ⚠️ P2 |

---

## 八、最终审核

| 审核项 | 审核人 | 日期 | 签名 |
|--------|--------|------|------|
| 代码审核 | | | |
| 测试审核 | | | |
| 文档审核 | | | |
| 安全审核 | | | |
| 发布批准 | | | |

---

## 九、门禁结果

| 项目 | 结果 |
|------|------|
| 编译 | ✅ 通过 |
| 测试 | ✅ 1039/1039 (100%) |
| 覆盖率 | ⏳ 待验证 (tarpaulin 超时) |
| 性能 | ⏳ 待验证 |
| 安全 | ✅ 通过 |
| 文档 | ✅ 完成 |

**最终决定**: ✅ **可以进入 RC2** / ⬜ 需要修复后发布

---

## 十、版本阶段

| 阶段 | 日期 | 状态 |
|------|------|------|
| Alpha | 2026-03-30 | ✅ 完成 |
| Beta | 2026-04-01 | ✅ 完成 |
| RC1 | 2026-04-02 | ✅ **当前阶段** |
| RC2 | 待定 | ⬜ |
| Release | 待定 | ⬜ |

---

*门禁清单 v2.1.0*
*更新时间: 2026-04-02 15:30*
