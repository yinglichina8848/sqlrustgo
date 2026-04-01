# Stored Procedure Execution Engine - Issue #1164 Implementation Plan

> **Version**: 1.0  
> **Date**: 2026-04-01  
> **Status**: In Progress  
> **Related Issue**: #1164  
> **Branch**: `feature/issue-1164-procedure-executor`

---

## 1. Overview

### 1.1 Current Status

The `develop/v2.1.0` branch contains a substantial stored procedure execution engine implementation in `crates/executor/src/stored_proc.rs`. This document outlines the remaining work to complete Issue #1164.

### 1.2 Implemented Features

| Feature | Status | Location |
|---------|--------|----------|
| ProcedureContext | ✅ | stored_proc.rs:32-214 |
| DECLARE | ✅ | stored_proc.rs |
| SET | ✅ | stored_proc.rs |
| IF/ELSEIF/ELSE | ✅ | stored_proc.rs |
| WHILE loop | ✅ | stored_proc.rs |
| LOOP + LEAVE/ITERATE | ✅ | stored_proc.rs |
| RETURN | ✅ | stored_proc.rs |
| CALL (nested) | ✅ | stored_proc.rs |
| SIGNAL/RESIGNAL | ✅ | stored_proc.rs |
| Variable expansion (@var) | ✅ | stored_proc.rs |
| Label stack | ✅ | stored_proc.rs |

### 1.3 Remaining Work

| Priority | Feature | Description |
|----------|---------|-------------|
| HIGH | SQL Integration | Execute actual SQL within procedures |
| HIGH | SELECT INTO | Proper row count checking |
| HIGH | Block-level scope | BEGIN/END scope management |
| MEDIUM | Handler support | DECLARE HANDLER FOR SQLEXCEPTION |
| MEDIUM | Cursor support | SELECT loops with FETCH |
| LOW | Trigger enhancement | Complex trigger body support |

---

## 2. SQL Integration

### 2.1 Problem

Currently, `RawSql` statements in procedures are placeholders that don't actually execute:

```rust
StoredProcStatement::RawSql(sql) => {
    if !sql.is_empty() {
        // In a real implementation, this would execute the SQL
    }
    Ok(())
}
```

### 2.2 Solution

Implement proper SQL execution by integrating with the existing executor:

```rust
impl StoredProcExecutor {
    fn execute_raw_sql(&self, sql: &str, ctx: &mut ProcedureContext) -> Result<ExecutorResult, String> {
        // Expand @variables in SQL
        let expanded_sql = self.expand_variables_in_sql(sql, ctx);
        
        // Parse the SQL statement
        let statement = parse(&expanded_sql)
            .map_err(|e| format!("Failed to parse SQL: {}", e))?;
        
        // Plan and execute using existing infrastructure
        let result = self.session.execute_statement(statement)?;
        
        Ok(result)
    }
}
```

### 2.3 Key Considerations

1. **Transaction scope**: Must maintain proper transaction context
2. **Variable expansion**: Replace @var with actual values before parsing
3. **Result handling**: SELECT results vs INSERT/UPDATE/DELETE affected rows
4. **Privilege checking**: Use caller's privileges for SQL execution

---

## 3. SELECT INTO Implementation

### 3.1 MySQL Semantics

```sql
SELECT col1, col2 INTO @var1, @var2 FROM table WHERE condition;
```

Rules:
- **0 rows**: Signal NOT FOUND (SQLSTATE 02000)
- **1 row**: Assign values to variables
- **>1 rows**: Signal TOO MANY ROWS (SQLSTATE 21000)

### 3.2 Current Implementation

```rust
StoredProcStatement::SelectInto { columns, into_vars, table, where_clause } => {
    // Current: just OK, doesn't actually execute
    Ok(())
}
```

### 3.3 Required Implementation

```rust
StoredProcStatement::SelectInto { columns, into_vars, table, where_clause } => {
    // Build and execute SELECT query
    let sql = format!(
        "SELECT {} FROM {} {}",
        columns.join(", "),
        table,
        where_clause.unwrap_or_default()
    );
    
    let result = self.execute_select(&sql, ctx)?;
    
    match result.rows.len() {
        0 => Err("SQLSTATE 02000: NOT FOUND".to_string()),
        1 => {
            for (i, var_name) in into_vars.iter().enumerate() {
                if let Some(val) = result.rows[0].get(i) {
                    ctx.set_var(var_name, val.clone());
                }
            }
            Ok(())
        }
        _ => Err("SQLSTATE 21000: TOO MANY ROWS".to_string()),
    }
}
```

---

## 4. Block-Level Scope

### 4.1 MySQL Semantics

```sql
BEGIN
    DECLARE x INT;
    BEGIN
        DECLARE x INT;  -- Shadows outer x
        SET x = 1;
    END;
    -- Here, x refers to outer x
END;
```

### 4.2 Required Implementation

Add scope stack to ProcedureContext:

```rust
pub struct ProcedureContext {
    scopes: Vec<VariableScope>,  // Stack of scopes
    current_scope: usize,       // Current scope index
}

impl ProcedureContext {
    pub fn enter_scope(&mut self) {
        self.scopes.push(VariableScope::new());
        self.current_scope = self.scopes.len() - 1;
    }
    
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
            self.current_scope = self.scopes.len() - 1;
        }
    }
    
    pub fn declare_variable(&mut self, name: &str, value: Value) {
        // Declare in current scope
        self.scopes[self.current_scope].variables.insert(name.to_string(), value);
    }
    
    pub fn lookup_variable(&self, name: &str) -> Option<&Value> {
        // Search from innermost to outermost
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.variables.get(name) {
                return Some(v);
            }
        }
        None
    }
}
```

### 4.3 BEGIN/END Statement

Add new statement type:

```rust
StoredProcStatement::BeginEnd { body } => {
    ctx.enter_scope();
    let result = self.execute_body(body, ctx);
    ctx.exit_scope();
    result
}
```

---

## 5. Handler Support

### 5.1 MySQL Syntax

```sql
DECLARE CONTINUE HANDLER FOR SQLEXCEPTION
BEGIN
    -- error handling code
    SET @error = 1;
END;
```

### 5.2 Handler Types

| Type | Action |
|------|--------|
| CONTINUE | Continue executing after handler |
| EXIT | Exit the compound statement containing the handler |
| UNDO | Not supported (requires rollback) |

### 5.3 Conditions

- SQLEXCEPTION: SQLSTATE starting with '40', '42', etc.
- SQLWARNING: SQLSTATE starting with '01'
- NOT FOUND: SQLSTATE '02000'
- Custom: Specific SQLSTATE codes

### 5.4 Implementation

```rust
pub struct HandlerFrame {
    pub condition: HandlerCondition,
    pub action: HandlerAction,
    pub body: Vec<StoredProcStatement>,
}

pub enum HandlerCondition {
    Sqlexception,
    Sqlwarning,
    NotFound,
    Specific(String),  // Specific SQLSTATE
}

pub enum HandlerAction {
    Continue,
    Exit,
}

impl StoredProcExecutor {
    fn find_handler(&self, ctx: &ProcedureContext, sqlstate: &str) -> Option<&HandlerFrame> {
        ctx.get_handlers().iter().rev().find(|h| {
            match &h.condition {
                HandlerCondition::Sqlexception => sqlstate.starts_with('4') || sqlstate.starts_with('5'),
                HandlerCondition::Sqlwarning => sqlstate.starts_with('0'),
                HandlerCondition::NotFound => sqlstate == "02000",
                HandlerCondition::Specific(s) => s == sqlstate,
            }
        })
    }
}
```

---

## 6. Cursor Support

### 6.1 MySQL Syntax

```sql
DECLARE cur CURSOR FOR SELECT col FROM table;
OPEN cur;
FETCH cur INTO @var;
CLOSE cur;
```

### 6.2 Implementation Plan

```rust
pub struct Cursor {
    name: String,
    query: String,
    result: Option<ExecutorResult>,
    row_index: usize,
}

pub enum StoredProcStatement {
    DeclareCursor { name: String, query: String },
    OpenCursor { name: String },
    FetchCursor { name: String, into_vars: Vec<String> },
    CloseCursor { name: String },
}
```

---

## 7. Trigger Enhancement

### 7.1 Current State

Trigger executor only supports simple `SET NEW.col = value`:

```rust
if body.starts_with("SET NEW.") {
    // Simple parser for SET NEW.col = expression
    self.parse_simple_set_assignments(body)
}
```

### 7.2 Goal

Support full procedural trigger bodies:

```sql
CREATE TRIGGER before_insert
BEFORE INSERT ON orders
FOR EACH ROW
BEGIN
    IF NEW.price < 0 THEN
        SET NEW.price = 0;
    END IF;
    SET NEW.total = NEW.price * NEW.quantity;
END;
```

### 7.3 Implementation

Unify TriggerExecutor with StoredProcExecutor:

```rust
impl TriggerExecutor {
    fn execute_trigger_body(&self, body: &[StoredProcStatement], context: &TriggerContext) -> Result<Record, TriggerError> {
        let mut ctx = ProcedureContext::new();
        ctx.set_trigger_context(context);
        
        // Execute trigger statements
        self.execute_body(body, &mut ctx)?;
        
        // Return modified NEW row
        Ok(context.new_row.clone())
    }
}
```

---

## 8. Implementation Phases

### Phase 1: SQL Integration (HIGH)
- [ ] Implement `execute_raw_sql()`
- [ ] Variable expansion in SQL
- [ ] Execute SELECT/INSERT/UPDATE/DELETE
- [ ] Return result sets properly

### Phase 2: SELECT INTO (HIGH)
- [ ] Execute SELECT query
- [ ] Row count checking (0, 1, >1)
- [ ] Variable assignment
- [ ] SQLSTATE signaling

### Phase 3: Block Scope (HIGH)
- [ ] Add scope stack to ProcedureContext
- [ ] Implement `enter_scope()` / `exit_scope()`
- [ ] Variable shadowing support
- [ ] BEGIN/END statement

### Phase 4: Handlers (MEDIUM)
- [ ] Handler storage in context
- [ ] Handler lookup by SQLSTATE
- [ ] CONTINUE/EXIT actions
- [ ] DECLARE HANDLER parsing

### Phase 5: Cursors (MEDIUM)
- [ ] Cursor struct
- [ ] OPEN/FETCH/CLOSE
- [ ] Multiple cursors
- [ ] Cursor variables

### Phase 6: Trigger Enhancement (LOW)
- [ ] Procedural trigger bodies
- [ ] BEFORE/AFTER integration
- [ ] NEW/OLD row access
- [ ] Trigger privilege checks

---

## 9. Testing Strategy

### Unit Tests

```rust
#[test]
fn test_declare_and_set() {
    let result = execute_procedure("test_proc", vec![]);
    assert!(result.is_ok());
}

#[test]
fn test_if_condition() {
    let args = vec![Value::Integer(10)];
    let result = execute_procedure("test_if", args);
    assert_eq!(result.unwrap(), Value::Integer(1));
}

#[test]
fn test_while_loop() {
    let result = execute_procedure("test_while", vec![]);
    // Verify loop executed correct number of times
}
```

### Integration Tests

```rust
#[test]
fn test_procedure_with_sql() {
    // Create table
    // Execute procedure that inserts/selects
    // Verify results
}
```

---

## 10. File Structure

```
crates/executor/src/
├── lib.rs
├── stored_proc.rs          # Main implementation
│   ├── ProcedureContext    # ✅ Done
│   ├── StoredProcExecutor  # ✅ Core done, needs SQL integration
│   ├── ExpressionEvaluator  # ✅ Done
│   └── Statement executors # ⚠️ RawSql needs implementation
└── trigger.rs             # ⚠️ Needs enhancement
```

---

## 11. Dependencies

| Module | Status |
|--------|--------|
| Parser | ✅ Complete |
| Catalog | ✅ Complete |
| Planner | ✅ Available |
| Executor | ✅ Available |
| Session | ⚠️ Needs integration |

---

## 12. Risks

| Risk | Mitigation |
|------|------------|
| SQL injection via @variables | Validate variable names, use parameterized queries |
| Infinite loops | Statement count limit |
| Transaction scope issues | Use session-level execution |
| Privilege escalation | RBAC already merged in v2.1.0 |

---

## 13. References

- MySQL Stored Procedure Language: https://dev.mysql.com/doc/refman/8.0/en/stored-programs-defined.html
- PostgreSQL PL/pgSQL: https://www.postgresql.org/docs/current/plpgsql.html
