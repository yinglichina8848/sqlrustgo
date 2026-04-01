//! MySQL Compatibility Integration Tests (Issue #1135)
//!
//! Tests for SHOW PROCESSLIST and KILL statement functionality

use sqlrustgo_parser::parse;
use sqlrustgo_parser::{KillStatement, KillType, Statement};
use sqlrustgo_security::{Session, SessionManager, SessionStatus};

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
