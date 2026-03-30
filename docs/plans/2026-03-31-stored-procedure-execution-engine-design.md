# Stored Procedure & Trigger Execution Engine Design

> **Version**: 2.0  
> **Date**: 2026-03-31  
> **Status**: Draft  
> **Related Issue**: #1164  
> **Review Status**: Architecture reviewed, adjustments incorporated

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

### 1.3 Classification

This is **SQL Runtime Interpreter Subsystem** implementation, not a simple feature extension.

Impact scope:
- executor pipeline
- expression evaluator
- variable scope engine
- control-flow interpreter
- error propagation semantics
- trigger integration into DML pipeline

Complexity level: MySQL Stored Program Runtime / PostgreSQL PL/pgSQL interpreter

---

## 2. Architecture

### 2.1 High-Level Design

```
┌─────────────────────────────────────────────────────────────────┐
│                      Session / Connection                         │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    ProcedureRuntime                         │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │              ProcedureContext                         │  │  │
│  │  │  - scopes: Vec<VariableScope>                      │  │  │
│  │  │  - labels: Vec<String>                             │  │  │
│  │  │  - handler_stack: Vec<HandlerFrame>                │  │  │
│  │  │  - call_stack_depth: usize                         │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │                           │                                 │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │           ExpressionEvaluator (trait)                │  │  │
│  │  │  fn eval_expr(&self, expr: &Expr, ctx: &mut ProcedureContext) -> Result<Value>  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │                           │                                 │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │           StatementInterpreter                       │  │  │
│  │  │  - execute_declare()                               │  │  │
│  │  │  - execute_if()                                    │  │  │
│  │  │  - execute_while()                                 │  │  │
│  │  │  - execute_loop()                                  │  │  │
│  │  │  - execute_call()                                  │  │  │
│  │  │  - execute_sql()                                   │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                           │                                      │
│                           ▼                                      │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    SQL Executor Bridge                      │  │
│  │         session.execute_with_context(stmt, ctx)           │  │
│  └───────────────────────────────────────────────────────────┘  │
│                           │                                      │
│                           ▼                                      │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    VolcanoExecutor                          │  │
│  │  (SeqScan, Projection, Aggregate, Join, etc.)            │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

**Key Design Principle**: ProcedureRuntime and TriggerRuntime are UNIFIED, not separate.

### 2.2 Unified Runtime Design

```
┌─────────────────────────────────────────────┐
│              ProcedureRuntime                │
│    (Unified runtime for procedure + trigger)│
├─────────────────────────────────────────────┤
│ - context: ProcedureContext               │
│ - expr_eval: Arc<dyn ExpressionEvaluator> │
│ - sql_bridge: Session                    │
└─────────────────────────────────────────────┘
                      ▲
                      │ triggers are implicit procedures
                      │
┌─────────────────────────────────────────────┐
│              TriggerRuntime                 │
│   (thin wrapper around ProcedureRuntime)    │
└─────────────────────────────────────────────┘
```

NOT two separate runtimes. This is the correct architecture (PostgreSQL PL/pgSQL model).

---

## 3. Core Data Structures

### 3.1 ProcedureContext (ENHANCED)

```rust
pub struct ProcedureContext {
    /// Stack of variable scopes (block-level)
    pub scopes: Vec<VariableScope>,
    /// Label stack for LEAVE/ITERATE
    pub labels: Vec<String>,
    /// Handler stack for error handling
    pub handler_stack: Vec<HandlerFrame>,
    /// Current call stack depth (for recursion protection)
    pub call_stack_depth: usize,
    /// Maximum allowed call depth
    pub max_call_depth: usize,
}

pub struct VariableScope {
    pub variables: HashMap<String, Value>,
}

pub struct HandlerFrame {
    pub condition: HandlerCondition,
    pub body: Vec<ProcedureStatement>,
    pub action: HandlerAction,
}

pub enum HandlerCondition {
    Sqlexception,
    Sqlwarning,
    NotFound,
    SpecificError(String),
}

pub enum HandlerAction {
    Continue,
    Exit,
}
```

**Critical**: `scopes: Vec<VariableScope>` enables block-level scoping.

### 3.2 Block-Level Scope Semantics

```rust
// Entering a BEGIN block
fn enter_block(&mut self) {
    self.scopes.push(VariableScope::new());
}

// Exiting a BEGIN block
fn exit_block(&mut self) {
    self.scopes.pop();
}

// Variable declaration
fn declare_variable(&mut self, name: &str, value: Value) {
    // Always declare in current (top) scope
    self.scopes.last_mut().unwrap().variables.insert(name.to_string(), value);
}

// Variable lookup (search from top to bottom)
fn lookup_variable(&self, name: &str) -> Option<&Value> {
    for scope in self.scopes.iter().rev() {
        if let Some(v) = scope.variables.get(name) {
            return Some(v);
        }
    }
    None
}

// Variable assignment (assign to innermost scope where variable exists)
fn assign_variable(&mut self, name: &str, value: Value) -> Result<(), ProcedureError> {
    for scope in self.scopes.iter_mut().rev() {
        if scope.variables.contains_key(name) {
            scope.variables.insert(name.to_string(), value);
            return Ok(());
        }
    }
    Err(ProcedureError::UnknownVariable(name.to_string()))
}
```

### 3.3 ExpressionEvaluator Trait

**CRITICAL DESIGN RULE**: Must reuse SQL expression engine, NOT create a separate one.

```rust
pub trait ExpressionEvaluator: Send + Sync {
    fn eval_expr(
        &self,
        expr: &Expr,
        ctx: &mut ProcedureContext,
    ) -> Result<Value, ProcedureError>;
    
    fn eval_condition(
        &self,
        expr: &Expr,
        ctx: &mut ProcedureContext,
    ) -> Result<bool, ProcedureError>;
}
```

**Why this matters**: Avoids "two expression engines problem" - a catastrophic design error in database projects.

### 3.4 Control Flow Structures

```rust
pub enum ProcedureControlFlow {
    Continue,
    Leave(String),      // Exit loop with optional label
    Iterate(String),    // Continue to next iteration with optional label
    Return(Value),      // Return from procedure
    Error(ProcedureError),
}

pub struct LoopFrame {
    pub label: Option<String>,
    pub body: Vec<ProcedureStatement>,
}
```

---

## 4. Execution Flow

### 4.1 Procedure Execution Lifecycle

```
CALL proc(args)
       │
       ▼
┌──────────────────┐
│ Lookup metadata   │
│ (from Catalog)   │
└──────────────────┘
       │
       ▼
┌──────────────────┐
│ Create Runtime    │
│ - push scope     │
│ - bind params    │
└──────────────────┘
       │
       ▼
┌──────────────────┐
│ Execute Stmts     │◄─────────────────┐
│ sequentially      │                  │
└──────────────────┘                  │
       │                               │
       ├──[IF]── evaluate condition ────┤
       ├──[WHILE]── evaluate condition ─┤
       ├──[LOOP]─── check LEAVE ──────┤
       ├──[CALL]─── recursive call ────┤
       ├──[RETURN]── exit proc ───────┘
       │
       ▼
┌──────────────────┐
│ Pop scope        │
│ Return value     │
└──────────────────┘
```

### 4.2 Label Stack for LEAVE/ITERATE

```rust
impl ProcedureRuntime {
    fn execute_loop(&mut self, label: Option<String>, body: Vec<ProcedureStatement>) -> Result<(), ProcedureError> {
        // Push label onto stack
        if let Some(ref lbl) = label {
            self.context.labels.push(lbl.clone());
        }
        
        loop {
            match self.execute_statements(body.clone())? {
                ProcedureControlFlow::Leave(lbl) => {
                    // If label matches, exit; if no label, exit innermost
                    if lbl.is_none() || (label.is_some() && label == Some(lbl)) {
                        break;
                    }
                    return Err(ProcedureError::UnhandledLeave(lbl));
                }
                ProcedureControlFlow::Iterate(lbl) => {
                    if lbl.is_none() || (label.is_some() && label == Some(lbl)) {
                        continue;
                    }
                    return Err(ProcedureError::UnhandledIterate(lbl));
                }
                ProcedureControlFlow::Return(v) => {
                    return Err(ProcedureError::Return(v));
                }
                ProcedureControlFlow::Continue => {
                    continue;
                }
                ProcedureControlFlow::Error(e) => {
                    // Check handler stack
                    if let Some(handler) = self.find_handler(&e) {
                        self.execute_handler(handler)?;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        
        // Pop label
        if label.is_some() {
            self.context.labels.pop();
        }
        
        Ok(())
    }
}
```

### 4.3 CALL with Recursion Protection

```rust
const MAX_CALL_DEPTH: usize = 32;

impl ProcedureRuntime {
    fn execute_call(&mut self, name: &str, args: Vec<Value>) -> Result<Option<Value>, ProcedureError> {
        // Recursion protection
        if self.context.call_stack_depth >= MAX_CALL_DEPTH {
            return Err(ProcedureError::MaxCallDepthExceeded(MAX_CALL_DEPTH));
        }
        
        // Increment call depth
        self.context.call_stack_depth += 1;
        
        // Push new context for nested call
        self.push_scope();
        
        let result = self.execute_procedure(name, args);
        
        // Pop context
        self.pop_scope();
        
        // Decrement call depth
        self.context.call_stack_depth -= 1;
        
        result
    }
}
```

---

## 5. SQL Bridge (Critical Integration Point)

### 5.1 SQL Executor Bridge Design

**CRITICAL**: Must use `session.execute_with_context()` NOT `planner.execute()` directly.

```rust
impl ProcedureRuntime {
    fn execute_sql(&mut self, stmt: &Statement) -> Result<QueryResult, ProcedureError> {
        // Use session bridge to maintain transaction scope
        self.session.execute_with_context(stmt, &mut self.context)
            .map_err(|e| ProcedureError::SqlError(e.to_string()))
    }
}
```

**Why this matters**: Direct planner.execute() breaks transaction scope - a severe bug.

### 5.2 SELECT INTO Semantics

**MySQL-compatible behavior**:

```rust
fn execute_select_into(&mut self, stmt: &SelectIntoStatement) -> Result<(), ProcedureError> {
    let result = self.execute_sql(&stmt.select)?;
    
    match result.rows.len() {
        0 => Err(ProcedureError::NoDataFound),
        1 => {
            let row = &result.rows[0];
            for (i, var_name) in stmt.into_vars.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    self.assign_variable(var_name, value.clone())?;
                }
            }
            Ok(())
        }
        _ => Err(ProcedureError::TooManyRows),
    }
}
```

**Required semantics**:
- 0 rows → error (NOTFOUND condition)
- 1 row → assign values
- >1 rows → error (TOO_MANY_ROWS condition)

---

## 6. Trigger Integration

### 6.1 Unified Runtime for Triggers

Triggers are implicit procedures - use same runtime:

```rust
pub struct TriggerRuntime {
    runtime: ProcedureRuntime,
}

impl TriggerRuntime {
    pub fn execute_trigger(&self, trigger: &TriggerInfo, context: &TriggerContext) -> Result<TriggerResult, TriggerError> {
        // Convert trigger body to ProcedureStatement
        let stmts = self.parse_trigger_body(&trigger.body)?;
        
        // Execute using unified runtime
        self.runtime.execute_statements(stmts)
        
        // Return modified row if any
    }
}
```

### 6.2 DML Pipeline Integration

```
INSERT executor
       │
       ├──► BEFORE triggers ──► modify NEW.* ──┐
       │                                      │
       ├──► Row modification                   │
       │                                      │
       └──► AFTER triggers ────────────────────┘
       
TriggerRuntime executes at each point, integrated into DML executor pipeline.
```

### 6.3 Phase 6 Decomposition

```
Phase 6A: BEFORE trigger runtime
Phase 6B: AFTER trigger runtime  
Phase 6C: Statement-level trigger support (future)
```

---

## 7. Error Handling

### 7.1 SIGNAL / RESIGNAL

```rust
#[derive(Debug)]
pub enum ProcedureError {
    Signal(String),
    SqlError(String),
    Return(Value),
    Leave(String),
    Iterate(String),
    NoDataFound,
    TooManyRows,
    MaxCallDepthExceeded(usize),
    UnknownVariable(String),
    DivisionByZero,
}

impl ProcedureRuntime {
    fn execute_signal(&self, condition: &str) -> Result<!, ProcedureError> {
        Err(ProcedureError::Signal(condition.to_string()))
    }
}
```

### 7.2 Handler Execution

```rust
impl ProcedureRuntime {
    fn find_handler(&self, error: &ProcedureError) -> Option<&HandlerFrame> {
        for handler in self.context.handler_stack.iter().rev() {
            match &handler.condition {
                HandlerCondition::Sqlexception => {
                    if matches!(error, ProcedureError::SqlError(_)) {
                        return Some(handler);
                    }
                }
                HandlerCondition::NotFound => {
                    if matches!(error, ProcedureError::NoDataFound) {
                        return Some(handler);
                    }
                }
                HandlerCondition::SpecificError(code) => {
                    // Match specific error code
                }
                _ => {}
            }
        }
        None
    }
    
    fn execute_handler(&mut self, handler: &HandlerFrame) -> Result<ProcedureControlFlow, ProcedureError> {
        match handler.action {
            HandlerAction::Continue => {
                // Execute handler body then continue
                self.execute_statements(handler.body.clone())?;
                Ok(ProcedureControlFlow::Continue)
            }
            HandlerAction::Exit => {
                // Execute handler body then exit current scope
                self.execute_statements(handler.body.clone())?;
                Err(ProcedureControlFlow::Leave(String::new()))
            }
        }
    }
}
```

---

## 8. Implementation Phases

### Phase 1: Core Infrastructure
- [x] `ProcedureContext` with enhanced structure (scopes, labels, handlers, call_depth)
- [x] `VariableScope` with block-level semantics
- [x] `ExpressionEvaluator` trait (reusing SQL expression engine)
- [x] `ProcedureRuntime` skeleton
- [x] `MAX_CALL_DEPTH` constant

### Phase 2: Basic Statement Executors
- [ ] `execute_declare()` - Block-level variable declaration
- [ ] `execute_set()` - Variable assignment
- [ ] `execute_return()` - Return value from procedure
- [ ] `execute_begin_end()` - Block scoping

### Phase 3: Control Flow
- [ ] `execute_if()` - IF/ELSEIF/ELSE branching
- [ ] `execute_while()` - WHILE loop
- [ ] `execute_loop()` - Infinite loop

### Phase 3.5: Label Engine (NEW PHASE)
- [ ] Label stack management
- [ ] `execute_leave()` with label matching
- [ ] `execute_iterate()` with label matching

### Phase 4: SQL Integration
- [ ] `execute_sql()` via session bridge
- [ ] `execute_select_into()` with proper row count checking
- [ ] `execute_call()` with recursion protection
- [ ] `execute_insert/update/delete()`

### Phase 5: Error Handling
- [ ] SIGNAL/RESIGNAL implementation
- [ ] Handler support (CONTINUE, EXIT HANDLER FOR SQLEXCEPTION)
- [ ] Condition handling (NOT FOUND, SQLWARNING)
- [ ] Proper error propagation

### Phase 6: Trigger Integration
- [ ] Phase 6A: BEFORE trigger runtime
- [ ] Phase 6B: AFTER trigger runtime
- [ ] Phase 6C: Statement-level trigger support (future)

---

## 9. File Structure

```
crates/executor/src/
├── lib.rs                    # Add procedure module exports
├── procedure/
│   ├── mod.rs               # ProcedureRuntime, ProcedureContext
│   ├── context.rs           # Enhanced context with scopes/labels/handlers
│   ├── scope.rs             # VariableScope with block semantics
│   ├── evaluator.rs         # ExpressionEvaluator trait
│   ├── statements.rs        # Statement executors (declare, set, return)
│   ├── control_flow.rs       # IF, WHILE, LOOP, LEAVE, ITERATE
│   ├── sql_bridge.rs         # SQL executor bridge
│   ├── error.rs             # ProcedureError enum
│   └── handler.rs           # SIGNAL, RESIGNAL, handlers
└── trigger.rs               # Enhanced (or deprecated in favor of unified runtime)
```

---

## 10. Dependencies

| Issue | Depends On |
|-------|-----------|
| Phase 1-5 (Procedure) | None (can start immediately) |
| Trigger privilege check | Issue #956 (RBAC) |
| Full trigger integration | Phase 1-5 complete |

**Recommended order**: Issue #956 (RBAC) → Procedure Phases 1-5 → Trigger Phase 6

---

## 11. RBAC Integration

```
RBAC
  │
  ▼
permission check
  │
  ▼
ProcedureRuntime.execute()
  │
  ▼
(Procedures execute with caller's privileges)
```

Stored procedure privilege escalation must be prevented by RBAC framework.

---

## 12. Testing Strategy

### Unit Tests
- Block-level scope shadowing
- Label matching for LEAVE/ITERATE
- SELECT INTO row count handling
- Recursion depth protection
- Handler execution

### Integration Tests
- Procedure with multiple control flow statements
- Nested procedure calls with parameter passing
- Trigger execution during DML operations

---

## 13. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Recursive procedure calls | Stack overflow | MAX_CALL_DEPTH = 32 |
| Infinite loops | Hang | Statement count limit |
| Two expression engines | Maintenance nightmare | Reuse SQL expression engine |
| Transaction scope broken | Data corruption | Use session bridge, not direct planner |

---

## 14. Key Design Principles

1. **UNIFIED runtime**: ProcedureRuntime = TriggerRuntime (triggers are implicit procedures)
2. **ExpressionEvaluator trait**: Reuse SQL expression engine, never create separate one
3. **Block-level scoping**: scopes Vec<VariableScope>, push on BEGIN, pop on END
4. **Label stack**: LEAVE/ITERATE with label matching
5. **SQL bridge via session**: Maintain transaction scope
6. **SELECT INTO semantics**: 0 rows → error, 1 row → assign, >1 rows → error
7. **Recursion protection**: MAX_CALL_DEPTH limit

---

## 15. Related Issues

- Issue #1162: Parser support for stored procedure control flow (merged)
- Issue #956: RBAC implementation (recommended to complete first)
- Issue #1164: This implementation issue

---

## 16. References

- MySQL Stored Procedure Language: https://dev.mysql.com/doc/refman/8.0/en/stored-programs-defined.html
- PostgreSQL PL/pgSQL: https://www.postgresql.org/docs/current/plpgsql.html
- SQL:2003 Standard: https://www.iso.org/standard/34627.html
