//! MySQL Compatibility Integration Tests (Issue #1135)
//!
//! Tests for SHOW PROCESSLIST and KILL statement functionality

use sqlrustgo_parser::parse;
use sqlrustgo_parser::{KillStatement, KillType, Statement};
use sqlrustgo_security::{Session, SessionManager, SessionStatus};
use sqlrustgo_storage::{MemoryStorage, StorageEngine};
use std::sync::Arc;

#[test]
fn test_parse_show_processlist_integration() {
    let result = parse("SHOW PROCESSLIST");
    assert!(result.is_ok());

    match result.unwrap() {
        Statement::ShowProcesslist => {}
        _ => panic!("Expected SHOW PROCESSLIST statement"),
    }
}

#[test]
fn test_parse_kill_connection_integration() {
    let result = parse("KILL 12345");
    assert!(result.is_ok());

    match result.unwrap() {
        Statement::Kill(k) => {
            assert_eq!(k.process_id, 12345);
            assert_eq!(k.kill_type, KillType::Connection);
        }
        _ => panic!("Expected KILL statement"),
    }
}

#[test]
fn test_parse_kill_connection_explicit_integration() {
    let result = parse("KILL CONNECTION 54321");
    assert!(result.is_ok());

    match result.unwrap() {
        Statement::Kill(k) => {
            assert_eq!(k.process_id, 54321);
            assert_eq!(k.kill_type, KillType::Connection);
        }
        _ => panic!("Expected KILL CONNECTION statement"),
    }
}

#[test]
fn test_parse_kill_query_integration() {
    let result = parse("KILL QUERY 99999");
    assert!(result.is_ok());

    match result.unwrap() {
        Statement::Kill(k) => {
            assert_eq!(k.process_id, 99999);
            assert_eq!(k.kill_type, KillType::Query);
        }
        _ => panic!("Expected KILL QUERY statement"),
    }
}

#[test]
fn test_session_manager_processlist_rows() {
    let manager = SessionManager::new();

    let id1 = manager.create_session("alice".to_string(), "127.0.0.1".to_string());
    let id2 = manager.create_session("bob".to_string(), "127.0.0.2".to_string());
    let id3 = manager.create_session("charlie".to_string(), "192.168.1.1".to_string());

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    let rows = manager.get_processlist_rows();
    assert_eq!(rows.len(), 3);

    let users: Vec<_> = rows.iter().map(|r| r.user.as_str()).collect();
    assert!(users.contains(&"alice"));
    assert!(users.contains(&"bob"));
    assert!(users.contains(&"charlie"));
}

#[test]
fn test_processlist_row_active_session() {
    let manager = SessionManager::new();
    let session_id = manager.create_session("testuser".to_string(), "10.0.0.1".to_string());

    let session = manager.get_session(session_id).unwrap();
    let row = session.to_processlist_row();

    assert_eq!(row.id, session_id);
    assert_eq!(row.user, "testuser");
    assert_eq!(row.host, "10.0.0.1");
    assert_eq!(row.command, "Query");
    assert!(row.db.is_none());
    assert_eq!(row.state, "");
}

#[test]
fn test_processlist_row_closed_session() {
    let manager = SessionManager::new();
    let session_id = manager.create_session("testuser".to_string(), "10.0.0.1".to_string());

    manager.close_session(session_id);

    let session = manager.get_session(session_id).unwrap();
    let row = session.to_processlist_row();

    assert_eq!(row.id, session_id);
    assert_eq!(row.user, "testuser");
    assert_eq!(row.command, "Dead");
}

#[test]
fn test_processlist_row_idle_session() {
    let session = Session::new(1, "testuser".to_string(), "10.0.0.1".to_string());
    let mut session = session;
    session.set_idle();

    let row = session.to_processlist_row();
    assert_eq!(row.command, "Sleep");
}

#[test]
fn test_kill_statement_structure() {
    let kill_stmt = KillStatement {
        process_id: 42,
        kill_type: KillType::Connection,
    };

    assert_eq!(kill_stmt.process_id, 42);
    assert_eq!(kill_stmt.kill_type, KillType::Connection);
}

#[test]
fn test_kill_query_statement_structure() {
    let kill_stmt = KillStatement {
        process_id: 123,
        kill_type: KillType::Query,
    };

    assert_eq!(kill_stmt.process_id, 123);
    assert_eq!(kill_stmt.kill_type, KillType::Query);
}

#[test]
fn test_session_manager_close_session() {
    let manager = SessionManager::new();
    let session_id = manager.create_session("alice".to_string(), "127.0.0.1".to_string());

    manager.close_session(session_id);

    let session = manager.get_session(session_id).unwrap();
    assert_eq!(session.status, SessionStatus::Closed);
}

#[test]
fn test_session_manager_get_active_sessions() {
    let manager = SessionManager::new();
    let id1 = manager.create_session("alice".to_string(), "127.0.0.1".to_string());
    let _id2 = manager.create_session("bob".to_string(), "127.0.0.2".to_string());

    manager.close_session(id1);

    let active = manager.get_active_sessions();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].user, "bob");
}

#[test]
fn test_session_manager_get_user_sessions() {
    let manager = SessionManager::new();
    let _id1 = manager.create_session("alice".to_string(), "127.0.0.1".to_string());
    let _id2 = manager.create_session("alice".to_string(), "127.0.0.2".to_string());
    let _id3 = manager.create_session("bob".to_string(), "127.0.0.3".to_string());

    let alice_sessions = manager.get_user_sessions("alice");
    assert_eq!(alice_sessions.len(), 2);

    let bob_sessions = manager.get_user_sessions("bob");
    assert_eq!(bob_sessions.len(), 1);
}

#[test]
fn test_session_manager_cleanup_closed() {
    let manager = SessionManager::new();
    let id1 = manager.create_session("alice".to_string(), "127.0.0.1".to_string());
    let _id2 = manager.create_session("bob".to_string(), "127.0.0.2".to_string());

    manager.close_session(id1);

    let removed = manager.cleanup_closed_sessions();
    assert_eq!(removed, 1);
    assert_eq!(manager.get_session_count(), 1);
}

#[test]
fn test_processlist_row_with_database() {
    let mut session = Session::new(1, "alice".to_string(), "127.0.0.1".to_string());
    session.database = Some("testdb".to_string());

    let row = session.to_processlist_row();
    assert_eq!(row.db, Some("testdb".to_string()));
}

#[test]
fn test_parse_information_schema_processlist() {
    let result = parse("SELECT * FROM information_schema.processlist");
    assert!(result.is_ok());

    match result.unwrap() {
        Statement::Select(s) => {
            assert_eq!(s.table, "information_schema.processlist");
            assert_eq!(s.columns.len(), 1);
            assert_eq!(s.columns[0].name, "*");
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_information_schema_processlist_with_columns() {
    let result = parse("SELECT ID, USERNAME, HOST FROM information_schema.processlist");
    assert!(result.is_ok());

    match result.unwrap() {
        Statement::Select(s) => {
            assert_eq!(s.table, "information_schema.processlist");
            assert_eq!(s.columns.len(), 3);
            assert_eq!(s.columns[0].name, "ID");
            assert_eq!(s.columns[1].name, "USERNAME");
            assert_eq!(s.columns[2].name, "HOST");
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_memory_storage_cancel_flag() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let mut storage = MemoryStorage::new();
    let cancel_flag = Arc::new(AtomicBool::new(false));

    storage.set_cancel_flag(cancel_flag.clone());

    assert!(storage.cancel_flag().is_some());

    let result = storage.check_cancelled();
    assert!(result.is_ok());

    cancel_flag.store(true, Ordering::SeqCst);

    let result = storage.check_cancelled();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Query cancelled"));
}

#[test]
fn test_memory_storage_scan_with_cancel() {
    use sqlrustgo_types::Value;
    use std::sync::atomic::{AtomicBool, Ordering};

    let mut storage = MemoryStorage::new();
    let cancel_flag = Arc::new(AtomicBool::new(false));

    storage.set_cancel_flag(cancel_flag.clone());

    storage
        .create_table(&sqlrustgo_storage::TableInfo {
            name: "test_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        })
        .unwrap();

    let records = vec![vec![Value::Integer(1)], vec![Value::Integer(2)]];
    storage.insert("test_table", records).unwrap();

    let result = storage.scan("test_table");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);

    cancel_flag.store(true, Ordering::SeqCst);

    let result = storage.scan("test_table");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Query cancelled"));
}

#[test]
fn test_memory_storage_scan_batch_with_cancel() {
    use sqlrustgo_types::Value;
    use std::sync::atomic::{AtomicBool, Ordering};

    let mut storage = MemoryStorage::new();
    let cancel_flag = Arc::new(AtomicBool::new(false));

    storage.set_cancel_flag(cancel_flag.clone());

    storage
        .create_table(&sqlrustgo_storage::TableInfo {
            name: "test_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        })
        .unwrap();

    let records: Vec<Vec<Value>> = (0..100).map(|i| vec![Value::Integer(i)]).collect();
    storage.insert("test_table", records).unwrap();

    cancel_flag.store(true, Ordering::SeqCst);

    let result = storage.scan_batch("test_table", 0, 10);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Query cancelled"));
}

#[test]
fn test_execution_engine_with_session_manager() {
    use sqlrustgo::{ExecutionEngine, MemoryStorage};
    use std::sync::{Arc, RwLock};

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let session_manager = Arc::new(SessionManager::new());

    let session_id =
        session_manager.create_session("testuser".to_string(), "127.0.0.1".to_string());

    let engine = ExecutionEngine::new_with_session(storage, session_manager.clone(), session_id);

    assert!(engine.session_id().is_some());
    assert_eq!(engine.session_id().unwrap(), session_id);
}

#[test]
fn test_execution_engine_kill_query_via_session() {
    use sqlrustgo::{ExecutionEngine, MemoryStorage};
    use std::sync::{Arc, RwLock};

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let session_manager = Arc::new(SessionManager::new());

    let session_id1 = session_manager.create_session("user1".to_string(), "127.0.0.1".to_string());
    let session_id2 = session_manager.create_session("user1".to_string(), "127.0.0.2".to_string());

    let mut engine =
        ExecutionEngine::new_with_session(storage, session_manager.clone(), session_id1);

    let kill_stmt = Statement::Kill(KillStatement {
        process_id: session_id2,
        kill_type: KillType::Query,
    });

    let result = engine.execute(kill_stmt);
    assert!(result.is_ok());
}

#[test]
fn test_execution_engine_kill_self_prevention() {
    use sqlrustgo::{ExecutionEngine, MemoryStorage};
    use std::sync::{Arc, RwLock};

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let session_manager = Arc::new(SessionManager::new());

    let session_id = session_manager.create_session("user1".to_string(), "127.0.0.1".to_string());

    let mut engine =
        ExecutionEngine::new_with_session(storage, session_manager.clone(), session_id);

    let kill_stmt = Statement::Kill(KillStatement {
        process_id: session_id,
        kill_type: KillType::Query,
    });

    let result = engine.execute(kill_stmt);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot kill self"));
}

#[test]
fn test_execution_engine_kill_nonexistent_session() {
    use sqlrustgo::{ExecutionEngine, MemoryStorage};
    use std::sync::{Arc, RwLock};

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let session_manager = Arc::new(SessionManager::new());

    let session_id = session_manager.create_session("user1".to_string(), "127.0.0.1".to_string());

    let mut engine =
        ExecutionEngine::new_with_session(storage, session_manager.clone(), session_id);

    let kill_stmt = Statement::Kill(KillStatement {
        process_id: 99999,
        kill_type: KillType::Query,
    });

    let result = engine.execute(kill_stmt);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown thread id"));
}

#[test]
fn test_execution_engine_kill_different_user_without_privilege() {
    use sqlrustgo::{ExecutionEngine, MemoryStorage};
    use std::sync::{Arc, RwLock};

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let session_manager = Arc::new(SessionManager::new());

    let session_id1 = session_manager.create_session("user1".to_string(), "127.0.0.1".to_string());
    let session_id2 = session_manager.create_session("user2".to_string(), "127.0.0.2".to_string());

    let mut engine =
        ExecutionEngine::new_with_session(storage, session_manager.clone(), session_id1);

    let kill_stmt = Statement::Kill(KillStatement {
        process_id: session_id2,
        kill_type: KillType::Query,
    });

    let result = engine.execute(kill_stmt);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Access denied"));
}

#[test]
fn test_execution_engine_kill_connection() {
    use sqlrustgo::{ExecutionEngine, MemoryStorage};
    use std::sync::{Arc, RwLock};

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let session_manager = Arc::new(SessionManager::new());

    let session_id1 = session_manager.create_session("user1".to_string(), "127.0.0.1".to_string());
    let session_id2 = session_manager.create_session("user1".to_string(), "127.0.0.2".to_string());

    let mut engine =
        ExecutionEngine::new_with_session(storage, session_manager.clone(), session_id1);

    let kill_stmt = Statement::Kill(KillStatement {
        process_id: session_id2,
        kill_type: KillType::Connection,
    });

    let result = engine.execute(kill_stmt);
    assert!(result.is_ok());

    let target_session = session_manager.get_session(session_id2);
    assert!(target_session.is_some());
    assert_eq!(target_session.unwrap().status, SessionStatus::Closed);
}
