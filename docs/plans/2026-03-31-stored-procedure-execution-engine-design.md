# Stored Procedure & Trigger Execution Engine Design

> **Version**: 1.0  
> **Date**: 2026-03-31  
> **Status**: Draft  
> **Related Issue**: #1165

---

## 1. Overview

### 1.1 Problem Statement

SQLRustGo's parser already supports stored procedure control flow statements (IF/WHILE/LOOP/DECLARE/LEAVE/ITERATE/RETURN/CALL), but the **execution engine** is incomplete:

| Component | Parser | Executor |
|-----------|--------|----------|
| Stored Procedure DECLARE | ✅ | ❌ Local variable storage |
| Stored Procedure IF/ELSE | ✅ | ❌ Condition evaluation |
| Stored Procedure WHILE | ✅ | ❌ Loop execution |
| Stored Procedure LOOP | ✅ | ❌ Loop execution |
| Stored Procedure CALL | ✅ | ❌ Nested call handling |
| Stored Procedure RETURN | ✅ | ❌ Value return |
| Trigger execution | ✅ | ⚠️ Basic only |

### 1.2 Goals

1. **Complete stored procedure execution**: Support full procedural SQL including variable declarations, control flow, and SQL statement execution
2. **Enhanced trigger execution**: Support complex trigger bodies beyond simple SET NEW.col
3. **Integration with existing Volcano executor**: Seamlessly integrate procedure execution with the existing query execution pipeline

---

## 2. Architecture

### 2.1 High-Level Design

```
┌─────────────────────────────────────────────────────────────────┐
│                      Session / Connection                         │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              ProcedureExecutionEngine                      │  │
│  │  ┌─────────────────┐  ┌─────────────────┐                │  │
│  │  │ ProcedureContext │  │ VariableScope   │                │  │
│  │  │ - procedure_name │  │ - local_vars    │                │  │
│  │  │ - params         │  │ - param_modes   │                │  │
│  │  │ - caller_ctx     │  │ - return_value  │                │  │
│  │  └─────────────────┘  └─────────────────┘                │  │
│  │                                                           │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │           ProcedureStatementExecutor                 │  │  │
│  │  │  - execute_declare()                               │  │  │
│  │  │  - execute_if()                                     │  │  │
│  │  │  - execute_while()                                  │  │  │
│  │  │  - execute_loop()                                   │  │  │
│  │  │  - execute_call()                                   │  │  │
│  │  │  - execute_set()                                    │  │  │
│  │  │  - execute_return()                                 │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │                           │                                 │  │
│  │                           ▼                                 │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │           ExpressionEvaluator                       │  │  │
│  │  │  - evaluate_condition()                             │  │  │
│  │  │  - evaluate_expression()                            │  │  │
│  │  │  - evaluate_assignment()                            │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                           │                                      │
│                           ▼                                      │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    VolcanoExecutor                         │  │
│  │  (SeqScan, Projection, Aggregate, Join, etc.)              │  │
│  └───────────────────────────────────────────────────────────┘  │
│                           │                                      │
│                           ▼                                      │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                      StorageEngine                         │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Breakdown

#### 2.2.1 ProcedureContext

```rust
pub struct ProcedureContext {
    pub procedure_name: String,
    pub params: Vec<Value>,                    // IN/OUT parameters
    pub caller_context: Option<Box<ProcedureContext>>,  // For nested calls
}

pub struct VariableScope {
    pub variables: HashMap<String, Value>,
    pub param_modes: HashMap<String, ParamMode>,
}

pub enum ParamMode {
    In(Value),
    Out,           // Caller-provided, may be set
    InOut(Value),  // Both
}
```

#### 2.2.2 ProcedureStatementExecutor

Each statement type from `ProcedureStatement` enum has a corresponding executor:

| Statement | Executor Method | Description |
|-----------|-----------------|-------------|
| Declare | `execute_declare()` | Allocate local variable |
| Set | `execute_set()` | Assign value to variable |
| If | `execute_if()` | Conditional branching |
| While | `execute_while()` | Loop execution |
| Loop | `execute_loop()` | Infinite loop with LEAVE |
| Leave | `execute_leave()` | Exit loop |
| Iterate | `execute_iterate()` | Continue to next iteration |
| Return | `execute_return()` | Return from procedure |
| Call | `execute_call()` | Nested procedure call |
| RawSql | `execute_sql()` | Execute embedded SQL |

#### 2.2.3 ExpressionEvaluator

```rust
pub trait ExpressionEvaluator {
    fn evaluate_condition(&self, expr: &str, ctx: &VariableScope) -> Result<bool, ProcedureError>;
    fn evaluate_expression(&self, expr: &str, ctx: &VariableScope) -> Result<Value, ProcedureError>;
    fn evaluate_assignment(&self, target: &str, value: Value, ctx: &mut VariableScope) -> Result<(), ProcedureError>;
}
```

---

## 3. Detailed Design

### 3.1 Procedure Execution Flow

```
execute_procedure(name, args):
    1. Look up procedure in catalog
    2. Create ProcedureContext with parameters
    3. Create VariableScope
    4. For each statement in procedure.body:
       a. Execute statement via ProcedureStatementExecutor
       b. If Return, exit and return value
       c. If LEAVE with matching label, break loop
       d. If error, handle via SIGNAL/RESIGNAL or propagate
    5. Return OUT parameters to caller
```

### 3.2 Variable Declaration

```sql
DECLARE variable_name DATA_TYPE [DEFAULT value];
```

```rust
fn execute_declare(&self, stmt: &ProcedureStatement) -> Result<(), ProcedureError> {
    if let ProcedureStatement::Declare { name, data_type, default_value } = stmt {
        let value = default_value
            .as_ref()
            .map(|v| self.eval_expression(v))
            .unwrap_or_else(|| Value::Null)?;
        
        self.scope.variables.insert(name.clone(), value);
        Ok(())
    }
}
```

### 3.3 Control Flow: IF Statement

```sql
IF condition THEN
    statements
[ELSEIF condition THEN
    statements]
[ELSE
    statements]
END IF;
```

```rust
fn execute_if(&self, stmt: &ProcedureStatement) -> Result<(), ProcedureError> {
    if let ProcedureStatement::If { condition, then_body, elseif_body, else_body } = stmt {
        // Evaluate main condition
        if self.eval_condition(condition)? {
            return self.execute_statements(then_body);
        }
        
        // Evaluate ELSEIF conditions
        for (elsif_cond, elsif_body) in elseif_body {
            if self.eval_condition(elsif_cond)? {
                return self.execute_statements(elsif_body);
            }
        }
        
        // Execute ELSE body
        self.execute_statements(else_body)
    }
}
```

### 3.4 Control Flow: WHILE Loop

```sql
WHILE condition DO
    statements
END WHILE;
```

```rust
fn execute_while(&self, stmt: &ProcedureStatement) -> Result<(), ProcedureError> {
    if let ProcedureStatement::While { condition, body } = stmt {
        while self.eval_condition(condition)? {
            match self.execute_statements(body) {
                Ok(()) => continue,
                Err(ProcedureError::Leave(_)) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
```

### 3.5 Control Flow: LOOP

```sql
[label:] LOOP
    statements
END LOOP;
```

```rust
fn execute_loop(&self, label: Option<String>, body: &[ProcedureStatement]) -> Result<(), ProcedureError> {
    loop {
        match self.execute_statements(body) {
            Ok(()) => continue,
            Err(ProcedureError::Leave(lbl)) => {
                if lbl == label {
                    break;  // Label matches, exit loop
                }
                return Err(ProcedureError::UnhandledLeave(lbl));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
```

### 3.6 LEAVE and ITERATE

```sql
LEAVE label;  -- Exit loop
ITERATE label;  -- Continue to next iteration
```

```rust
#[derive(Debug)]
pub enum ProcedureControlFlow {
    Leave(String),      // Exit loop with optional label
    Iterate(String),   // Continue to next iteration with optional label
    Return(Value),     // Return from procedure
}

fn execute_leave(&self, label: String) -> Result<!, ProcedureError> {
    Err(ProcedureControlFlow::Leave(label).into())
}

fn execute_iterate(&self, label: String) -> Result<!, ProcedureError> {
    Err(ProcedureControlFlow::Iterate(label).into())
}
```

### 3.7 CALL Statement (Nested Procedures)

```sql
CALL procedure_name(param1, param2, ...);
CALL procedure_name(param1, ...) INTO var;
```

```rust
fn execute_call(&self, stmt: &ProcedureStatement) -> Result<Option<Value>, ProcedureError> {
    if let ProcedureStatement::Call { procedure_name, args, into_var } = stmt {
        // Evaluate arguments
        let evaluated_args: Vec<Value> = args.iter()
            .map(|arg| self.eval_expression(arg))
            .collect::<Result<_, _>>()?;
        
        // Push new context for nested call
        let nested_ctx = ProcedureContext::new(procedure_name, evaluated_args);
        self.push_context(nested_ctx);
        
        // Execute nested procedure
        let result = self.execute_procedure(procedure_name, evaluated_args)?;
        
        // Pop context
        self.pop_context();
        
        // Handle INTO clause
        if let Some(var_name) = into_var {
            self.scope.variables.insert(var_name, result);
        }
        
        Ok(None)  // CALL doesn't return value unless INTO is used
    }
}
```

### 3.8 Embedded SQL Execution

For statements like `SELECT`, `INSERT`, `UPDATE`, `DELETE` within procedures:

```rust
fn execute_sql(&self, sql: &str) -> Result<ExecutorResult, ProcedureError> {
    // Parse the SQL statement
    let statement = parse(sql).map_err(|e| ProcedureError::SqlError(e))?;
    
    // Plan the statement using existing planner
    let plan = self.planner.create_plan(statement)
        .map_err(|e| ProcedureError::PlanningError(e))?;
    
    // Execute using existing executor
    let result = self.executor.execute(&plan)
        .map_err(|e| ProcedureError::ExecutionError(e))?;
    
    Ok(result)
}
```

### 3.9 SELECT ... INTO

```sql
SELECT col1, col2 INTO var1, var2 FROM table_name WHERE condition;
```

```rust
fn execute_select_into(&self, stmt: &ProcedureStatement) -> Result<(), ProcedureError> {
    if let ProcedureStatement::SelectInto { columns, into_vars, table, where_clause } = stmt {
        let sql = format!("SELECT {} FROM {} {}", columns.join(", "), table, where_clause.unwrap_or_default());
        let result = self.execute_sql(&sql)?;
        
        // Assign first row to variables
        if let Some(row) = result.rows.first() {
            for (i, var_name) in into_vars.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    self.scope.variables.insert(var_name.clone(), value.clone());
                }
            }
        }
        
        Ok(())
    }
}
```

---

## 4. Trigger Execution Enhancement

### 4.1 Current State

The current trigger executor only supports simple `SET NEW.col = value` patterns.

### 4.2 Enhanced Design

```rust
pub struct EnhancedTriggerExecutor<S: StorageEngine> {
    storage: Arc<S>,
    executor: Arc<ProcedureExecutor>,
}

impl<S: StorageEngine> EnhancedTriggerExecutor<S> {
    pub fn execute_trigger(&self, trigger: &TriggerInfo, context: &TriggerContext) -> Result<TriggerResult, TriggerError> {
        match &trigger.body_stmts {
            TriggerBody::Simple(sql) => self.execute_simple_trigger(sql, context),
            TriggerBody::Block(stmts) => self.execute_block_trigger(stmts, context),
        }
    }
    
    fn execute_block_trigger(&self, stmts: &[TriggerStatement], context: &TriggerContext) -> Result<TriggerResult, TriggerError> {
        let mut scope = VariableScope::with_trigger_context(context);
        
        for stmt in stmts {
            match stmt {
                TriggerStatement::Set { target, value } => {
                    let evaluated_value = self.eval_expression(value, &scope)?;
                    scope.set_variable(target, evaluated_value)?;
                }
                TriggerStatement::If { condition, then_body, else_body } => {
                    if self.eval_condition(condition, &scope)? {
                        self.execute_block_trigger(then_body, &mut scope)?;
                    } else if let Some(else_stmts) = else_body {
                        self.execute_block_trigger(else_stmts, &mut scope)?;
                    }
                }
                TriggerStatement::Insert { ... } => { /* DML in trigger */ }
                TriggerStatement::Update { ... } => { /* DML in trigger */ }
            }
        }
        
        Ok(TriggerResult::Modified(context.new_row.clone()))
    }
}
```

---

## 5. Integration with Existing System

### 5.1 Integration Points

1. **Catalog**: Procedures stored in `Catalog.stored_procedures`
2. **Parser**: `ProcedureStatement` enum already defined
3. **Executor**: `StoredProcExecutor` exists as placeholder
4. **Storage**: `StorageEngine` triggers API exists

### 5.2 Modified File Structure

```
crates/executor/src/
├── lib.rs                    # Add procedure module exports
├── stored_proc.rs            # Rename/replace with full implementation
├── procedure/
│   ├── mod.rs               # ProcedureExecutor
│   ├── context.rs           # ProcedureContext, VariableScope
│   ├── evaluator.rs         # ExpressionEvaluator
│   ├── statements.rs        # Statement executors
│   └── control_flow.rs      # Loop/branch handling
└── trigger.rs               # Enhanced with block body support
```

---

## 6. Error Handling

### 6.1 SIGNAL / RESIGNAL

```rust
pub enum ProcedureError {
    Signal(String),           // User-defined error
    SqlError(String),        // SQL execution error
    Return(Value),           // Return value (control flow)
    Leave(String),           // LEAVE label
    Iterate(String),         // ITERATE label
    DivisionByZero,
    // ... etc
}

fn execute_signal(&self, condition: &str) -> Result<!, ProcedureError> {
    Err(ProcedureError::Signal(condition.to_string()))
}
```

### 6.2 Handler Support

```sql
DECLARE CONTINUE HANDLER FOR SQLEXCEPTION
BEGIN
    -- error handling code
END;
```

```rust
pub struct Handler {
    condition: HandlerCondition,
    body: Vec<ProcedureStatement>,
}

pub enum HandlerCondition {
    Sqlexception,
    Sqlwarning,
    NotFound,
    SpecificError(String),
}

impl ProcedureExecutor {
    fn find_handler(&self, condition: &HandlerCondition) -> Option<&Handler> {
        self.handlers.iter().find(|h| h.condition == *condition)
    }
}
```

---

## 7. Testing Strategy

### 7.1 Unit Tests

- `test_declare_*`: Variable declaration and default values
- `test_if_*`: Conditional branching
- `test_while_*`: Loop execution
- `test_loop_*`: Infinite loop with LEAVE/ITERATE
- `test_call_*`: Nested procedure calls
- `test_return_*`: Return value propagation

### 7.2 Integration Tests

- Procedure with multiple control flow statements
- Nested procedure calls with parameter passing
- Trigger execution during DML operations

### 7.3 Test Cases

```rust
#[test]
fn test_procedure_with_if_and_variables() {
    let sql = "CREATE PROCEDURE test_if(x INT) BEGIN 
        DECLARE result INT DEFAULT 0;
        IF x > 0 THEN 
            SET result = 1;
        ELSEIF x < 0 THEN 
            SET result = -1;
        ELSE 
            SET result = 0;
        END IF;
        RETURN result;
    END";
    
    let result = execute_procedure("test_if", vec![Value::Integer(5)]);
    assert_eq!(result, Value::Integer(1));
}
```

---

## 8. Implementation Phases

### Phase 1: Core Infrastructure
- `ProcedureContext` and `VariableScope`
- Basic `ProcedureExecutor` skeleton
- `ExpressionEvaluator` trait

### Phase 2: Statement Executors
- `execute_declare()`
- `execute_set()`
- `execute_return()`

### Phase 3: Control Flow
- `execute_if()` with ELSEIF/ELSE
- `execute_while()`
- `execute_loop()` with LEAVE/ITERATE

### Phase 4: SQL Integration
- `execute_sql()` using existing planner/executor
- `execute_select_into()`
- `execute_call()` for nested procedures

### Phase 5: Error Handling
- SIGNAL/RESIGNAL implementation
- Handler support (CONTINUE, EXIT)
- Proper error propagation

### Phase 6: Trigger Enhancement
- Enhanced trigger body support
- Trigger execution integration with DML

---

## 9. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Recursive procedure calls | Stack overflow | Implement call depth limit |
| Infinite loops | Hang | Implement statement count limit |
| Performance regression | Slow queries | Benchmark before/after |
| Memory leaks | OOM | Clear VariableScope on exit |

---

## 10. Dependencies

- `crates/parser`: ProcedureStatement definitions
- `crates/planner`: SQL planning
- `crates/executor`: VolcanoExecutor for SQL execution
- `crates/storage`: Storage engine
- `crates/catalog`: Procedure registry
- `crates/types`: Value types

---

## 11. Open Questions

1. **Recursion**: Should procedures support recursive calls? If yes, implement tail-call optimization or explicit recursion limit.

2. **Transaction integration**: Should procedure execution be atomic? How to handle COMMIT/ROLLBACK within procedures?

3. **Cursor support**: For `SELECT * FROM table WHERE ...` followed by `FETCH`, need cursor management.

4. **OUT parameter semantics**: Should OUT parameters be returned in result set or via special mechanism?

---

## 12. References

- MySQL Stored Procedure Language: https://dev.mysql.com/doc/refman/8.0/en/stored-programs-defined.html
- PostgreSQL PL/pgSQL: https://www.postgresql.org/docs/current/plpgsql.html
- SQL:2003 Standard: https://www.iso.org/standard/34627.html
