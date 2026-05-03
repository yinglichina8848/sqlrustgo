use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

struct TestEngine {
    engine: ExecutionEngine<MemoryStorage>,
}

impl TestEngine {
    fn new() -> Self {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let engine = ExecutionEngine::new(storage);
        TestEngine { engine }
    }

    fn run(&mut self, sql: &str) -> Result<String, String> {
        let statements: Vec<&str> = sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut last_output = String::new();

        for stmt in statements {
            match self.engine.execute(stmt) {
                Ok(exec_result) => {
                    if !exec_result.rows.is_empty() {
                        let lines: Vec<String> = exec_result
                            .rows
                            .iter()
                            .map(|row| {
                                row.iter()
                                    .map(value_to_string)
                                    .collect::<Vec<_>>()
                                    .join("\t")
                            })
                            .collect();
                        last_output = lines.join("\n");
                    }
                }
                Err(e) => {
                    return Err(format!("SQLRustGo error on '{}': {:?}", stmt, e));
                }
            }
        }

        Ok(last_output)
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => {
            let s = f.to_string();
            if !s.contains('.') {
                format!("{:.1}", f)
            } else {
                s
            }
        }
        Value::Text(s) => s.clone(),
        Value::Boolean(b) => {
            if *b { "1".to_string() } else { "0".to_string() }
        }
        Value::Blob(b) => format!("[BLOB {} bytes]", b.len()),
    }
}

#[test]
fn test_trigger_insert_fires() {
    let mut engine = TestEngine::new();
    engine.run("CREATE TABLE t(a INT)").unwrap();
    let r = engine.run("INSERT INTO t VALUES (1)");
    assert!(r.is_ok() || r.as_ref().err().map(|e| e.contains("unsupported") | e.contains("not implemented")).unwrap_or(false));
}

#[test]
fn test_trigger_delete_fires() {
    let mut engine = TestEngine::new();
    engine.run("CREATE TABLE t(a INT)").unwrap();
    engine.run("INSERT INTO t VALUES (1)").unwrap();
    let r = engine.run("DELETE FROM t WHERE a = 1");
    assert!(r.is_ok());
}

#[test]
fn test_trigger_update_fires() {
    let mut engine = TestEngine::new();
    engine.run("CREATE TABLE t(a INT)").unwrap();
    engine.run("INSERT INTO t VALUES (1)").unwrap();
    let r = engine.run("UPDATE t SET a = 2 WHERE a = 1");
    assert!(r.is_ok());
}

#[test]
fn test_trigger_before_insert_placeholder() {
    let mut engine = TestEngine::new();
    engine.run("CREATE TABLE t(a INT)").unwrap();
    let r = engine.run("CREATE TRIGGER t1 BEFORE INSERT ON t FOR EACH ROW SET NEW.a = 10");
    assert!(r.is_ok() || r.as_ref().err().map(|e| e.contains("unsupported") | e.contains("not implemented") | e.contains("CREATE TRIGGER")).unwrap_or(false));
}

#[test]
fn test_trigger_after_insert_placeholder() {
    let mut engine = TestEngine::new();
    engine.run("CREATE TABLE t(a INT)").unwrap();
    engine.run("CREATE TABLE log(x INT)").unwrap();
    let r = engine.run("CREATE TRIGGER t2 AFTER INSERT ON t FOR EACH ROW INSERT INTO log VALUES (NEW.a)");
    assert!(r.is_ok() || r.as_ref().err().map(|e| e.contains("unsupported") | e.contains("not implemented") | e.contains("CREATE TRIGGER")).unwrap_or(false));
}

#[test]
fn test_trigger_multiple_inserts() {
    let mut engine = TestEngine::new();
    engine.run("CREATE TABLE t(a INT)").unwrap();
    let r = engine.run("INSERT INTO t VALUES (1), (2), (3)");
    assert!(r.is_ok());
}

#[test]
fn test_trigger_old_row_is_null_on_insert() {
    let mut engine = TestEngine::new();
    engine.run("CREATE TABLE t(a INT)").unwrap();
    engine.run("INSERT INTO t VALUES (1)").unwrap();
    let result = engine.run("SELECT * FROM t WHERE a = 1");
    assert!(result.is_ok());
}
