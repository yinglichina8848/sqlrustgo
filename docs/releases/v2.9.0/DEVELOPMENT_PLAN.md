# v2.9.0 版本定义与开发计划

> **版本**: v2.9.0
> **代号**: Trusted Governance & SQL Proof
> **基线**: v2.8.0 GA (release/v2.8.0)
> **目标**: 全面可信任治理 + SQL 数据库系统证明

---

## 1. v2.8.0 现状评估

| 指标 | v2.8.0 实际 | 评估 |
|------|-------------|------|
| 单元测试 | 258 PASS, 0 FAIL, 33 ignored | ⚠️ 需消除 ignore |
| 分布式测试 | 685 PASS (100%) | ✅ |
| 安全测试 | 81 PASS (100%) | ✅ |
| SQL Corpus | 174/426 (40.8%) | 🔴 需提升 |
| MySQL 5.7 替代度 | 45.5/100 | 🔴 差距大 |
| GMP 适用度 | 65/100 | ⚠️ |
| 生产部署 | 0 | 🔴 |
| 性能基准 | 无 sysbench | 🔴 |

## 2. 量化目标

| 指标 | v2.8.0 | v2.9.0 目标 | 评估方法 |
|------|--------|-------------|---------|
| SQL Corpus 通过率 | 40.8% | **≥ 80%** | cargo test -p sql-corpus |
| Ignored 测试 | 33 | **0** | cargo test --all-features |
| 安全性评分 | 75% | **≥ 90%** | 安全审计矩阵 |
| 性能基准 | 无 | **sysbench OLTP ≥ 10K QPS** | sysbench run |
| GRANT/REVOKE | ❌ | **✅** | parser tests |
| AES-256 集成 | feature-gated | **✅ 存储集成** | encryption tests |
| 角色管理 | ❌ | **✅** | parser tests |
| 门禁规则 | R1-R7 | **R1-R10** | gate check |
| 形式化证明 | 0 | **≥ 10 条** | proof registry |

## 3. 任务矩阵

### Phase G: 可信任治理 (Trusted Governance)
| Task | 功能 | 优先级 | 工时 |
|------|------|--------|------|
| G-01 | R门禁扩展 (R8-R10) | P0 | 3d |
| G-02 | 证明注册表系统升级 (Formulog/Dafny) | P0 | 5d |
| G-03 | 攻击面验证 AV10 | P0 | 5d |
| G-04 | 治理规则自动化 CI/CD | P1 | 3d |
| G-05 | 门禁违规告警 (Webhook) | P1 | 2d |

### Phase S: SQL 可证明性 (SQL Proof)
| Task | 功能 | 优先级 | 工时 |
|------|------|--------|------|
| S-01 | Parser 正确性证明 (Formulog) | P0 | 8d |
| S-02 | 类型系统安全性证明 (Dafny) | P0 | 8d |
| S-03 | 事务 ACID 性质证明 (TLA+) | P0 | 10d |
| S-04 | B+Tree 不变式证明 (Dafny) | P1 | 10d |
| S-05 | 查询等价性证明框架 | P1 | 8d |

### Phase C: SQL 兼容性 (Compatibility)
| Task | 功能 | 优先级 | 工时 |
|------|------|--------|------|
| C-01 | SQL Corpus 40.8% → 80% | P0 | 15d |
| C-02 | CTE (WITH/Recursive) | P0 | 5d |
| C-03 | JSON 函数 (JSON_EXTRACT/OBJECT) | P1 | 5d |
| C-04 | 窗口函数补全 (LEAD/LAG/NTILE) | P1 | 3d |
| C-05 | GROUPING SETS/CUBE/ROLLUP | P2 | 5d |
| C-06 | 子查询去关联优化 | P1 | 5d |

### Phase D: 分布式增强 (Distributed)
| Task | 功能 | 优先级 | 工时 |
|------|------|--------|------|
| D-01 | 半同步复制完善 | P1 | 3d |
| D-02 | 并行复制 (MTS) | P1 | 5d |
| D-03 | 多源复制 | P2 | 8d |
| D-04 | 2PC 强化 | P1 | 5d |

### Phase E: 生产就绪 (Enterprise)
| Task | 功能 | 优先级 | 工时 |
|------|------|--------|------|
| E-01 | Sysbench OLTP 基准 | P0 | 5d |
| E-02 | 慢查询日志 | P1 | 3d |
| E-03 | 消除 33 个 #[ignore] 测试 | P0 | 5d |
| E-04 | GRANT/REVOKE 列级权限 | P0 | 5d |
| E-05 | 角色管理 | P1 | 3d |
| E-06 | AES-256 存储加密 | P1 | 5d |
| E-07 | 连接池 | P2 | 5d |
| E-08 | 二级索引端到端验证 | P0 | 3d |

## 4. 里程碑
```
Month 1 (05/2026) — Governance & Proof
  G-01~G-05, S-01~S-02, C-01 (40.8%→60%)
Month 2 (06/2026) — SQL 兼容性 + 生产化
  C-01~C-04 (60%→80%), D-01~D-02, E-01~E-03, S-03
Month 3 (07/2026) — 安全 + 稳定性
  E-04~E-08, D-03~D-04, C-05~C-06, S-04~S-05
08/15: v2.9.0 GA
```

## 5. 门禁标准
| 门禁 | 要求 |
|------|------|
| G-Gate | R1-R10 通过, AV1-AV10 无 CRITICAL |
| P-Gate | ≥ 10 条形式化证明 |
| T-Gate | Corpus ≥ 80%, 0 ignored, sysbench ≥ 10K QPS |
| S-Gate | GRANT/REVOKE, 加密, 角色 |
| R-Gate | 以上全通过 → GA |

## 6. 初始证明清单
| ID | 陈述 | 语言 |
|----|------|------|
| PROOF-001 | SQL SELECT 解析不丢失信息 | Formulog |
| PROOF-002 | 类型推断终止且唯一 | Dafny |
| PROOF-003 | WAL 重放后 = 崩溃前已提交 | TLA+ |
| PROOF-004 | B+Tree 查询返回所有匹配行 | Dafny |
| PROOF-005 | MVCC 快照读一致性 | TLA+ |

## 7. 相关 Issues
- Issue #116: G-01~G-05 可信任治理体系
- Issue #117: S-01~S-05 SQL 可证明性
- Issue #118: C-01~C-06 SQL 兼容性
- Issue #119: D-01~D-04 分布式增强
- Issue #120: E-01~E-08 生产就绪

---

*最后更新: 2026-05-02*
