# SQLRustGo Error Messages Reference

> Version: `v2.8.0`
> Last Updated: 2026-04-23

---

## Overview

SQLRustGo provides comprehensive error messages for debugging and user feedback. All error messages are in English for internationalization.

## Error Codes

| Code | Error Type | Description |
|------|------------|-------------|
| 1000 | `ParseError` | SQL syntax error |
| 1001 | `ExecutionError` | General execution error |
| 1002 | `TypeMismatch` | Data type mismatch |
| 1003 | `DivisionByZero` | Division by zero attempted |
| 1004 | `NullValueError` | NULL value in operation |
| 1005 | `ConstraintViolation` | Constraint check failed |
| 1006 | `TableNotFound` | Table does not exist |
| 1007 | `ColumnNotFound` | Column does not exist |
| 1008 | `DuplicateKey` | Duplicate key violation |
| 1009 | `IoError` | I/O operation failed |
| 1010 | `ProtocolError` | Network protocol error |
| 1011 | `TimeoutError` | Operation timed out |
| 1012 | `OverflowError` | Numeric overflow |
| 1013 | `AuthError` | Authentication failed |

---

## Error Message Format

### Parse Errors (1000)

```
Parse error: near "{token}" at line {line}
Parse error: unexpected token "{token}"
Parse error: syntax error in SQL statement
```

Examples:
```
Parse error: near "FORM" at line 1
Parse error: unexpected token "WHERE"
Parse error: syntax error in SQL statement
```

### Table/Column Errors (1006-1007)

```
Table not found: {table_name}
Column not found: {column_name} in table {table_name}
```

Examples:
```
Table not found: users
Column not found: email in table users
```

### Constraint Errors (1005, 1008)

```
Constraint violation: PRIMARY KEY must be unique
Constraint violation: NOT NULL column '{column}' cannot be NULL
Duplicate key: {value} already exists
```

### Type Errors (1002)

```
Type mismatch: cannot convert "{type}" to "{type}"
Type mismatch: operator {op} not supported for types {type} and {type}
```

### Execution Errors (1001)

```
Execution error: JOINs are not yet fully supported in ExecutionEngine::execute_select
Execution error: division by zero
Execution error: table '{table}' already exists
```

---

## MySQL Compatibility Error Codes

SQLRustGo error codes are compatible with MySQL 5.7 for familiar error handling.

| MySQL Code | SQLRustGo Error | Notes |
|------------|------------------|-------|
| 1062 | `DuplicateKey` | |
| 1146 | `TableNotFound` | |
| 1054 | `ColumnNotFound` | |
| 1216 | `ConstraintViolation` | Foreign key |
| 1217 | `ConstraintViolation` | Child record exists |
| 1136 | `TypeMismatch` | Column count mismatch |
| 1292 | `TypeMismatch` | Incorrect datetime value |

---

## Error Handling Best Practices

### Rust Applications

```rust
use sqlrustgo_types::{SqlError, SqlResult};

fn example() -> SqlResult<()> {
    match engine.execute("SELECT * FROM users") {
        Ok(result) => { /* handle result */ }
        Err(SqlError::TableNotFound(name)) => {
            eprintln!("Table {} does not exist", name);
        }
        Err(SqlError::ParseError(msg)) => {
            eprintln!("SQL syntax error: {}", msg);
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
        }
    }
    Ok(())
}
```

### Network Applications

Error responses follow MySQL wire protocol:
```
ERROR {code} ({state}): {message}
```

Example:
```
ERROR 1146 (42S02): Table 'users' doesn't exist
```

---

## Logging

All errors are logged with:
- Timestamp
- Error code
- Error message
- Stack trace (debug builds)

```rust
tracing::error!("Database error: {:?}", err);
```

---

## Related Documentation

- [Security Hardening Guide](./SECURITY_HARDENING.md)
- [API Reference](./API_REFERENCE.md)
- [Client Connection Guide](./CLIENT_CONNECTION.md)