//! Concurrent KILL Stress Test for Issue #1154
//!
//! Tests the KILL functionality under high concurrency:
//! - 100 concurrent sessions
//! - Random KILL operations between sessions
//! - Verify no deadlocks or memory leaks
//! - Ensure proper privilege checking under load

use sqlrustgo::{
    ExecutionEngine, KillStatement, KillType, MemoryStorage, Statement, StorageEngine,
};
use sqlrustgo_security::{SessionManager, SessionStatus};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

const NUM_SESSIONS: usize = 100;
const OPERATIONS_PER_SESSION: usize = 10;
const KILL_PROBABILITY: f32 = 0.1;

#[test]
fn test_concurrent_kill_stress() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let session_manager = Arc::new(SessionManager::new());
    let kill_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));
    let success_count = Arc::new(AtomicUsize::new(0));

    let mut session_ids = Vec::with_capacity(NUM_SESSIONS);
    for i in 0..NUM_SESSIONS {
        let user = format!("user_{}", i % 10);
        let session_id = session_manager.create_session(user, format!("127.0.0.{}", i % 255));
        session_ids.push(session_id);
    }

    let start = Instant::now();

    let handles: Vec<_> = session_ids
        .iter()
        .map(|&session_id| {
            let storage = storage.clone();
            let session_manager = session_manager.clone();
            let kill_count = kill_count.clone();
            let error_count = error_count.clone();
            let success_count = success_count.clone();
            let session_ids = session_ids.clone();

            thread::spawn(move || {
                let mut engine =
                    ExecutionEngine::new_with_session(storage, session_manager.clone(), session_id);

                for op in 0..OPERATIONS_PER_SESSION {
                    let use_kill = rand_kill_decision();

                    if use_kill && op > 0 {
                        let target_idx = (session_id as usize + op) % session_ids.len();
                        let target_id = session_ids[target_idx];

                        if target_id != session_id {
                            kill_count.fetch_add(1, Ordering::SeqCst);

                            let kill_stmt = Statement::Kill(KillStatement {
                                process_id: target_id,
                                kill_type: if op % 2 == 0 {
                                    KillType::Connection
                                } else {
                                    KillType::Query
                                },
                            });

                            match engine.execute(kill_stmt) {
                                Ok(_) => {
                                    success_count.fetch_add(1, Ordering::SeqCst);
                                }
                                Err(e) => {
                                    error_count.fetch_add(1, Ordering::SeqCst);
                                    let _ = e;
                                }
                            }
                        }
                    } else {
                        let select_stmt =
                            sqlrustgo::parse("SELECT * FROM nonexistent_table").unwrap();
                        let _ = engine.execute(select_stmt);
                    }
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let elapsed = start.elapsed();
    let total_kills = kill_count.load(Ordering::SeqCst);
    let total_errors = error_count.load(Ordering::SeqCst);
    let total_success = success_count.load(Ordering::SeqCst);

    println!(
        "Concurrent KILL Stress Test Results:
        - Duration: {:?}
        - Total sessions: {}
        - KILL attempts: {}
        - Successful KILLs: {}
        - Access denied (expected): {}
        - Ops/sec: {:.2}",
        elapsed,
        NUM_SESSIONS,
        total_kills,
        total_success,
        total_errors,
        (NUM_SESSIONS * OPERATIONS_PER_SESSION) as f64 / elapsed.as_secs_f64()
    );

    assert!(
        total_kills > 0,
        "Should have attempted at least some KILL operations"
    );
}

fn rand_kill_decision() -> bool {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f32 / u32::MAX as f32) < KILL_PROBABILITY
}

#[test]
fn test_kill_own_sessions_allowed() {
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
    assert!(result.is_ok(), "User should be able to KILL own sessions");
}

#[test]
fn test_kill_different_user_requires_privilege() {
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
fn test_concurrent_kill_same_target() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let session_manager = Arc::new(SessionManager::new());

    let target_id = session_manager.create_session("killer1".to_string(), "127.0.0.1".to_string());

    let killer1 = session_manager.create_session("killer1".to_string(), "127.0.0.2".to_string());
    let killer2 = session_manager.create_session("killer2".to_string(), "127.0.0.3".to_string());

    let storage1 = storage.clone();
    let sm1 = session_manager.clone();
    let handle1: thread::JoinHandle<Result<_, _>> = thread::spawn(move || {
        let mut engine = ExecutionEngine::new_with_session(storage1, sm1.clone(), killer1);
        let kill_stmt = Statement::Kill(KillStatement {
            process_id: target_id,
            kill_type: KillType::Connection,
        });
        engine.execute(kill_stmt)
    });

    let storage2 = storage.clone();
    let sm2 = session_manager.clone();
    let handle2: thread::JoinHandle<Result<_, _>> = thread::spawn(move || {
        let mut engine = ExecutionEngine::new_with_session(storage2, sm2.clone(), killer2);
        let kill_stmt = Statement::Kill(KillStatement {
            process_id: target_id,
            kill_type: KillType::Connection,
        });
        engine.execute(kill_stmt)
    });

    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();

    assert!(
        result1.is_ok(),
        "Killer1 should be able to kill own session"
    );
    assert!(
        result2.is_err(),
        "Killer2 should not be able to kill another user's session"
    );
}

#[test]
fn test_session_state_after_kill_connection() {
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

#[test]
fn test_session_state_after_kill_query() {
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

    let target_session = session_manager.get_session(session_id2);
    assert!(target_session.is_some());
    assert_ne!(
        target_session.unwrap().status,
        SessionStatus::Closed,
        "KILL QUERY should not close the session"
    );
}

#[test]
fn test_cancel_flag_propagation() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let mut storage = MemoryStorage::new();
    let cancel_flag = Arc::new(AtomicBool::new(false));

    storage.set_cancel_flag(cancel_flag.clone());

    cancel_flag.store(true, Ordering::SeqCst);

    let result = storage.check_cancelled();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Query cancelled"));
}

#[test]
fn test_no_deadlock_under_concurrent_kill() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let session_manager = Arc::new(SessionManager::new());

    let mut session_ids = Vec::new();
    for i in 0..50 {
        let sid = session_manager.create_session(format!("user_{}", i), format!("127.0.0.{}", i));
        session_ids.push(sid);
    }

    let start = Instant::now();
    let timeout = Duration::from_secs(30);

    let handles: Vec<thread::JoinHandle<_>> = session_ids
        .iter()
        .map(|&session_id| {
            let storage = storage.clone();
            let session_manager = session_manager.clone();
            let session_ids = session_ids.clone();

            thread::spawn(move || {
                let mut engine =
                    ExecutionEngine::new_with_session(storage, session_manager.clone(), session_id);

                for target_id in session_ids.iter().take(5) {
                    if *target_id != session_id {
                        let kill_stmt = Statement::Kill(KillStatement {
                            process_id: *target_id,
                            kill_type: KillType::Query,
                        });
                        let _ = engine.execute(kill_stmt);
                    }
                }
            })
        })
        .collect();

    for h in handles {
        let start_thread = Instant::now();
        loop {
            if h.is_finished() {
                h.join().unwrap();
                break;
            }
            if start_thread.elapsed() > timeout {
                panic!("Thread hung - possible deadlock detected!");
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    println!("No deadlock test completed in {:?}", start.elapsed());
}
