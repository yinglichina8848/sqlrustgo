# [Epic-03] Benchmark 完善

## 概述

完善 Benchmark CLI 工具和 JSON 输出格式。

**资源占比**: 20%
**优先级**: P1
**状态**: ✅ 已完成

---

## Issues

### [BEN-01] Benchmark CLI 完善

**优先级**: P1
**工作量**: 100 行
**状态**: ✅ 已完成

**描述**: 完善 Benchmark CLI 工具

**Acceptance Criteria**:
- [x] `benchmark --help` 显示完整帮助
- [x] 支持 `--config` 配置文件
- [x] 支持 `--output` 输出路径

---

### [BEN-02] JSON 输出格式

**优先级**: P0
**工作量**: 50 行
**状态**: ✅ 已完成

**描述**: 实现 Benchmark 结果 JSON 格式输出

**Acceptance Criteria**:
- [x] `--output json` 生成正确 JSON
- [x] JSON 包含 timestamp, version, metrics
- [x] 符合 `BenchmarkResult` 结构

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
**状态**: ✅ 已完成

**描述**: 集成 PostgreSQL 对比基准测试

**Acceptance Criteria**:
- [x] 支持 `--pg-conn` PostgreSQL 连接
- [x] 自动运行对比测试
- [x] 输出对比报告

---

## 实现步骤

1. **CLI 增强** ✅
   - 添加 `--config` 参数
   - 添加 `--pg-conn` 参数

2. **JSON 序列化** ✅
   - 实现 `BenchmarkResult` Serialize
   - 生成标准 JSON 格式

3. **PostgreSQL 集成** ✅
   - 复用 `crates/bench/src/db/postgres.rs`
   - 实现对比报告生成

---

## 关键文件

| 文件 | 用途 |
|------|------|
| `crates/bench-cli/src/cli.rs` | CLI 参数定义 |
| `crates/bench-cli/src/main.rs` | CLI 主程序 + PostgreSQL 对比 |
| `crates/bench-cli/src/reporter.rs` | JSON 序列化 |
| `crates/bench/src/db/postgres.rs` | PostgreSQL 连接 |
| `crates/bench/src/lib.rs` | bench crate 公共接口 |
| `crates/bench/src/memory.rs` | 内存监控与限制 (10GB 上限) |
| `crates/bench/src/runner.rs` | 基准测试运行器 (含内存检查) |

---

## 内存保护机制

为防止基准测试无限占用内存导致系统不稳定，新增内存限制功能：

### 功能特性
- **默认限制**: 10GB 内存上限
- **定期检查**: 每 1000 次操作检查一次内存使用
- **高内存警告**: 内存使用超过 80% 时警告
- **超限停止**: 内存使用超过 10GB 时自动停止测试

### 日志输出示例
```
Memory limit: 10737418240 bytes (10 GB)
Initial memory usage: 0.05 GB / 10.00 GB limit (0.5%)
Starting workload: oltp with 4 threads for 60s
Final memory usage: 1.23 GB / 10.00 GB limit (12.3%)
```

### JSON 输出新增字段
```json
{
  "memory": {
    "limit_bytes": 10737418240,
    "final_usage": "1.23 GB / 10.00 GB limit (12.3%)"
  }
}
```

---

**关联 Issue**: BEN-01, BEN-02, BEN-03
**总工作量**: ~250 行 (+ ~150 行内存保护)
**提交**: `0e80eeb` feat(bench): EPIC-03 Benchmark 完善