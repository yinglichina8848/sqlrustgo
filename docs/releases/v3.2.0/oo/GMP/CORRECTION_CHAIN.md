# OO-4: Correction Chain 设计文档

> **版本**: v1.0
> **日期**: 2026-05-16
> **基于**: v3.2.0
> **维护人**: hermes-z6g4
> **状态**: 设计中

---

## 一、概述

### 1.1 目标

实现 Correction Chain（纠错链），提供错误修正的完整审计追踪：

- **修正记录**: 记录所有数据修正
- **双人复核**: 修正需要审批
- **签名链接**: 修正操作签名不可抵赖
- **链式追溯**: 修正链可完整回溯

### 1.2 核心理念

```
Correction Chain = Immutable History + Approval + Signature + Audit
```

---

## 二、技术架构

### 2.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                    Correction Chain System                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │  Correction │───▶│   Approval   │───▶│   Signature      │  │
│  │   Request   │    │   Workflow   │    │   Chain         │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │  Correction  │    │   Two-Four   │    │   Immutable     │  │
│  │   Record     │    │   Eyes Check │    │   History       │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、数据结构

### 3.1 纠错记录表 (gmp_correction_chain)

```sql
CREATE TABLE gmp_correction_chain (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name          TEXT NOT NULL,
    record_id           TEXT NOT NULL,
    column_name         TEXT,
    original_value      JSONB NOT NULL,
    corrected_value     JSONB NOT NULL,
    correction_type     TEXT NOT NULL,           -- CLARIFICATION/ERROR_CORRECTION
    reason              TEXT NOT NULL,
    requester_id        TEXT NOT NULL,
    requester_role      TEXT,
    approver_id         TEXT,
    approver_role       TEXT,
    corrector_id        TEXT,
    signature_id        UUID REFERENCES gmp_signature_metadata(id),
    status              TEXT NOT NULL DEFAULT 'PENDING',
    previous_correction_id UUID,                 -- 链接到上一个修正
    chain_depth         INT DEFAULT 1,
    timestamp           BIGINT NOT NULL,
    created_at          TIMESTAMP DEFAULT NOW(),
    updated_at          TIMESTAMP DEFAULT NOW()
);
```

### 3.2 修正链遍历表 (gmp_correction_path)

```sql
CREATE TABLE gmp_correction_path (
    id                  BIGSERIAL PRIMARY KEY,
    root_correction_id  UUID NOT NULL REFERENCES gmp_correction_chain(id),
    correction_id       UUID NOT NULL REFERENCES gmp_correction_chain(id),
    path_position       INT NOT NULL,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

---

## 四、修正类型

### 4.1 修正类型定义

| 类型 | 代码 | 说明 | 审批要求 |
|------|------|------|----------|
| 说明性修正 | CLARIFICATION | 数据澄清，不改变实际值 | 1人审批 |
| 错误修正 | ERROR_CORRECTION | 修正错误数据 | 2人审批 |

### 4.2 修正流程状态机

```
     ┌─────────────┐
     │   PENDING   │
     └──────┬──────┘
            │
    ┌───────┴───────┐
    │               │
    ▼               ▼
┌────────┐     ┌────────┐
│APPROVED│     │REJECTED│
└───┬────┘     └────────┘
    │
    ▼
┌────────┐     ┌────────┐
│APPLIED │     │CANCELLED│
└────────┘     └────────┘
```

---

## 五、修正链遍历算法

### 5.1 链构建

```rust
/// 构建修正链
fn build_correction_chain(
    &self,
    table_name: &str,
    record_id: &str,
) -> Result<Vec<CorrectionChainEntry>, Error> {
    let mut chain = Vec::new();
    let mut current = self.find_latest_correction(table_name, record_id)?;

    while let Some(correction) = current {
        chain.push(correction.clone());

        if let Some(prev_id) = correction.previous_correction_id {
            current = self.get_correction_by_id(prev_id)?;
        } else {
            current = None;
        }
    }

    chain.reverse();
    Ok(chain)
}
```

### 5.2 链验证

```rust
/// 验证修正链完整性
fn verify_chain_integrity(&self, chain: &[CorrectionChainEntry]) -> Result<bool, Error> {
    for i in 1..chain.len() {
        // 验证链式链接
        if chain[i].previous_correction_id != Some(chain[i-1].id) {
            return Ok(false);
        }

        // 验证签名连续性
        if chain[i].chain_depth != chain[i-1].chain_depth + 1 {
            return Ok(false);
        }
    }
    Ok(true)
}
```

---

## 六、API 设计

### 6.1 核心 Trait: `CorrectionChainProvider`

```rust
/// 纠错链提供者接口
pub trait CorrectionChainProvider {
    /// 创建修正请求
    fn create_correction(
        &self,
        request: &CorrectionRequest,
    ) -> Result<CorrectionResponse, Error>;

    /// 审批修正
    fn approve_correction(
        &self,
        correction_id: Uuid,
        approver: &Approver,
    ) -> Result<(), Error>;

    /// 应用修正
    fn apply_correction(
        &self,
        correction_id: Uuid,
    ) -> Result<ApplyResult, Error>;

    /// 获取修正链
    fn get_correction_chain(
        &self,
        table_name: &str,
        record_id: &str,
    ) -> Result<Vec<CorrectionChainEntry>, Error>;

    /// 验证链完整性
    fn verify_chain(
        &self,
        root_id: Uuid,
    ) -> Result<ChainVerification, Error>;
}
```

### 6.2 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionRequest {
    pub table_name: String,
    pub record_id: String,
    pub column_name: Option<String>,
    pub original_value: JsonValue,
    pub corrected_value: JsonValue,
    pub correction_type: CorrectionType,
    pub reason: String,
    pub requester_id: String,
    pub requester_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionChainEntry {
    pub id: Uuid,
    pub table_name: String,
    pub record_id: String,
    pub correction_type: CorrectionType,
    pub original_value: JsonValue,
    pub corrected_value: JsonValue,
    pub reason: String,
    pub requester_id: String,
    pub approver_id: Option<String>,
    pub corrector_id: Option<String>,
    pub status: CorrectionStatus,
    pub previous_correction_id: Option<Uuid>,
    pub chain_depth: i32,
    pub signature_id: Option<Uuid>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CorrectionType {
    Clarification,
    ErrorCorrection,
}
```

---

## 七、SQL 语句支持

### 7.1 发起修正

```sql
CORRECT TABLE table_name
SET column = new_value
FOR record_id = 'xxx'
WITH REASON 'reason text'
[AS TYPE CLARIFICATION];
```

### 7.2 查询修正链

```sql
-- 获取记录的所有修正
SELECT * FROM gmp_correction_chain
WHERE table_name = 'sensitive_data'
AND record_id = '123'
ORDER BY chain_depth;

-- 验证修正链完整性
SELECT verify_correction_chain('correction-id-here');
```

---

## 八、实现状态

| 阶段 | 任务 | 状态 | PR |
|------|------|------|-----|
| 1 | 数据结构定义 | ✅ | #1027 |
| 2 | 修正链遍历 | ✅ | #1027 |
| 3 | 双人复核 | ✅ | #1027 |
| 4 | 签名集成 | ✅ | #1076 |

---

## 九、依赖

- GMP-1: 审计链 (已完成)
- GMP-2: 电子签名 (已完成)
- GMP-3: Immutable Record (已完成)

---

*本文档由 hermes-agent 创建*
*版本 1.0 - 2026-05-16*