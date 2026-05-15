# SQLRustGo v3.2.0 开发计划

> **版本**: v1.0
> **日期**: 2026-05-15
> **维护人**: hermes-z6g4
> **基于**: v320_STRATEGIC_DEVELOPMENT_GUIDE.md + v320_ISSUE_SUMMARY.md

---

## 一、版本概述

### 1.1 战略定位

```
v3.2.0 = "Trusted GMP Data Platform"

从 "MySQL 替代品" → "GMP Native 可信数据平台"
```

### 1.2 核心目标

| 目标类别 | 目标 | 成功指标 |
|----------|------|----------|
| **GMP Native** | 建立可信审计底座 | 数字签名 + 电子签名 + Immutable |
| **性能优化** | MySQL 协议 30K+ QPS | Sysbench point_select ≥ 30K |
| **技术修复** | P0 瓶颈全部修复 | B+Tree/Sort/Limit/SSI |
| **MySQL 兼容** | SQL 兼容性 85% | 窗口函数/多表DML/HASH JOIN |

### 1.3 版本范围

```
v3.2.0 范围：
├── P0: GMP Native 核心能力
├── P0: 技术瓶颈修复
├── P1: GMP 平台能力
└── P1: MySQL 兼容性提升
```

---

## 二、问题 → 开发任务映射

### 2.1 P0 GMP Native 核心能力

| Task ID | 问题 | 解决方案 | 工作量 | 依赖 |
|----------|------|----------|--------|------|
| **GMP-1** | 数字签名审计链 | 实现 ECDSA-P256 签名 + 验签 | L | GMP-7 |
| **GMP-2** | 电子签名 | 实现私钥签名 + 理由 + 时间戳 | M | GMP-1 |
| **GMP-3** | 双人复核 | CREATE APPROVAL POLICY | M | GMP-2 |
| **GMP-4** | Immutable Record | CREATE TABLE ... IMMUTABLE | M | GMP-1 |
| **GMP-5** | Correction Chain | CORRECT RECORD ... 语法 | M | GMP-4 |
| **GMP-6** | Provenance Tracking | data_provenance 表 + 链式哈希 | L | GMP-1 |
| **GMP-7** | Trusted Timestamp | RFC3161 TSA 集成 | M | - |
| **GMP-8** | HSM/KMS 集成 | TPM/HSM/KMS 接口 | L | - |

### 2.2 P0 技术瓶颈修复

| Task ID | 问题 | 解决方案 | 工作量 | 依赖 |
|----------|------|----------|--------|------|
| **PERF-1** | MySQL 协议 109x | 移除强制 flush + 缓存 uppercase | M | - |
| **PERF-2** | CBO 未激活 | CostConstants 实现 + 统计信息 | L | OO-3 |
| **PERF-3** | B+Tree 分裂不传播 | 修复分裂算法 | M | OO 文档 |
| **PERF-4** | Sort/Limit 未实现 | 实现 Sort/Limit 算子 | M | - |
| **PERF-5** | SSI 死锁检测 | wait-for graph + 偏序检测 | L | OO-4 |

### 2.3 P1 GMP 平台能力

| Task ID | 问题 | 解决方案 | 工作量 | 依赖 |
|----------|------|----------|--------|------|
| **GMP-9** | GMP Workflow | 状态机引擎 (偏差/CAPA/审批) | L | GMP-1~6 |
| **GMP-10** | 移动端采集 | 可信采集协议 + 设备签名 | M | GMP-1~6 |
| **GMP-11** | SOP/培训绑定 | 执行前培训检查 | M | GMP-9 |
| **GMP-12** | 设备校准 | calibration 状态检查 | M | - |

### 2.4 P1 MySQL 兼容性

| Task ID | 问题 | 解决方案 | 工作量 | 依赖 |
|----------|------|----------|--------|------|
| **SQL-1** | 窗口函数缺失 | 实现 NTILE/LEAD/LAG | M | OO 文档 |
| **SQL-2** | 多表 DML | 多表 UPDATE/DELETE | L | MVCC 增强 |
| **SQL-3** | HASH JOIN | HashJoin 算子 | L | CBO |
| **SQL-4** | MERGE | MERGE INTO 实现 | L | SQL-3 |
| **SQL-5** | Event Scheduler | MySQL Event 兼容 | M | - |

### 2.5 P1 OO 文档补全

| Task ID | 问题 | 解决方案 | 工作量 | 依赖 |
|----------|------|----------|--------|------|
| **OO-1** | TLA+ MVCC | 创建 MVCC TLA+ 规格 | M | - |
| **OO-2** | WAL + 审计链 | 创建 WAL chaos 测试文档 | M | - |
| **OO-3** | CBO 集成 | 创建 CBO_INTEGRATION.md | S | PERF-2 |
| **OO-4** | SSI 死锁 | 创建 SERIALIZABLE_SSI.md | S | PERF-5 |
| **OO-5** | Buffer Pool | 创建 BUFFER_POOL_LRU.md | S | - |
| **OO-6** | 向量化 | 创建 ITERATOR_MODEL.md | S | - |

---

## 三、开发里程碑

### 3.1 里程碑总览

| Milestone | 日期 | 交付内容 |
|-----------|------|----------|
| **M1** | Week 2 | GMP-7 (HSM/KMS) + PERF-1 (MySQL协议) |
| **M2** | Week 4 | GMP-1 (数字签名) + OO-1~2 (TLA+文档) |
| **M3** | Week 6 | GMP-2~4 (电子签名/双人复核/Immutable) |
| **M4** | Week 8 | GMP-5~6 (Correction/Provenance) + PERF-2~3 |
| **M5** | Week 10 | PERF-4~5 (Sort/Limit/SSI) + SQL-1 (窗口函数) |
| **M6** | Week 12 | SQL-2~4 (多表DML/HASH JOIN/MERGE) |
| **M7** | Week 14 | GMP-9~12 (Workflow/移动端/SOP/设备) |
| **M8** | Week 16 | SQL-5 (Event) + OO-3~6 (文档补全) |
| **RC1** | Week 18 | Beta Release + 门禁检查 |
| **GA** | Week 20 | GA Release |

### 3.2 详细里程碑

```
Week 1: 准备
├── 架构设计评审
├── HSM/KMS 接口设计
├── MySQL 协议优化方案
└── TLA+ 规格起草

Week 2: M1 - HSM + MySQL协议
├── [GMP-7] HSM/KMS 接口框架
├── [PERF-1] MySQL 协议 flush 优化
├── [PERF-1] uppercase 缓存
└── 验证: Sysbench point_select ≥ 5K QPS

Week 3: M2 准备
├── [GMP-7] TPM 集成
├── [GMP-7] ECDSA 签名实现
└── [OO-1] TLA+ MVCC 规格

Week 4: M2 - 数字签名
├── [GMP-1] 数字签名审计链
├── [GMP-1] 验签接口
├── [OO-2] WAL + 审计链文档
└── 验证: 签名/验签单元测试通过

Week 5: M3 准备
├── [GMP-2] 电子签名表结构
└── [GMP-2] 签名策略引擎

Week 6: M3 - 电子签名/双人复核/Immutable
├── [GMP-2] 电子签名 CREAT TABLE
├── [GMP-3] 双人复核 APPROVAL POLICY
├── [GMP-4] Immutable Record
└── 验证: 电子签名集成测试通过

Week 7: M4 准备
├── [GMP-5] Correction Chain 语法
└── [GMP-6] Provenance Tracking 表

Week 8: M4 - Correction/Provenance + 性能修复
├── [GMP-5] CORRECT RECORD 实现
├── [GMP-6] Provenance 链式哈希
├── [PERF-2] CBO 代价模型激活
├── [PERF-3] B+Tree 分裂修复
└── 验证: CBO 优化效果显著

Week 9: M5 准备
├── [PERF-4] Sort 算子
└── [PERF-5] Limit 算子

Week 10: M5 - Sort/Limit/SSI + 窗口函数
├── [PERF-4] Sort/Limit 执行
├── [PERF-5] SSI 死锁检测
├── [SQL-1] NTILE 实现
├── [SQL-1] LEAD/LAG 实现
└── 验证: ORDER BY/LIMIT 正常工作

Week 11: M6 准备
├── [SQL-2] 多表 UPDATE
└── [SQL-2] 多表 DELETE

Week 12: M6 - 多表DML/HASH JOIN/MERGE
├── [SQL-2] 多表 DML 完整实现
├── [SQL-3] HashJoin 算子
├── [SQL-4] MERGE INTO
└── 验证: TPC-H SF=1 22/22 + Sysbench ≥ 15K

Week 13: M7 准备
├── [GMP-9] Workflow 状态机
└── [GMP-10] 移动端采集协议

Week 14: M7 - Workflow/移动端/SOP/设备
├── [GMP-9] GMP Workflow Engine
├── [GMP-10] 移动端可信采集
├── [GMP-11] SOP/培训绑定
├── [GMP-12] 设备校准管理
└── 验证: GMP 场景测试通过

Week 15: M8 准备
└── [SQL-5] Event Scheduler

Week 16: M8 - Event + OO文档
├── [SQL-5] Event Scheduler 实现
├── [OO-3] CBO_INTEGRATION.md
├── [OO-4] SERIALIZABLE_SSI.md
├── [OO-5] BUFFER_POOL_LRU.md
├── [OO-6] ITERATOR_MODEL.md
└── 验证: 所有 OO 文档存在

Week 17: 集成测试
├── 全量集成测试
├── GMP 合规验证
└── 性能回归测试

Week 18: RC1 - Beta Release
├── [x] Alpha Gate
├── [ ] Beta Gate
├── [ ] RC Gate
└── 发布 Beta

Week 19: 修复
├── Beta 问题修复
└── RC 问题修复

Week 20: GA - Final Release
├── [ ] GA Gate
├── [ ] 文档完善
└── 发布 GA
```

---

## 四、测试计划

### 4.1 测试分层

```
测试金字塔：
         ▲
        /█\      GA Gate (完整验证)
       /███\
      /█████\    RC Gate (全量功能)
     /███████\   Beta Gate (核心功能)
    /█████████\  Alpha Gate (基础功能)
   /███████████\
  └─────────────────────────────────
```

### 4.2 Alpha Gate (Week 2)

| 测试 | 验证内容 | 通过标准 |
|------|----------|----------|
| A1 Build | 编译通过 | cargo build --all-features |
| A2 Test | 单元测试 | cargo test --lib ≥ 90% |
| A3 Clippy | 零警告 | cargo clippy -- -D warnings |
| A4 Format | 格式检查 | cargo fmt --check |
| A5 Coverage | 覆盖率 | ≥ 75% |
| A6 HSM | 密钥接口 | 单元测试通过 |

### 4.3 Beta Gate (Week 12)

| 测试 | 验证内容 | 通过标准 |
|------|----------|----------|
| B1 Build | 编译通过 | cargo build --release |
| B2 Test | 功能测试 | cargo test --lib ≥ 90% |
| B3 Clippy | 零警告 | cargo clippy -- -D warnings |
| B4 Coverage | 覆盖率 | ≥ 80% |
| B5 SQL Compat | 窗口函数 | NTILE/LEAD/LAG 测试通过 |
| B6 SQL Compat | 多表 DML | UPDATE/DELETE 多表通过 |
| B7 Sysbench | MySQL 协议 | point_select ≥ 15K QPS |
| B8 TPC-H | SF=1 | 22/22 查询通过 |

### 4.4 RC Gate (Week 18)

| 测试 | 验证内容 | 通过标准 |
|------|----------|----------|
| R1 Build | 编译通过 | cargo build --release |
| R2 Test | 全量测试 | cargo test ≥ 90% |
| R3 Clippy | 零警告 | cargo clippy -- -D warnings |
| R4 Coverage | 覆盖率 | ≥ 85% |
| R5 Sysbench | MySQL 协议 | point_select ≥ 30K QPS |
| R6 SQL Compat | 完整性 | 85% MySQL 语法 |
| R7 GMP | 电子签名 | 签名/验签集成测试 |
| R8 GMP | Immutable | UPDATE/DELETE 拒绝 |
| R9 GMP | Correction | 修正链完整 |
| R10 TPC-H | SF=1 | 22/22 通过 |
| R11 Stability | 72h | 99% QPS 保持 |
| R12 Security | 审计链 | 签名验证通过 |

### 4.5 GA Gate (Week 20)

| 测试 | 验证内容 | 通过标准 |
|------|----------|----------|
| G1 Build | 编译通过 | cargo build --release |
| G2 Test | 全量测试 | cargo test --lib |
| G3 Clippy | 零警告 | cargo clippy --all-features -- -D warnings |
| G4 Format | 格式检查 | cargo fmt --check |
| G5 Coverage | 覆盖率 | ≥ 85% |
| G6 Security | 安全扫描 | cargo audit |
| G7 SQL Compat | 兼容性 | 85% MySQL 语法 |
| G8 TPC-H SF=1 | OLAP | 22/22 |
| G9 Performance | 性能基准 | 达标 |
| G10 Proofs | TLA+ | 形式化验证 |
| G11 Docs | 文档完整 | 所有 OO 文档存在 |
| G12 MySQL | 协议兼容 | 握手/查询/结果 |
| G-QA1 | 电子签名 | GMP 合规 |
| G-QA2 | Immutable | 数据不可改 |
| G-QA3 | Correction | 修正链可查 |
| G-QA4 | Provenance | 来源可追溯 |
| G-QA5 | Timestamp | RFC3161 |
| G-QA6 | Workflow | 状态机正确 |
| G-S1 | Integration | 集成测试 |
| G-S2 | Sysbench | point_select ≥ 30K |
| G-S3 | WAL | 崩溃恢复 |
| G-S4 | Stability | 72h 稳定 |

---

## 五、OO 文档计划

### 5.1 v3.2.0 OO 文档清单

| 文档 | 优先级 | 状态 | 说明 |
|------|--------|------|------|
| `oo/gmp/DIGITAL_SIGNATURE.md` | P0 | ⏳ | 数字签名审计链设计 |
| `oo/gmp/ELECTRONIC_SIGNATURE.md` | P0 | ⏳ | 电子签名实现 |
| `oo/gmp/IMMUTABLE_RECORD.md` | P0 | ⏳ | Immutable 表设计 |
| `oo/gmp/CORRECTION_CHAIN.md` | P0 | ⏳ | 修正链表设计 |
| `oo/gmp/PROVENANCE_TRACKING.md` | P0 | ⏳ | 来源追踪设计 |
| `oo/gmp/HSM_INTEGRATION.md` | P0 | ⏳ | HSM/KMS 集成 |
| `oo/gmp/WORKFLOW_ENGINE.md` | P1 | ⏳ | GMP 工作流引擎 |
| `oo/gmp/MOBILE_COLLECTION.md` | P1 | ⏳ | 移动端采集协议 |
| `oo/transaction/SERIALIZABLE_SSI.md` | P1 | ⏳ | SSI 死锁检测 |
| `oo/execution/ITERATOR_MODEL.md` | P1 | ⏳ | 向量化执行 |
| `oo/cbo/CBO_INTEGRATION.md` | P1 | ⏳ | CBO 集成 |
| `oo/storage/BUFFER_POOL_LRU.md` | P1 | ⏳ | Buffer Pool LRU |

### 5.2 OO 文档质量标准

```bash
# 检查项
├── 文件存在性
├── 最小大小 (≥ 5KB)
├── 必需章节:
│   ├── 架构图
│   ├── 时序图
│   ├── 算法实现
│   ├── 测试计划
│   └── 覆盖率分析
└── 代码-文档一致性
```

---

## 六、资源估算

### 6.1 人力估算

| 角色 | 人数 | 工作量 |
|------|------|--------|
| 架构师 | 1 | 20 周 |
| 高级工程师 | 2 | 20 周 × 2 |
| 工程师 | 3 | 20 周 × 3 |
| QA | 1 | 15 周 |
| 文档工程师 | 1 | 10 周 |

### 6.2 工作量汇总

| 类别 | 任务数 | 总工作量 |
|------|--------|----------|
| GMP Native | 12 | L×4 + M×5 + S×3 |
| 技术修复 | 5 | M×3 + L×2 |
| MySQL 兼容 | 5 | M×2 + L×3 |
| OO 文档 | 6 | M×2 + S×4 |
| **总计** | **28** | **~120 人周** |

---

## 七、风险管理

### 7.1 高风险项

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| HSM/TPM 集成复杂度 | 高 | 中 | 预留 2 周 buffer |
| SSI 死锁检测正确性 | 高 | 中 | TLA+ 验证 |
| MySQL 协议性能未达标 | 中 | 低 | 分阶段验证 |
| 数字签名安全漏洞 | 高 | 低 | 安全审计 |

### 7.2 中风险项

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| TLA+ 规格编写延期 | 中 | 中 | 提前启动 |
| GMP 合规验证复杂度 | 中 | 中 | 咨询专家 |
| 多表 DML 正确性 | 中 | 低 | 充分测试 |

---

## 八、验收标准

### 8.1 功能验收

| 功能 | 验收条件 |
|------|----------|
| 数字签名审计链 | 签名不可伪造，验签 100% 准确 |
| 电子签名 | 满足 21 CFR Part 11 要求 |
| Immutable Record | UPDATE/DELETE 返回错误 |
| Correction Chain | 修正链完整可查 |
| HSM/KMS | 支持 TPM/HSM/KMS 三种 |
| MySQL 协议 | point_select ≥ 30K QPS |
| CBO | 查询计划优化生效 |
| 窗口函数 | NTILE/LEAD/LAG 通过测试 |
| TPC-H SF=1 | 22/22 查询通过 |

### 8.2 质量验收

| 指标 | 目标 |
|------|------|
| 测试覆盖率 | ≥ 85% |
| Clippy 警告 | 0 |
| Format 检查 | 通过 |
| TLA+ 验证 | 100% P0 功能 |
| OO 文档 | 全部存在 |

---

## 九、附录

### 9.1 参考文档

| 文档 | 说明 |
|------|------|
| `v320_STRATEGIC_DEVELOPMENT_GUIDE.md` | 战略开发指导 |
| `v320_ISSUE_SUMMARY.md` | 问题汇总 |
| `OO_DOCUMENT_ANALYSIS.md` | OO 文档分析 |
| `SYSTEM_BOTTLENECK_ANALYSIS.md` | 系统瓶颈分析 |
| `MYSQL_PROTOCOL_OPTIMIZATION.md` | MySQL 协议优化 |
| `GMP_REPLACEMENT_ASSESSMENT.md` | GMP 替代评估 |

### 9.2 术语表

| 术语 | 说明 |
|------|------|
| SSI | Serializable Snapshot Isolation |
| MVCC | Multi-Version Concurrency Control |
| WAL | Write-Ahead Logging |
| HSM | Hardware Security Module |
| KMS | Key Management Service |
| TPM | Trusted Platform Module |
| ALCOA+ | Attributable, Legible, Contemporaneous, Original, Accurate + Complete, Consistent, Enduring, Available |

---

*本文档由 hermes-z6g4 维护*
*版本 1.0 - 2026-05-15*
