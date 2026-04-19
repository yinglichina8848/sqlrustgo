# SQLRustGo v2.5.0 测试计划

**版本**: v2.5.0 (里程碑版本)
**发布日期**: 2026-04-16

---

## 一、测试策略

### 1.1 测试金字塔

```
                    ┌───────────┐
                    │   E2E     │  ← 少量，高价值
                   ─┴───────────┴─
                  ┌───────────────────┐
                  │   Integration     │  ← 覆盖关键路径
                 ─┴───────────────────┴─
                ┌───────────────────────────┐
                │       Unit Tests          │  ← 大量，快速反馈
               ─┴───────────────────────────┴─
```

### 1.2 测试目标

| 指标 | 目标 | 当前状态 |
|------|------|----------|
| 单元测试覆盖率 | ≥ 80% | 48.57% |
| 集成测试覆盖率 | ≥ 60% | 35% |
| 关键路径覆盖率 | 100% | 95% |
| 回归测试通过率 | 100% | 98% |

### 1.3 测试类型

| 类型 | 目的 | 工具 |
|------|------|------|
| 单元测试 | 验证单个函数/模块 | Rust test |
| 集成测试 | 验证组件交互 | Rust integration tests |
| 异常测试 | 验证边界和错误处理 | Custom frameworks |
| 压力测试 | 验证高负载下的稳定性 | Custom + sysbench |
| 性能测试 | 验证性能指标 | TPC-H, OLTP |

---

## 二、单元测试

### 2.1 模块划分

| Crate | 测试文件 | 目标覆盖率 |
|-------|----------|------------|
| parser | 316+ | 80% |
| catalog | 120+ | 75% |
| storage | 513+ | 85% |
| executor | 100+ | 80% |
| optimizer | 50+ | 70% |
| vector | 50+ | 75% |
| graph | 30+ | 70% |
| transaction | 40+ | 80% |

### 2.2 测试用例设计原则

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 1. 正常用例
    #[test]
    fn test_normal_case() {
        // Given
        let input = create_valid_input();

        // When
        let result = process(input);

        // Then
        assert!(result.is_ok());
    }

    // 2. 边界用例
    #[test]
    fn test_boundary_case() {
        // 空输入
        let result = process(vec![]);
        assert!(matches!(result, Err(Error::EmptyInput)));

        // 极大值
        let result = process(vec![u64::MAX]);
        assert!(result.is_ok());
    }

    // 3. 错误用例
    #[test]
    fn test_error_case() {
        let input = create_invalid_input();
        let result = process(input);
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }
}
```

---

## 三、集成测试

### 3.1 测试分类

| 测试组 | 位置 | 说明 |
|--------|------|------|
| SQL 功能 | `tests/integration/` | DML, DDL, DQL |
| 事务 | `tests/anomaly/` | MVCC, 并发 |
| 性能 | `tests/performance/` | 基准测试 |
| 压力 | `tests/stress/` | 长时间运行 |

### 3.2 关键集成测试

#### 外键约束测试

| 测试用例 | 描述 | 预期结果 |
|----------|------|----------|
| FK_INSERT_001 | 插入父表不存在的引用 | FAIL |
| FK_INSERT_002 | 插入有效的外键 | PASS |
| FK_DELETE_001 | 删除被引用的行 (CASCADE) | PASS |
| FK_DELETE_002 | 删除被引用的行 (RESTRICT) | FAIL |
| FK_UPDATE_001 | 更新引用到不存在的值 | FAIL |

#### MVCC 快照隔离测试

| 测试用例 | 描述 | 预期结果 |
|----------|------|----------|
| MVCC_READ_001 | 读不阻塞写 | PASS |
| MVCC_WRITE_001 | 写不阻塞读 | PASS |
| MVCC_SI_001 | 读已提交的事务开始时的快照 | PASS |
| MVCC_SI_002 | 写入冲突检测 | PASS |
| MVCC_SI_003 | 提交后新事务可见 | PASS |

#### 预处理语句测试

| 测试用例 | 描述 | 预期结果 |
|----------|------|----------|
| PREP_001 | PREPARE/EXECUTE | PASS |
| PREP_002 | 参数绑定 | PASS |
| PREP_003 | DEALLOCATE | PASS |
| PREP_004 | 多次执行 | PASS |

#### 子查询测试

| 测试用例 | 描述 | 预期结果 |
|----------|------|----------|
| SUBQ_001 | EXISTS 子查询 | PASS |
| SUBQ_002 | IN 子查询 | PASS |
| SUBQ_003 | ANY/ALL 子查询 | PASS |
| SUBQ_004 | 相关子查询 | PASS |

---

## 四、性能测试

### 4.1 TPC-H 测试

| 查询 | SF=1 目标 | SF=10 目标 | 状态 |
|------|-----------|------------|------|
| Q1 | < 500ms | < 5s | ✅ |
| Q2 | < 200ms | < 2s | ✅ |
| Q3 | < 300ms | < 3s | ✅ |
| Q4 | < 200ms | < 2s | ✅ |
| Q5 | < 400ms | < 4s | ✅ |
| ... | ... | ... | ... |
| 全部 | < 10s | < 100s | ✅ |

### 4.2 OLTP 测试

| 工作负载 | TPS 目标 | 并发 | 状态 |
|----------|----------|------|------|
| point_select | > 50,000 | 32 | ✅ |
| range_scan | > 10,000 | 32 | ✅ |
| insert | > 20,000 | 16 | ✅ |
| update_index | > 15,000 | 16 | ✅ |
| update_non_index | > 20,000 | 16 | ✅ |
| delete | > 15,000 | 16 | ✅ |
| mixed | > 10,000 | 32 | ✅ |

### 4.3 向量搜索测试

| 数据规模 | 召回率目标 | 延迟目标 | 状态 |
|----------|------------|----------|------|
| 10K | > 95% | < 5ms | ✅ |
| 100K | > 90% | < 10ms | ✅ |
| 1M | > 85% | < 20ms | ✅ |

---

## 五、异常测试

### 5.1 边界测试

| 测试类型 | 测试点 |
|----------|--------|
| 整数溢出 | u64::MAX + 1, i64::MIN - 1 |
| 除零 | 1 / 0, 1 % 0 |
| 空指针 | Option::None |
| 索引越界 | vec[0] on empty |
| 字符串编码 | UTF-8 边界字符 |
| 日期时间 | 闰年, 时区边界 |

### 5.2 崩溃恢复测试

| 测试用例 | 描述 | 验证点 |
|----------|------|--------|
| CRASH_001 | 正常关机后恢复 | 数据完整 |
| CRASH_002 | 事务中途崩溃 | WAL 重放 |
| CRASH_003 | 多次崩溃恢复 | 幂等性 |
| CRASH_004 | PITR 恢复 | 时间点准确 |

### 5.3 并发测试

| 测试用例 | 描述 | 验证点 |
|----------|------|--------|
| CONC_001 | 100 并发读 | 无死锁 |
| CONC_002 | 50 读 + 50 写 | MVCC 正确性 |
| CONC_003 | 死锁检测 | 正确回滚 |

---

## 六、回归测试

### 6.1 每日回归

```bash
# 运行每日回归测试
cargo test --test regression_test

# 检查覆盖率
cargo tarpaulin --workspace
```

### 6.2 发布前回归

```bash
# 1. 完整测试
cargo test --workspace

# 2. 性能基准
cargo bench --workspace

# 3. 文档检查
cargo doc --no-deps
```

---

## 七、测试执行

### 7.1 CI/CD 集成

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [main, develop/*]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: |
          cargo test --workspace
          cargo test --doc

      - name: Run coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --workspace

      - name: Run benches
        if: github.event_name == 'push'
        run: cargo bench --workspace
```

### 7.2 本地测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定 crate 测试
cargo test -p sqlrustgo-parser

# 运行集成测试
cargo test --test regression_test

# 运行性能测试
cargo test --test tpch_sf1_benchmark

# 生成覆盖率报告
cargo tarpaulin --workspace --out html
open tarpaulin-report.html
```

---

## 八、测试报告

### 8.1 报告内容

| 章节 | 内容 |
|------|------|
| 执行摘要 | 测试概述、结果统计 |
| 测试环境 | 硬件、软件配置 |
| 测试结果 | 通过/失败统计 |
| 覆盖率分析 | 代码覆盖详情 |
| 性能指标 | 基准测试结果 |
| 问题汇总 | 失败用例分析 |

### 8.2 报告模板

```markdown
# SQLRustGo v2.5.0 测试报告

## 执行摘要
- 测试日期: YYYY-MM-DD
- 测试版本: 2.5.0
- 测试总数: XXX
- 通过: XXX (XX%)
- 失败: XX (X%)
- 跳过: XX

## 测试环境
- OS: Linux 5.15.0
- CPU: 8 cores
- Memory: 32GB
- Rust: 1.75.0

## 结果统计
| 类型 | 通过 | 失败 | 总计 |
|------|------|------|------|
| 单元测试 | XX | X | XX |
| 集成测试 | XX | X | XX |
| 性能测试 | XX | X | XX |

## 覆盖率
- 行覆盖率: XX%
- 函数覆盖率: XX%
- 分支覆盖率: XX%

## 问题汇总
| ID | 严重性 | 描述 | 状态 |
|----|--------|------|------|
| ISS-001 | 高 | XXX | 已修复 |
| ISS-002 | 中 | XXX | 进行中 |
```

---

## 九、附录

### A. 测试命令速查

```bash
# 单元测试
cargo test --lib

# 文档测试
cargo test --doc

# 集成测试
cargo test --test '*'

# 所有测试
cargo test --workspace

# 覆盖率
cargo tarpaulin --workspace
```

### B. 测试数据

- TPC-H SF=1: `data/tpch-sf01/`
- TPC-H SF=10: `data/tpch-sf10/`

---

*测试计划 v2.5.0*
*最后更新: 2026-04-16*
