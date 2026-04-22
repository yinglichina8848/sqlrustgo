# v2.7.0 性能评估报告

> 版本: `v2.7.0`  
> 日期: 2026-05-XX  
> 基准commit: `TBD`  
> 环境: macOS M2 Pro / Linux (待填写)

---

## 1. 执行摘要

| 指标 | v2.6.0 基线 | v2.7.0 目标 | 状态 |
|------|-------------|-------------|------|
| OLTP QPS | >= 50,000 | >= v2.6.0 (不退化) | ⏳ |
| TPC-H SF1 | 100% | 100% | ⏳ |
| 延迟 P95 | < 50ms | < 50ms | ⏳ |
| 延迟 P99 | < 100ms | < 100ms | ⏳ |
| 向量检索 P99 | < 100ms (10k 数据) | < 100ms | ⏳ |
| 图谱查询 P99 | < 200ms | < 200ms | ⏳ |

---

## 2. 测试环境

### 2.1 硬件配置
- CPU: Apple M2 Pro (12核)
- 内存: 32GB
- 磁盘: 512GB SSD

### 2.2 软件配置
- 操作系统: macOS 14.4
- Rust: 1.77.0
- 数据库版本: SQLRustGo v2.7.0

---

## 3. Sysbench OLTP 测试

### 3.1 oltp_point_select
```
命令: sysbench oltp_point_select ...
QPS: TBD
延迟 P95: TBD
延迟 P99: TBD
```

### 3.2 oltp_read_only
```
命令: sysbench oltp_read_only ...
QPS: TBD
延迟 P95: TBD
延迟 P99: TBD
```

### 3.3 oltp_read_write
```
命令: sysbench oltp_read_write ...
QPS: TBD
延迟 P95: TBD
延迟 P99: TBD
```

---

## 4. TPC-H 测试 (SF=1)

| Query | 状态 | 执行时间 | 备注 |
|-------|------|----------|------|
| Q1 | ⏳ | - | |
| Q2 | ⏳ | - | |
| Q3 | ⏳ | - | |
| Q4 | ⏳ | - | |
| Q5 | ⏳ | - | |
| Q6 | ⏳ | - | |
| Q7 | ⏳ | - | |
| Q8 | ⏳ | - | |
| Q9 | ⏳ | - | |
| Q10 | ⏳ | - | |
| Q11 | ⏳ | - | |
| Q12 | ⏳ | - | |
| Q13 | ⏳ | - | |
| Q14 | ⏳ | - | |
| Q15 | ⏳ | - | |
| Q16 | ⏳ | - | |
| Q17 | ⏳ | - | |
| Q18 | ⏳ | - | |
| Q19 | ⏳ | - | |
| Q20 | ⏳ | - | |
| Q21 | ⏳ | - | |
| Q22 | ⏳ | - | |

**通过率**: X/22 (X%)

---

## 5. 与 v2.6.0 对比

| 场景 | v2.6.0 | v2.7.0 | 变化 |
|------|---------|---------|------|
| OLTP QPS | TBD | TBD | TBD |
| TPC-H SF1 | TBD | TBD | TBD |

---

## 6. 结论与风险

### 6.1 结论
- [ ] 性能达到目标
- [ ] 性能未退化

### 6.2 风险
1. TBD

---

## 7. 附录

### 7.1 执行命令
```bash
# Sysbench
sysbench ...

# TPC-H
cargo run --bin tpch -- sf=1
```

### 7.2 原始结果
- 路径: `artifacts/performance/`
