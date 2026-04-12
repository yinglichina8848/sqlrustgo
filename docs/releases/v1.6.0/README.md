# SQLRustGo v1.6.0 文档入口

> **版本**: v1.6.0 GA
> **发布日期**: 2026-03-20

---

## 版本概述

v1.6.0 是 SQLRustGo 的**事务隔离版本**，在 v1.5.0 基础上实现了完整的并发事务控制能力。

### 核心特性

- **MVCC**: 多版本并发控制
- **事务隔离**: READ COMMITTED
- **行级锁**: 读写锁机制
- **死锁检测**: Wait-For Graph
- **WAL 改进**: 并发写入
- **索引增强**: 唯一/复合/全文索引
- **数据类型**: DATE、TIMESTAMP
- **性能优化**: 查询缓存、连接池、TPC-H Benchmark

---

## 文档索引

### 发布文档

| 文档 | 说明 |
|------|------|
| [RELEASE_NOTES.md](./RELEASE_NOTES.md) | 发布说明 |
| [CHANGE_LOG.md](./CHANGE_LOG.md) | 变更日志 |
| [VERSION_PLAN.md](./VERSION_PLAN.md) | 版本计划 |
| [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md) | 门禁清单 |

### 技术文档

| 文档 | 说明 |
|------|------|
| [ARCHITECTURE_DESIGN.md](./ARCHITECTURE_DESIGN.md) | 架构设计 |
| [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) | API 文档 |
| [USER_MANUAL.md](./USER_MANUAL.md) | 用户手册 |
| [TEST_MANUAL.md](./TEST_MANUAL.md) | 测试手册 |

### 质量文档

| 文档 | 说明 |
|------|------|
| [MATURITY_ASSESSMENT.md](./MATURITY_ASSESSMENT.md) | 成熟度评估 |
| [SECURITY_ANALYSIS.md](./SECURITY_ANALYSIS.md) | 安全分析 |
| [DEFECT_REPORT.md](./DEFECT_REPORT.md) | 缺陷报告 |

### 性能文档

| 文档 | 说明 |
|------|------|
| [PERFORMANCE_BENCHMARK_REPORT.md](./PERFORMANCE_BENCHMARK_REPORT.md) | 性能基准报告 |
| [PERFORMANCE_ANALYSIS_REPORT.md](./PERFORMANCE_ANALYSIS_REPORT.md) | 性能分析报告 |
| [PERFORMANCE_LIMIT_REPORT.md](./PERFORMANCE_LIMIT_REPORT.md) | 性能极限报告 |

---

## 快速开始

### 构建

```bash
cargo build --release
```

### 测试

```bash
cargo test --workspace
```

### 基准测试

```bash
cargo bench --bench tpch_bench
```

---

## 里程碑

| 日期 | 里程碑 |
|------|---------|
| 2026-03-18 | Craft 规划 |
| 2026-03-19 | Alpha 发布 |
| 2026-03-19 | Beta 发布 |
| 2026-03-19 | RC 发布 |
| 2026-03-20 | **GA 正式发布** |

---

## 相关链接

- [ROADMAP.md](../../ROADMAP.md) - 路线图
- [CHANGELOG.md](../../CHANGELOG.md) - 完整变更日志
- [GitHub Releases](https://github.com/minzuuniversity/sqlrustgo/releases)

---

*本文档由 AI 辅助生成*
*更新日期: 2026-03-20*
