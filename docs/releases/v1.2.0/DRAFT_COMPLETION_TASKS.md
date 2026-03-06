# v1.2.0 开发收尾任务清单

> **版本**: v1.2.0-draft
> **创建日期**: 2026-03-06
> **创建人**: yinglichina8848
> **状态**: 🔴 待修复
> **负责人**: maintainer

---

## 一、当前状态摘要

### 1.1 版本进度

| 阶段 | 状态 | 说明 |
|------|------|------|
| 开发阶段 | ✅ 完成 | 核心功能已实现 |
| Draft 阶段 | 🔄 进行中 | 需修复代码质量问题 |
| Alpha 阶段 | ⏳ 待开始 | Draft 通过后进入 |
| Beta 阶段 | ⏳ 待开始 | 功能冻结 |
| RC 阶段 | ⏳ 待开始 | 发布候选 |
| GA 发布 | ⏳ 待开始 | 正式发布 |

### 1.2 门禁通过率

| 分类 | 通过 | 失败 | 未测试 | 通过率 |
|------|------|------|--------|--------|
| 代码质量 | 3 | 3 | 0 | 50% |
| 功能门禁 | 11 | 0 | 1 | 92% |
| 性能门禁 | 0 | 0 | 5 | 0% |
| 文档门禁 | 1 | 0 | 5 | 17% |
| **总计** | **15** | **3** | **11** | **52%** |

---

## 二、待修复问题详情

### 2.1 Clippy 错误 (15个 - 阻塞)

| # | 错误类型 | 文件位置 | 修复方法 |
|---|----------|----------|----------|
| 1 | `unused_import` | `src/executor/benchmark.rs:11` | 删除 `use super::*;` |
| 2 | `unused_mut` | `src/storage/buffer_pool.rs:396` | 移除 `mut` 关键字 |
| 3 | `unused_mut` | `src/storage/file_storage.rs:845` | 移除 `mut` 关键字 |
| 4 | `dead_code` | `src/transaction/wal.rs:305` | 删除或标记 `#[allow(dead_code)]` |
| 5 | `useless_comparison` | `src/monitoring/health.rs:237` | 移除无意义比较 `>= 0` |
| 6 | `map_or_simplify` | 多处 | 改用更简洁写法 |
| 7 | `approx_constant` | 多处 | 使用 `std::f64::consts::PI` |
| 8 | `approx_constant` | 多处 | 使用 `std::f64::consts::E` |
| 9 | `items_after_test_module` | 多处 | 移动测试模块位置 |
| 10 | `useless_vec` | 多处 | 简化 vec! 宏使用 |

### 2.2 测试编译错误 (1个 - 阻塞)

| # | 错误类型 | 文件 | 说明 |
|---|----------|------|------|
| 1 | `mismatched_types` | `tests/integration_test.rs` | API 签名不匹配 |

**详细说明**:
- `tests/integration_test.rs` 中使用了旧的 API
- 需要检查 `ExecutionEngine`, `TransactionManager`, `WriteAheadLog` 的当前接口
- 可能需要更新导入和方法调用

### 2.3 Benchmark 编译错误 (1个 - 阻塞)

| # | 错误类型 | 文件 | 说明 |
|---|----------|------|------|
| 1 | `mismatched_types` | `benches/executor_bench.rs` | API 签名不匹配 |

---

## 三、任务分配

### 3.1 maintainer 任务清单

#### 任务 A: 修复 Clippy 错误 (优先级: P0)

```bash
# 执行命令查看详细错误
cargo clippy --all-targets -- -D warnings 2>&1 | tee clippy_errors.txt
```

**修复步骤**:

1. **unused_import** - 删除未使用的导入
   ```rust
   // src/executor/benchmark.rs:11
   // 删除: use super::*;
   ```

2. **unused_mut** - 移除不必要的 mut
   ```rust
   // src/storage/buffer_pool.rs:396
   // let mut buf = ... 改为 let buf = ...
   
   // src/storage/file_storage.rs:845
   // let mut storage = ... 改为 let storage = ...
   ```

3. **dead_code** - 处理未使用代码
   ```rust
   // src/transaction/wal.rs:305
   // 选项1: 删除 test_wal_append 函数
   // 选项2: 添加 #[allow(dead_code)]
   ```

4. **useless_comparison** - 移除无意义比较
   ```rust
   // src/monitoring/health.rs:237
   // 移除 .as_millis() >= 0 的断言
   ```

5. **approx_constant** - 使用标准常量
   ```rust
   // 将 3.14159... 改为 std::f64::consts::PI
   // 将 2.71828... 改为 std::f64::consts::E
   ```

#### 任务 B: 修复测试编译错误 (优先级: P0)

```bash
# 查看测试编译错误
cargo test --no-run 2>&1 | grep "error\[E"
```

**修复步骤**:

1. 检查 `tests/integration_test.rs` 中的导入
2. 更新 API 调用以匹配当前接口
3. 确保所有测试函数签名正确

#### 任务 C: 修复 Benchmark 编译错误 (优先级: P1)

```bash
# 查看 benchmark 编译错误
cargo bench --no-run 2>&1 | grep "error\[E"
```

#### 任务 D: 清理 Warnings (优先级: P2)

```bash
# 查看所有警告
cargo build 2>&1 | grep "warning:"
```

---

## 四、验收标准

### 4.1 Draft 阶段通过标准

| 检查项 | 命令 | 要求 |
|--------|------|------|
| Clippy | `cargo clippy --all-targets -- -D warnings` | ✅ 零错误 |
| 测试编译 | `cargo test --no-run` | ✅ 编译通过 |
| 测试执行 | `cargo test` | ✅ 全部通过 |
| 格式化 | `cargo fmt --check` | ✅ 通过 |

### 4.2 完成后操作

完成上述任务后，执行以下操作：

```bash
# 1. 验证所有检查通过
cargo clippy --all-targets -- -D warnings
cargo test
cargo fmt --check

# 2. 更新门禁清单
# 编辑 docs/releases/v1.2.0/RELEASE_GATE_CHECKLIST.md

# 3. 提交修复
git add .
git commit -m "fix: resolve clippy errors and test compilation issues for v1.2.0-draft"

# 4. 推送到远程
git push origin develop-v1.2.0
```

---

## 五、时间要求

| 任务 | 预计时间 | 截止日期 |
|------|----------|----------|
| 任务 A (Clippy) | 2h | 2026-03-07 |
| 任务 B (测试) | 2h | 2026-03-07 |
| 任务 C (Benchmark) | 1h | 2026-03-07 |
| 任务 D (Warnings) | 1h | 2026-03-08 |
| **总计** | **6h** | **2026-03-08** |

---

## 六、风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| API 变更导致大量测试失败 | 高 | 逐文件修复，优先核心测试 |
| Clippy 错误涉及架构问题 | 中 | 评估是否需要重构或添加 allow |
| 时间不足 | 中 | 优先 P0 任务，P2 可延后 |

---

## 七、相关文档

- [VERSION_PLAN.md](./VERSION_PLAN.md) - 版本计划
- [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md) - 门禁清单
- [RELEASE_NOTES.md](./RELEASE_NOTES.md) - 发布说明

---

## 八、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-06 | 初始版本，创建任务清单 |

---

*本文档由 yinglichina8848 创建，分配给 maintainer 执行*
