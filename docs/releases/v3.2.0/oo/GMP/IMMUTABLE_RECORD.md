# OO-3: Immutable Record 设计文档

> **版本**: v1.0
> **日期**: 2026-05-16
> **基于**: v3.2.0
> **维护人**: hermes-z6g4
> **状态**: 设计中

---

## 一、概述

### 1.1 目标

实现 Immutable Record (EBR - Append-Based Record) 机制，保证数据不可修改：

- **数据不可变性**: 写入后永不修改/删除
- **历史版本保留**: 完整的修改历史
- **纠错机制**: 通过 CORRECTION RECORD 修正错误

### 1.2 核心理念

```
Immutable Record = Append-Only Storage + Correction Records
```

---

## 二、技术架构

### 2.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                    Immutable Record System                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Storage   │◀───│   DML        │───▶│   History       │  │
│  │   Engine    │    │   Interceptor │    │   Manager       │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Append-Only │    │  Correction  │    │   Signature      │  │
│  │   Storage   │    │   Records     │    │   Chain         │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、CREATE TABLE 语法

### 3.1 IMMUTABLE 表

```sql
CREATE TABLE sensitive_data (
    id          BIGINT PRIMARY KEY,
    name        TEXT NOT NULL,
    value       TEXT,
    created_at  TIMESTAMP DEFAULT NOW()
) ENGINE = IMMUTABLE;
```

### 3.2 表类型

| 引擎类型 | 说明 | UPDATE/DELETE | CORRECTION |
|----------|------|----------------|------------|
| INNODB | 标准表 | ✅ 允许 | ❌ 不支持 |
| IMMUTABLE | 不可变表 | ❌ 禁止 | ✅ 通过 CORRECTION |

---

## 四、数据结构

### 4.1 历史记录表 (gmp_immutable_history)

```sql
CREATE TABLE gmp_immutable_history (
    id                  BIGSERIAL PRIMARY KEY,
    original_table      TEXT NOT NULL,
    original_record_id  TEXT NOT NULL,
    operation           TEXT NOT NULL,           -- INSERT/CORRECTION
    previous_data       JSONB,                  -- 修正前的数据
    new_data            JSONB NOT NULL,          -- 修正后的数据
    correction_reason   TEXT,
    corrector_id        TEXT NOT NULL,
    corrector_role      TEXT,
    correction_policy   TEXT,
    signature_id        UUID REFERENCES gmp_signature_metadata(id),
    timestamp           BIGINT NOT NULL,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

### 4.2 修正记录表 (gmp_correction_records)

```sql
CREATE TABLE gmp_correction_records (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name          TEXT NOT NULL,
    record_id           TEXT NOT NULL,
    original_value      JSONB NOT NULL,
    corrected_value     JSONB NOT NULL,
    correction_type     TEXT NOT NULL,           -- CLARIFICATION/ERROR_CORRECTION
    reason              TEXT NOT NULL,
    corrector_id        TEXT NOT NULL,
    approver_id         TEXT,
    signature_id        UUID REFERENCES gmp_signature_metadata(id),
    status              TEXT DEFAULT 'PENDING',   -- PENDING/APPROVED/REJECTED
    timestamp           BIGINT NOT NULL,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

---

## 五、DML 拦截机制

### 5.1 拦截逻辑

```rust
/// 检查表是否为 IMMUTABLE 类型
fn check_immutable_modification(table_name: &str, operation: &str) -> Result<(), Error> {
    let table_type = catalog.get_table_type(table_name)?;

    match operation {
        "UPDATE" | "DELETE" if table_type == TableType::Immutable => {
            Err(Error::ImmutableTableModification {
                table: table_name.to_string(),
                operation: operation.to_string(),
            })
        }
        _ => Ok(()),
    }
}
```

### 5.2 CORRECTION vs UPDATE

| 特性 | UPDATE | CORRECTION |
|------|--------|------------|
| 目的 | 修改数据 | 纠错 |
| 历史 | 覆盖 | 保留 |
| 签名 | 可选 | 必须 |
| 审批 | 可选 | 必须 |

---

## 六、CORRECTION 流程

### 6.1 修正流程

```
1. User initiates correction request
         │
         ▼
2. System validates IMMUTABLE table
         │
         ▼
3. User provides correction reason
         │
         ▼
4. System creates CORRECTION record (PENDING)
         │
         ▼
5. Second user approves/rejects
         │
         ├──▶ Approved → Apply correction + Sign
         │
         └──▶ Rejected → Cancel correction
```

### 6.2 SQL 语法

```sql
-- 发起修正
CORRECT RECORD table_name
FOR record_id
SET column = value
WITH REASON 'correction reason'
[BY POLICY policy_name];

-- 查询修正历史
SELECT * FROM gmp_correction_records
WHERE table_name = 'sensitive_data';

-- 查询历史版本
SELECT * FROM gmp_immutable_history
WHERE original_table = 'sensitive_data'
AND original_record_id = '123';
```

---

## 七、API 设计

### 7.1 核心 Trait: `ImmutableRecordProvider`

```rust
/// 不可变记录提供者接口
pub trait ImmutableRecordProvider {
    /// 创建不可变记录
    fn insert(&self, table: &str, record: &Record) -> Result<RecordId, Error>;

    /// 查询历史记录
    fn query_history(
        &self,
        table: &str,
        record_id: &str,
    ) -> Result<Vec<HistoryEntry>, Error>;

    /// 发起修正
    fn create_correction(
        &self,
        request: &CorrectionRequest,
    ) -> Result<CorrectionId, Error>;

    /// 审批修正
    fn approve_correction(
        &self,
        correction_id: Uuid,
        approver_id: &str,
    ) -> Result<(), Error>;

    /// 应用修正
    fn apply_correction(
        &self,
        correction_id: Uuid,
    ) -> Result<(), Error>;
}
```

---

## 八、实现状态

| 阶段 | 任务 | 状态 | PR |
|------|------|------|-----|
| 1 | 数据结构定义 | ✅ | #1029 |
| 2 | DML 拦截 | ✅ | #1029 |
| 3 | CORRECTION 语法 | ✅ | #1029 |
| 4 | 历史查询 | ✅ | #1029 |

---

## 九、依赖

- GMP-1: 审计链 (已完成)
- GMP-2: 电子签名 (已完成)
- GMP-4: Correction Chain (进行中)

---

*本文档由 hermes-agent 创建*
*版本 1.0 - 2026-05-16*