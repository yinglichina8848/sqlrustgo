# [Epic-03] Benchmark 完善

## 概述

完善 Benchmark CLI 工具和 JSON 输出格式。

**资源占比**: 20%
**优先级**: P1

---

## Issues

### [BEN-01] Benchmark CLI 完善

**优先级**: P1
**工作量**: 100 行

**描述**: 完善 Benchmark CLI 工具

**Acceptance Criteria**:
- [ ] `benchmark --help` 显示完整帮助
- [ ] 支持 `--config` 配置文件
- [ ] 支持 `--output` 输出路径

---

### [BEN-02] JSON 输出格式

**优先级**: P0
**工作量**: 50 行

**描述**: 实现 Benchmark 结果 JSON 格式输出

**Acceptance Criteria**:
- [ ] `--output json` 生成正确 JSON
- [ ] JSON 包含 timestamp, version, metrics
- [ ] 符合 `BenchmarkResult` 结构

**JSON 格式**:
```json
{
  "timestamp": "2026-03-21T10:00:00Z",
  "version": "1.7.0",
  "workload": "tpch",
  "metrics": {
    "qps": 125000,
    "p50_ms": 0.15,
    "p95_ms": 0.45,
    "p99_ms": 1.2
  }
}
```

---

### [BEN-03] PostgreSQL 对比测试

**优先级**: P1
**工作量**: 100 行

**描述**: 集成 PostgreSQL 对比基准测试

**Acceptance Criteria**:
- [ ] 支持 `--pg-conn` PostgreSQL 连接
- [ ] 自动运行对比测试
- [ ] 输出对比报告

---

## 实现步骤

1. **CLI 增强**
   - 添加 `--output json` 参数
   - 添加 `--pg-conn` 参数

2. **JSON 序列化**
   - 实现 `BenchmarkResult` Serialize
   - 生成标准 JSON 格式

3. **PostgreSQL 集成**
   - 复用 `crates/bench/src/db/postgres.rs`
   - 实现对比报告生成

---

## 关键文件

| 文件 | 用途 |
|------|------|
| `crates/bench-cli/src/cli.rs` | CLI 参数定义 |
| `crates/bench-cli/src/reporter.rs` | JSON 序列化 |
| `crates/bench/src/db/postgres.rs` | PostgreSQL 连接 |

---

**关联 Issue**: BEN-01, BEN-02, BEN-03
**总工作量**: ~250 行