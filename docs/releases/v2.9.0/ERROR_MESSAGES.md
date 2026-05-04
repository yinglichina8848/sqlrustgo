# SQLRustGo Error Messages Reference

> **Version**: v2.9.0
> **Last Updated**: 2026-05-05

---

## Overview

SQLRustGo provides comprehensive error messages for debugging and user feedback.

---

## Error Code Format

All errors follow the format:
```
SQLRustGo[CODE]: Message
```

Example:
```
SQLRustGo[E1001]: Table 'users' not found
```

---

## Error Codes

### E1000 - E1999: Parser Errors

| Code | Error | Description |
|------|-------|-------------|
| E1001 | SYNTAX_ERROR | SQL syntax error |
| E1002 | UNEXPECTED_TOKEN | Unexpected token |
| E1003 | UNTERMINATED_STRING | Unterminated string literal |
| E1004 | INVALID_NUMBER | Invalid number format |
| E1005 | UNKNOWN_KEYWORD | Unknown SQL keyword |

### E2000 - E2999: Catalog Errors

| Code | Error | Description |
|------|-------|-------------|
| E2001 | TABLE_NOT_FOUND | Table does not exist |
| E2002 | COLUMN_NOT_FOUND | Column does not exist |
| E2003 | DATABASE_NOT_FOUND | Database does not exist |
| E2004 | INDEX_NOT_FOUND | Index does not exist |
| E2005 | DUPLICATE_NAME | Name already exists |

### E3000 - E3999: Execution Errors

| Code | Error | Description |
|------|-------|-------------|
| E3001 | DIVISION_BY_ZERO | Division by zero |
| E3002 | TYPE_MISMATCH | Type mismatch in expression |
| E3003 | NULL_VALUE | NULL value in NOT NULL column |
| E3004 | CONSTRAINT_VIOLATION | Constraint violation |
| E3005 | LOCK_TIMEOUT | Lock acquisition timeout |

### E4000 - E4999: Transaction Errors

| Code | Error | Description |
|------|-------|-------------|
| E4001 | DEADLOCK_DETECTED | Deadlock detected |
| E4002 | SERIALIZATION_FAILURE | Serialization failure |
| E4003 | ROLLBACK_REQUIRED | Transaction must rollback |
| E4004 | INVALID_TRANSACTION_STATE | Invalid transaction state |

### E5000 - E5999: Storage Errors

| Code | Error | Description |
|------|-------|-------------|
| E5001 | IO_ERROR | Disk I/O error |
| E5002 | OUT_OF_SPACE | Disk space exhausted |
| E5003 | CORRUPT_PAGE | Corrupt data page |
| E5004 | CHECKSUM_MISMATCH | Page checksum mismatch |

---

## Error Handling

### Rust API

```rust
use sqlrustgo::types::{SqlError, SqlResult};

fn handle_error(err: SqlError) {
    match err.code.as_str() {
        "E2001" => println!("Table not found: {}", err.detail),
        "E3001" => println!("Division by zero"),
        _ => println!("Unknown error: {}", err),
    }
}
```

### SQL Error Handling

```sql
-- 使用 TRY_CATCH 捕获错误
BEGIN TRY
    INSERT INTO users (id, name) VALUES (1, 'test');
END TRY
BEGIN CATCH
    SELECT ERROR_MESSAGE() AS error;
END CATCH;
```

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
