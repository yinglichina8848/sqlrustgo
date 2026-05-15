# SQLRustGo v3.2.0 测试计划

> **版本**: v1.0
> **日期**: 2026-05-15
> **维护人**: hermes-z6g4
> **基于**: GOVERNANCE_INDEX.md 测试计划模版

---

## 一、测试策略

### 1.1 测试金字塔

```
         ▲
        /█\      GA Gate (完整验证)
       /███\     RC Gate (全量功能)
      /█████\    Beta Gate (核心功能)
     /███████\   Alpha Gate (基础功能)
    /█████████\
   └─────────────────────────────────
```

### 1.2 测试分层

| 层级 | 范围 | 测试类型 | 覆盖率目标 |
|------|------|----------|------------|
| **L0** | 单元测试 | 函数/方法级 | 80% |
| **L1** | 集成测试 | 模块间交互 | 70% |
| **L2** | 系统测试 | 端到端场景 | 60% |
| **L3** | 验收测试 | GMP 场景 | 100% |

---

## 二、Alpha 阶段测试 (Week 2)

### 2.1 门禁检查项

| 测试 | 验证内容 | 通过标准 |
|------|----------|----------|
| A1 Build | 编译通过 | cargo build --all-features |
| A2 Test | 单元测试 | cargo test --lib ≥ 90% |
| A3 Clippy | 零警告 | cargo clippy -- -D warnings |
| A4 Format | 格式检查 | cargo fmt --check |
| A5 Coverage | 覆盖率 | ≥ 75% |
| A6 HSM | 密钥接口 | 单元测试通过 |

### 2.2 HSM/KMS 测试

```bash
# HSM 接口测试
cargo test -p sqlrustgo-crypto --lib hsm_

# TPM 接口测试
cargo test -p sqlrustgo-crypto --lib tpm_

# KMS 接口测试
cargo test -p sqlrustgo-crypto --lib kms_
```

### 2.3 MySQL 协议优化测试

```bash
# 协议性能基准测试
cargo test -p sqlrustgo-mysql-server --test protocol_benchmark

# 批量写入测试
cargo test -p sqlrustgo-mysql-server --test batch_write

# Uppercase 缓存测试
cargo test -p sqlrustgo-mysql-server --test cache_test
```

---

## 三、Beta 阶段测试 (Week 12)

### 3.1 门禁检查项

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

### 3.2 GMP 电子签名测试

```sql
-- 测试用例 1: 电子签名创建
INSERT INTO electronic_signature (record_id, signer_id, signature, reason)
VALUES ('batch-001', 'user-001', 'signature...', 'Batch release approval');

-- 验证签名
SELECT verify_signature('batch-001', 'signature...');

-- 测试用例 2: 双人复核
CREATE APPROVAL POLICY batch_release (
    required_signatures = 2,
    required_roles = ('QA_MANAGER', 'PRODUCTION_MANAGER')
);

-- 验证双人复核
SELECT check_approval_policy('batch_release', 'BATCH-001');
```

### 3.3 Immutable Record 测试

```sql
-- 测试用例: Immutable 表禁止修改
CREATE TABLE batch_record (...) IMMUTABLE;

-- 验证 INSERT 允许
INSERT INTO batch_record VALUES (...);  -- 应成功

-- 验证 UPDATE 禁止
UPDATE batch_record SET quantity = 1000 WHERE id = '...';  -- 应失败

-- 验证 DELETE 禁止
DELETE FROM batch_record WHERE id = '...';  -- 应失败
```

### 3.4 Correction Chain 测试

```sql
-- 测试用例: 修正记录
CORRECT RECORD batch_record
    SET quantity = 1000
    WHERE id = 'batch-001'
    REASON '批次拆分'
    APPROVED BY 'qa-manager-001';

-- 验证修正链完整
SELECT * FROM correction_chain
    WHERE original_record_id = 'batch-001'
    ORDER BY corrected_at;
```

---

## 四、RC 阶段测试 (Week 18)

### 4.1 门禁检查项

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

### 4.2 Sysbench 测试

```bash
# Point Select 测试
sysbench oltp_point_select \
    --tables=10 \
    --threads=8 \
    --time=60 \
    run

# 目标: ≥ 30,000 QPS

# OLTP Read-Write 测试
sysbench oltp_read_write \
    --tables=10 \
    --threads=8 \
    --time=60 \
    run

# 目标: ≥ 8,000 QPS
```

### 4.3 TPC-H SF=1 测试

```bash
# 生成 SF=1 数据
./scripts/tpch_dbgen -s 1

# 执行 TPC-H
bash scripts/gate/check_tpch.sh sf=1

# 目标: 22/22 查询通过
```

### 4.4 GMP 合规测试

```bash
# 审计链完整性测试
cargo test -p sqlrustgo-audit --test audit_chain_integrity

# 数字签名验证测试
cargo test -p sqlrustgo-crypto --test signature_verification

# 电子签名合规测试
cargo test -p sqlrustgo-gmp --test electronic_signature_compliance
```

---

## 五、GA 阶段测试 (Week 20)

### 5.1 门禁检查项

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

### 5.2 GMP 场景测试

#### 5.2.1 批记录审批流程

```sql
-- 场景: 批记录创建 → QA 审核 → 生产经理审批 → 放行
BEGIN;

-- 1. 创建批记录 (Immutable)
INSERT INTO batch_record (batch_number, product, quantity, status)
VALUES ('BATCH-2026-001', 'PRODUCT-A', 1000, 'manufactured');

-- 2. QA 签名
INSERT INTO electronic_signature (...)
VALUES ('qa-approval', 'qa-manager-001', 'signature...', 'QA Approved');

-- 3. 生产经理审批 (双人复核)
INSERT INTO electronic_signature (...)
VALUES ('prod-approval', 'prod-manager-001', 'signature...', 'Production Approved');

-- 4. 放行
UPDATE batch_record SET status = 'released' WHERE batch_number = 'BATCH-2026-001';

COMMIT;
```

#### 5.2.2 偏差处理流程

```sql
-- 场景: 偏差发现 → 记录 → 评估 → 审批 → 关闭

-- 1. 发现偏差
INSERT INTO deviation (batch_id, description, severity)
VALUES ('BATCH-001', '温度超标', 'critical');

-- 2. 根因分析
CORRECT RECORD batch_record
    SET temperature_record = '记录值'
    REASON '偏差记录'
    APPROVED BY 'qa-manager-001';

-- 3. CAPA 流程
INSERT INTO capa (deviation_id, corrective_action, preventive_action)
VALUES ('DEV-001', '调整设备参数', '增加监控频率');

-- 4. 审批关闭
INSERT INTO approval_chain (record_id, approvers, status)
VALUES ('CAPA-001', ['qa-manager', 'quality-director'], 'approved');

-- 5. 电子签名关闭
INSERT INTO electronic_signature (...)
VALUES ('deviation-close', 'quality-director', 'signature...', 'Deviation Closed');
```

### 5.3 72 小时稳定性测试

```bash
# 长时间运行测试
cargo test --test long_running_stability -- --test-threads=1

# 预期结果: 99% QPS 保持
```

---

## 六、测试数据准备

### 6.1 TPC-H SF 数据集

| SF | 行数 (Lineitem) | 数据大小 | 用途 |
|----|------------------|----------|------|
| SF=0.1 | 60,000 | ~10MB | 快速验证 |
| SF=1 | 600,000 | ~100MB | 标准测试 |
| SF=10 | 6,000,000 | ~1GB | 压力测试 |

### 6.2 GMP 测试数据

```sql
-- 批记录测试数据
INSERT INTO batch_record (batch_number, product_code, quantity)
SELECT 
    'BATCH-' || generate_series(1, 1000),
    'PROD-' || (random() * 100)::int,
    (random() * 1000)::int
FROM generate_series(1, 1000);

-- 用户和角色
INSERT INTO users (id, username, role) VALUES
    ('user-001', 'qa_manager', 'QA_MANAGER'),
    ('user-002', 'prod_manager', 'PRODUCTION_MANAGER'),
    ('user-003', 'operator', 'PRODUCTION_OPERATOR');
```

---

## 七、测试工具

### 7.1 自动化测试脚本

```bash
# Alpha Gate
bash scripts/gate/check_alpha_v320.sh

# Beta Gate
bash scripts/gate/check_beta_v320.sh

# RC Gate
bash scripts/gate/check_rc_v320.sh

# GA Gate
bash scripts/gate/check_ga_v320.sh
```

### 7.2 性能测试工具

```bash
# Sysbench
sysbench --version

# TPC-H
bash scripts/gate/check_tpch.sh sf=1

# 回归测试
bash scripts/gate/check_regression.sh
```

---

## 八、测试覆盖率目标

### 8.1 按模块

| 模块 | Alpha | Beta | RC | GA |
|------|-------|------|-----|-----|
| executor | 75% | 80% | 85% | 90% |
| storage | 75% | 80% | 85% | 90% |
| transaction | 80% | 85% | 90% | 95% |
| network | 70% | 75% | 80% | 85% |
| parser | 80% | 85% | 90% | 95% |
| **GMP 模块** | - | 70% | 80% | 90% |

### 8.2 GMP 覆盖率

| 功能 | 覆盖率目标 | 关键测试 |
|------|------------|----------|
| 数字签名 | 95% | 签名/验签/边界 |
| 电子签名 | 95% | 策略/双人复核 |
| Immutable | 100% | INSERT/UPDATE/DELETE |
| Correction | 95% | 链完整性 |
| Provenance | 90% | 字段级追踪 |
| Workflow | 90% | 状态转换 |

---

## 九、测试执行规范

### 9.1 本地开发测试 (L0-L1)

```bash
# 快速测试 (L0)
cargo test --lib -- --test-threads=4

# 完整单元测试 (L1)
cargo test --all-features --lib

# Clippy 检查
cargo clippy --all-features -- -D warnings
```

### 9.2 CI 门禁测试 (L2)

```bash
# Alpha Gate
bash scripts/gate/check_alpha_v320.sh

# Beta Gate
bash scripts/gate/check_beta_v320.sh

# RC Gate
bash scripts/gate/check_rc_v320.sh
```

### 9.3 验收测试 (L3)

```bash
# GMP 场景测试
bash scripts/test/gmp_scenario_test.sh

# 72h 稳定性测试
bash scripts/test/stability_72h.sh

# TPC-H 测试
bash scripts/gate/check_tpch.sh sf=1
```

---

## 十、问题追踪

### 10.1 测试失败处理

```
测试失败 → 创建 Issue → 分析根因 → 修复 → 验证 → 关闭
     ↓
  标记为 Bug 或 Test Gap
```

### 10.2 测试覆盖率追踪

| 模块 | 当前 | 目标 | 差距 | 改进计划 |
|------|------|------|------|----------|
| executor | 85% | 90% | 5% | 增加边界测试 |
| transaction | 80% | 95% | 15% | SSI 测试增强 |
| GMP | - | 90% | - | 新建测试 |

---

*本文档由 hermes-z6g4 维护*
*版本 1.0 - 2026-05-15*
