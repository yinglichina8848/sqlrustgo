# v2.9.0 测试策略与阶段演化设计

> 5个阶段: develop → alpha → beta → rc → ga
> 核心原则: 所有功能开发在开发阶段完成, Alpha起仅做集成+修复

## 阶段测试矩阵

| 阶段 | 测试内容 | 门禁 |
|------|---------|------|
| develop | cargo test --lib, clippy, fmt, 回归 | BLOCK |
| alpha | 全量测试, 集成 28项, SQL Corpus ≥85% | BLOCK |
| beta | TPC-H ≥18/22, Sysbench ≥5K, 安全 100% | BLOCK |
| rc | TPC-H 全量, Sysbench ≥10K, P011-P012 | BLOCK |
| ga | 混沌工程, 覆盖率≥85%, 全部前序 | BLOCK |

## 教训(v2.7.0/2.8.0)

- 集成测试拖到RC才发现问题 → 分布式到develop+alpha
- SQL Corpus 未纳入CI门禁 → Alpha强制≥85%
- 性能无基线 → Beta建立sysbench+TPC-H基线
- 覆盖率未强制执行 → 各阶段递进门禁
