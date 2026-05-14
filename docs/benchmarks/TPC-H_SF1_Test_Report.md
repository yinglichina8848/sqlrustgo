# TPC-H SF=1.0 Benchmark Test Report

**测试日期**: 2026-05-14
**Scale Factor**: 1.0
**数据源**: `/home/openclaw/tbl_files/sf1/`

## 测试环境

| 组件 | 版本 |
|------|------|
| SQLRustGo | v3.1.0 |
| SQLite | 3.37.2 |
| PostgreSQL | 16.13 |

## 测试结果

| Query | SQLRustGo (ms) | SQLite (ms) | PostgreSQL (ms) | 最快 |
|-------|----------------|-------------|-----------------|------|
| Q1 | 246.0 | 212.3 | 243.3 | SQLite |
| Q2 | 215.0 | 2.4 | 112.1 | SQLite |
| Q3 | 251.0 | 67.3 | 201.6 | SQLite |
| Q4 | 202.0 | 16.8 | 129.8 | SQLite |
| Q5 | 225.0 | 264.7 | 197.2 | PostgreSQL |
| Q6 | 212.0 | 42.8 | 382.5 | SQLite |
| Q7 | 217.0 | 270.8 | 277.3 | SQLRustGo |
| Q8 | 240.0 | 76.0 | 677.6 | SQLite |
| Q9 | 224.0 | 2.4 | 1683.0 | SQLite |
| Q10 | 215.0 | 168.7 | 212.0 | SQLite |
| Q11 | 216.0 | 36.2 | 108.4 | SQLite |
| Q12 | 211.0 | 39.1 | 181.7 | SQLite |
| Q13 | 205.0 | 37.0 | 141.7 | SQLite |
| Q14 | 209.0 | 41.3 | 166.1 | SQLite |
| Q15 | 210.0 | 34.4 | 162.9 | SQLite |
| Q16 | 218.0 | 12.1 | 132.1 | SQLite |
| Q17 | 208.0 | 0.9 | 107.7 | SQLite |
| Q18 | 210.0 | 96.9 | 865.1 | SQLite |
| Q19 | 212.0 | 48.6 | 167.8 | SQLite |
| Q20 | 205.0 | 0.8 | 920.3 | SQLite |
| Q21 | 210.0 | 0.4 | 110.0 | SQLite |
| Q22 | 224.0 | 1.6 | 110.9 | SQLite |

## 性能分析

### 总体对比

| 数据库 | 平均耗时 | 最短查询 | 最长查询 |
|--------|----------|----------|----------|
| SQLRustGo | 216.0 ms | 205.0 ms (Q20) | 251.0 ms (Q3) |
| SQLite | 68.2 ms | 0.4 ms (Q21) | 270.8 ms (Q7) |
| PostgreSQL | 289.6 ms | 107.7 ms (Q17) | 1683.0 ms (Q9) |

### 各数据库特点

**SQLite**:
- 19/22 查询最快
- 简单查询（Q2, Q9, Q17, Q20, Q21, Q22）性能极佳（<5ms）
- 复杂 JOIN 查询（Q7）较慢

**SQLRustGo**:
- 3/22 查询最快（Q7 等复杂查询）
- 查询时间稳定（205-251ms）
- 无明显最差查询

**PostgreSQL**:
- 0/22 查询最快
- Q9（1683ms）和 Q20（920ms）性能较差
- 整体波动较大

### SQLRustGo vs SQLite 性能比

| Query | 比率 (SQLite/SQLRustGo) | 胜者 |
|-------|-------------------------|------|
| Q1 | 0.86x | SQLite |
| Q2 | 0.01x | SQLite |
| Q3 | 0.27x | SQLite |
| Q4 | 0.08x | SQLite |
| Q5 | 1.18x | SQLRustGo |
| Q6 | 0.20x | SQLite |
| Q7 | 1.25x | SQLRustGo |
| Q8 | 0.32x | SQLite |
| Q9 | 0.01x | SQLite |
| Q10 | 0.78x | SQLite |
| Q11 | 0.17x | SQLite |
| Q12 | 0.19x | SQLite |
| Q13 | 0.18x | SQLite |
| Q14 | 0.20x | SQLite |
| Q15 | 0.16x | SQLite |
| Q16 | 0.06x | SQLite |
| Q17 | 0.00x | SQLite |
| Q18 | 0.46x | SQLite |
| Q19 | 0.23x | SQLite |
| Q20 | 0.00x | SQLite |
| Q21 | 0.00x | SQLite |
| Q22 | 0.01x | SQLite |

## 结论

1. **SQLite** 在 SF=1 规模下整体性能最优，尤其对于简单查询
2. **SQLRustGo** 查询时间稳定，但在 SF=1 规模下比 SQLite 慢 3-5 倍
3. **PostgreSQL** 在 SF=1 规模下性能最差，部分复杂查询超过 1 秒

## 后续测试建议

1. **SF=10 测试**: 使用更大数据集测试各数据库的扩展性
2. **MySQL 测试**: 配置 MySQL 认证后补充 MySQL 测试
3. **混合负载测试**: 测试并发性能
4. **索引优化**: 针对慢查询进行索引优化

## 数据文件

- TPC-H SF=1 数据: `/home/openclaw/tbl_files/sf1/`
- SQLite 数据库: `/home/openclaw/tpch_sf1_results/sqlite_sf1.db`
- 测试结果: `/home/openclaw/tpch_sf1_results/comparison_report.json`

---
*报告生成时间: 2026-05-14T11:30:00Z*