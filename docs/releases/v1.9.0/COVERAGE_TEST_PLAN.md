# SQLRustGo v1.9.0 覆盖率测试计划

> **版本**: v1.9.0  
> **更新日期**: 2026-03-28  
> **状态**: 测试计划

---

## 1. 测试目标

验证 v1.9.0 版本代码覆盖率，确保达到发布标准。

### 1.1 覆盖率目标

| 指标 | 目标 | 历史基线 |
|------|------|----------|
| 总覆盖率 | ≥70% | 65.24% |
| Parser 模块 | ≥85% | 88.64% |
| Executor 模块 | ≥50% | 50.53% |
| Storage 模块 | ≥55% | 57.62% |

---

## 2. 测试环境

### 2.1 工具版本

| 工具 | 版本 |
|------|------|
| cargo-tarpaulin | 0.35.2+ |
| Rust | stable |
| OS | macOS |

### 2.2 依赖要求

- 所有测试文件可正常编译
- 无 #[ignore] 的测试可正常运行

---

## 3. 测试命令

### 3.1 标准命令（必须使用）

```bash
cargo tarpaulin --workspace --ignore-panics --timeout 120 --out Html
```

### 3.2 参数说明

| 参数 | 值 | 说明 |
|------|-----|------|
| `--workspace` | - | 覆盖整个 workspace（所有 crate） |
| `--ignore-panics` | - | 忽略 panic 的覆盖率统计，避免测试失败导致覆盖率无法生成 |
| `--timeout` | 120 | 单个测试超时时间（秒） |
| `--out` | Html | 输出 HTML 格式报告 |

### 3.3 不允许的参数

- **禁止使用**: `--all` 参数（会导致编译错误，因为某些 MockStorage 实现不完整）
- **禁止使用**: 任何跳过失败测试的自定义参数

---

## 4. 测试范围

### 4.1 包含的模块

| 模块 | 测试目标 | 预期覆盖率 |
|------|----------|------------|
| sqlrustgo-parser | 解析器测试 | ≥85% |
| sqlrustgo-executor | 执行器测试 | ≥50% |
| sqlrustgo-storage | 存储引擎测试 | ≥55% |
| sqlrustgo-planner | Planner 测试 | ≥50% |
| sqlrustgo-optimizer | 优化器测试 | ≥50% |
| sqlrustgo-transaction | 事务测试 | ≥60% |

### 4.2 测试文件

所有集成测试和单元测试（除标记为 #[ignore] 的测试外）:

- `tests/integration/*.rs`
- `tests/unit/*.rs`
- `tests/anomaly/*.rs`
- `tests/stress/*.rs`

### 4.3 排除的模块

以下模块不计入覆盖率（未实现或正在开发）:

| 模块 | 排除原因 |
|------|----------|
| sqlancer | 实验性功能 |
| backup | 独立工具 |
| transaction-stress | 压力测试模块 |

---

## 5. 测试执行步骤

### 5.1 前置检查

1. 确保代码已更新到最新: `git pull origin develop/v1.9.0`
2. 确保测试可以编译: `cargo build --tests`
3. 确保基本测试通过: `cargo test`

### 5.2 执行命令

```bash
# 1. 进入项目目录
cd /Users/liying/workspace/dev/openheart/sqlrustgo

# 2. 运行覆盖率测试
cargo tarpaulin --workspace --ignore-panics --timeout 120 --out Html --output-dir ./coverage-report
```

### 5.3 预期输出

- 控制台输出覆盖率百分比
- HTML 报告生成在 `./coverage-report/tarpaulin-report.html`

---

## 6. 结果记录格式

### 6.1 必须记录的指标

| 指标 | 记录方式 |
|------|----------|
| 总覆盖率 | 百分比（如 65.24%） |
| 执行行数 | 数字（如 7197） |
| 仪器化行数 | 数字（如 11031） |
| 测试时间 | 时间戳 |
| 变更 | 与上次对比的差异 |

### 6.2 模块级覆盖率

| 模块 | 覆盖率 | 状态（达标/未达标） |
|------|--------|---------------------|
| sqlrustgo-parser | X% | ✅/❌ |
| sqlrustgo-executor | X% | ✅/❌ |
| sqlrustgo-storage | X% | ✅/❌ |
| sqlrustgo-planner | X% | ✅/❌ |
| sqlrustgo-optimizer | X% | ✅/❌ |
| sqlrustgo-transaction | X% | ✅/❌ |

---

## 7. 已知问题和限制

### 7.1 编译错误（2026-03-28）

以下代码存在编译错误，导致 `--workspace` 模式无法正常运行：

| 文件 | 问题 | 状态 |
|------|------|------|
| crates/optimizer/src/stats.rs | MockStorage 缺少 trait 方法 | ✅ 已修复 |
| crates/optimizer/src/stats.rs | ColumnDefinition 缺少字段 | ✅ 已修复 |
| crates/catalog/src/rebuild.rs | StorageColumn 缺少字段 | ✅ 已修复 |
| 其他测试文件 | 可能还有类似问题 | 🔶 待修复 |

### 7.2 临时解决方案

由于完整覆盖率测试存在编译问题，采用以下替代方案：

```bash
# 方案1：运行集成测试（不包含 lib tests）
cargo tarpaulin --ignore-panics --timeout 120 --out Html --output-dir ./coverage-report
```

方案1 会运行大部分集成测试，但不包括 lib tests 中的一些单元测试。

### 7.3 被排除的测试

以下测试因已知问题被标记为 ignore，不计入覆盖率：

```
tests/integration/upsert_test.rs:
  - test_replace_updates_existing_row (#[ignore])
  - test_insert_ignore_skips_duplicate (#[ignore])

tests/integration/foreign_key_test.rs:
  - test_fk_delete_restrict (#[ignore])
  - test_fk_delete_set_null (#[ignore])
  - test_fk_update_cascade (#[ignore])
  - test_fk_update_set_null (#[ignore])
  - test_fk_self_reference_delete_cascade (#[ignore])
  - test_fk_combined_actions (#[ignore])
```

---

## 8. 验收标准

### 8.1 通过条件

- [ ] 总覆盖率 ≥ 65%（历史基线）- 使用方案1
- [ ] 测试可正常运行（无 panic）
- [ ] 结果记录完整

### 8.2 失败处理

如果覆盖率低于目标：
1. 记录实际覆盖率
2. 分析覆盖率不足的模块
3. 创建 Issue 跟踪
4. 制定改进计划

---

## 9. 历史数据参考

### v1.9.0 发布时 (2026-03-27)

| 模块 | 覆盖率 |
|------|--------|
| sqlrustgo-parser | 88.64% |
| sqlrustgo-executor | 50.53% |
| sqlrustgo-storage | 57.62% |
| **总计** | **~70%** |

### 测试命令参考

```bash
cargo tarpaulin --workspace --ignore-panics --timeout 120
```

---

## 10. 附录：快速参考

### 运行覆盖率测试

```bash
cargo tarpaulin --workspace --ignore-panics --timeout 120 --out Html --output-dir ./coverage-report
```

### 查看报告

```bash
open ./coverage-report/tarpaulin-report.html
```

### 查看各模块覆盖率

```bash
grep -E "^\|\| .*:" ./coverage-report/tarpaulin-report.html | head -30
```