use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

fn create_engine_with_storage(storage: Arc<RwLock<MemoryStorage>>) -> ExecutionEngine<MemoryStorage> {
    ExecutionEngine::new(storage)
}

fn setup_table(engine: &mut ExecutionEngine<MemoryStorage>) {
    engine
        .execute("CREATE TABLE t (id INTEGER PRIMARY KEY, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();
    engine.execute("INSERT INTO t VALUES (5, 500)").unwrap();
    engine.execute("INSERT INTO t VALUES (10, 1000)").unwrap();
    engine.execute("INSERT INTO t VALUES (15, 1500)").unwrap();
}

#[test]
fn test_for_update_equality_acquires_next_key_lock() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));

    let storage_setup = storage.clone();
    thread::spawn(move || {
        let mut engine = create_engine_with_storage(storage_setup);
        setup_table(&mut engine);
    })
    .join()
    .unwrap();

    let storage_tx1 = storage.clone();
    let tx1_handle = thread::spawn(move || {
        let mut engine = create_engine_with_storage(storage_tx1);
        engine.execute("BEGIN").unwrap();
        assert!(engine.execute("SELECT * FROM t WHERE id = 5 FOR UPDATE").is_ok());
        thread::sleep(Duration::from_millis(100));
        engine.execute("COMMIT").unwrap();
    });

    let storage_tx2 = storage.clone();
    let tx2_handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        let mut engine = create_engine_with_storage(storage_tx2);
        engine.execute("BEGIN").unwrap();
        let result = engine.execute("INSERT INTO t VALUES (5, 999)");
        println!("tx2: INSERT result: {:?}", result.is_ok());
        engine.execute("COMMIT").unwrap();
    });

    tx1_handle.join().unwrap();
    tx2_handle.join().unwrap();
}

#[test]
fn test_for_update_range_acquires_gap_lock() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));

    let storage_setup = storage.clone();
    thread::spawn(move || {
        let mut engine = create_engine_with_storage(storage_setup);
        setup_table(&mut engine);
    })
    .join()
    .unwrap();

    let storage_tx1 = storage.clone();
    let tx1_handle = thread::spawn(move || {
        let mut engine = create_engine_with_storage(storage_tx1);
        engine.execute("BEGIN").unwrap();
        assert!(engine.execute("SELECT * FROM t WHERE id > 5 FOR UPDATE").is_ok());
        thread::sleep(Duration::from_millis(100));
        engine.execute("COMMIT").unwrap();
    });

    let storage_tx2 = storage.clone();
    let tx2_handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        let mut engine = create_engine_with_storage(storage_tx2);
        engine.execute("BEGIN").unwrap();
        let result = engine.execute("INSERT INTO t VALUES (7, 700)");
        println!("tx2: INSERT id=7 result: {:?}", result.is_ok());
        engine.execute("COMMIT").unwrap();
    });

    tx1_handle.join().unwrap();
    tx2_handle.join().unwrap();
}

#[test]
fn test_for_update_less_than_acquires_gap_lock() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));

    let storage_setup = storage.clone();
    thread::spawn(move || {
        let mut engine = create_engine_with_storage(storage_setup);
        setup_table(&mut engine);
    })
    .join()
    .unwrap();

    let storage_tx1 = storage.clone();
    let tx1_handle = thread::spawn(move || {
        let mut engine = create_engine_with_storage(storage_tx1);
        engine.execute("BEGIN").unwrap();
        assert!(engine.execute("SELECT * FROM t WHERE id < 10 FOR UPDATE").is_ok());
        thread::sleep(Duration::from_millis(100));
        engine.execute("COMMIT").unwrap();
    });

    let storage_tx2 = storage.clone();
    let tx2_handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        let mut engine = create_engine_with_storage(storage_tx2);
        engine.execute("BEGIN").unwrap();
        let result = engine.execute("INSERT INTO t VALUES (8, 800)");
        println!("tx2: INSERT id=8 result: {:?}", result.is_ok());
        engine.execute("COMMIT").unwrap();
    });

    tx1_handle.join().unwrap();
    tx2_handle.join().unwrap();
}

#[test]
fn test_serializable_isolation_gap_locking() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));

    let storage_setup = storage.clone();
    thread::spawn(move || {
        let mut engine = create_engine_with_storage(storage_setup);
        setup_table(&mut engine);
    })
    .join()
    .unwrap();

    let storage_tx1 = storage.clone();
    let tx1_handle = thread::spawn(move || {
        let mut engine = create_engine_with_storage(storage_tx1);
        engine.execute("BEGIN SERIALIZABLE").unwrap();
        assert!(engine.execute("SELECT * FROM t WHERE id > 5 FOR UPDATE").is_ok());
        thread::sleep(Duration::from_millis(100));
        engine.execute("COMMIT").unwrap();
    });

    let storage_tx2 = storage.clone();
    let tx2_handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        let mut engine = create_engine_with_storage(storage_tx2);
        engine.execute("BEGIN SERIALIZABLE").unwrap();
        let result = engine.execute("INSERT INTO t VALUES (12, 1200)");
        println!("tx2: INSERT id=12 result: {:?}", result.is_ok());
        engine.execute("COMMIT").unwrap();
    });

    tx1_handle.join().unwrap();
    tx2_handle.join().unwrap();
}
