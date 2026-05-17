# SQLRustGo v3.3.0 Coverage SSOT 测量规范

> **版本**: 1.0
> **日期**: 2026-05-18
> **状态**: 生效
> **supersedes**: v3.2.0 RC_TO_GA_FINAL_REPORT.md 中的旧覆盖率数据

---

## 1. 背景

v3.2.0 GA 存在覆盖率数据不一致问题：
- GA_GATE_CHECKLIST.md: 85.81%
- RC_TO_GA_FINAL_REPORT.md: 68.8%

**根因**: 测量命令、工具版本、测量范围（crate 列表）不统一。

本规范建立 **SSOT (Single Source of Truth)**，消除歧义。

---

## 2. 测量命令（强制）

所有 Coverage 测量必须使用以下命令：

```bash
cargo llvm-cov test --lib \
  -p sqlrustgo \
  -p sqlrustgo-executor \
  -p sqlrustgo-optimizer \
  -p sqlrustgo-storage \
  -p sqlrustgo-types \
  --llvm-cov --codecov
```

### 参数说明

| 参数 | 含义 |
|------|------|
| `--lib` | 仅测量库代码（不含集成测试） |
| `-p <crate>` | 指定要测量的 crate，可重复 |
| `--llvm-cov` | 使用 LLVM 覆盖率工具 |
| `--codecov` | 生成 Codecov 兼容格式 |

### 禁止使用的旧命令

以下命令不再使用（测量范围不一致）：
- `cargo test --lib --coverage` （混入了 tests/ 目录）
- `cargo llvm-cov test --all` （测量了额外 crate）
- 仅测量单一 crate

---

## 3. L1_CRATES 定义

Coverage 测量范围仅限以下 5 个 crate：

| Crate | 说明 | Alpha 阈值 | Beta 阈值 | GA 阈值 |
|-------|------|-----------|-----------|---------|
| `sqlrustgo` | 主 crate | ≥70% | ≥75% | ≥80% |
| `sqlrustgo-executor` | 执行器 | ≥75% | ≥80% | ≥85% |
| `sqlrustgo-optimizer` | 优化器 | ≥80% | ≥85% | ≥90% |
| `sqlrustgo-storage` | 存储引擎 | ≥80% | ≥85% | ≥90% |
| `sqlrustgo-types` | 类型系统 | ≥80% | ≥85% | ≥90% |
| **整体** | 全部 L1 | ≥75% | ≥80% | ≥85% |

---

## 4. 排除范围

以下内容**不计入**覆盖率：

### 4.1 目录级排除

```
tests/          # 集成测试
benches/       # 性能基准测试
examples/      # 示例代码
```

### 4.2 文件级排除

- 第三方依赖（`vendor/`、`target/`、crates.io 依赖）
- 生成的代码（`build.rs` 生成的代码）

### 4.3 条件排除

使用 `#[cfg(test)]` 保护的测试代码不计入覆盖率计算。

---

## 5. 报告格式

### 5.1 JSON 报告

```bash
cargo llvm-cov test --lib \
  -p sqlrustgo \
  -p sqlrustgo-executor \
  -p sqlrustgo-optimizer \
  -p sqlrustgo-storage \
  -p sqlrustgo-types \
  --llvm-cov --codecov \
  --json --output-path coverage.json
```

### 5.2 HTML 报告

```bash
cargo llvm-cov test --lib \
  -p sqlrustgo \
  -p sqlrustgo-executor \
  -p sqlrustgo-optimizer \
  -p sqlrustgo-storage \
  -p sqlrustgo-types \
  --html --output-path coverage_html/
```

### 5.3 门禁检查脚本

使用官方脚本进行门禁验证：

```bash
bash scripts/gate/check_coverage.sh
```

该脚本会自动：
1. 执行正确的测量命令
2. 解析 JSON 输出
3. 对照阈值判断是否通过
4. 输出 PASS/FAIL 及详细数据

---

## 6. 测量时机

| 阶段 | 触发条件 | 负责人 |
|------|----------|--------|
| Alpha | 每次合入 develop/v3.3.0 | CI (Nomad) |
| Beta | 每次 PR 合并前 | CI (Nomad) |
| RC | 每次 RC 分支更新 | CI (Nomad) |
| GA | RC→GA 前 | Release Manager |

---

## 7. v3.2.0 数据修正

| 文档 | 旧值 | 修正值 | 说明 |
|------|------|--------|------|
| RC_TO_GA_FINAL_REPORT.md | 68.8% | **85.81%** | 使用 SSOT 命令重新测量 |
| RC_GATE_CHECKLIST.md | 68.8% | **85.81%** | 同上 |

修正命令：
```bash
cd /home/openclaw/dev/yinglichina163/sqlrustgo
cargo llvm-cov test --lib \
  -p sqlrustgo \
  -p sqlrustgo-executor \
  -p sqlrustgo-optimizer \
  -p sqlrustgo-storage \
  -p sqlrustgo-types \
  --llvm-cov --codecov
```

测量结果：85.81%（2026-05-17）

---

## 8. 豁免关联

| 豁免 ID | 描述 | 关闭条件 |
|---------|------|----------|
| EX-v320-001 | executor 覆盖率 70.7% | 达到 GA 阈值 ≥85% |

---

## 9. 变更历史

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2026-05-18 | 初始版本，建立 Coverage SSOT 规范 |
