//! T-27: ReadWriteSplitter unit tests

use sqlrustgo_distributed::read_write_splitter::{QueryClass, ReadWriteSplitter};

#[tokio::test]
async fn test_route_simple_select_is_read() -> Result<(), Box<dyn std::error::Error>> {
    let splitter = ReadWriteSplitter::new(1);
    let result = splitter.route_simple("SELECT * FROM users WHERE id = 1")?;
    assert_eq!(result.0, QueryClass::Read);
    assert!(
        !result.1,
        "SELECT should route to replica (is_primary=false)"
    );
    Ok(())
}

#[tokio::test]
async fn test_route_simple_insert_is_write() -> Result<(), Box<dyn std::error::Error>> {
    let splitter = ReadWriteSplitter::new(1);
    let result = splitter.route_simple("INSERT INTO users (name) VALUES ('Alice')")?;
    assert_eq!(result.0, QueryClass::Write);
    assert!(result.1, "INSERT should route to primary (is_primary=true)");
    Ok(())
}

#[tokio::test]
async fn test_route_simple_update_is_write() -> Result<(), Box<dyn std::error::Error>> {
    let splitter = ReadWriteSplitter::new(1);
    let result = splitter.route_simple("UPDATE users SET name = 'Bob' WHERE id = 1")?;
    assert_eq!(result.0, QueryClass::Write);
    assert!(result.1);
    Ok(())
}

#[tokio::test]
async fn test_route_simple_delete_is_write() -> Result<(), Box<dyn std::error::Error>> {
    let splitter = ReadWriteSplitter::new(1);
    let result = splitter.route_simple("DELETE FROM users WHERE id = 1")?;
    assert_eq!(result.0, QueryClass::Write);
    assert!(result.1);
    Ok(())
}

#[tokio::test]
async fn test_route_simple_show_is_read() -> Result<(), Box<dyn std::error::Error>> {
    let splitter = ReadWriteSplitter::new(1);
    let result = splitter.route_simple("SHOW TABLES")?;
    assert_eq!(result.0, QueryClass::Read);
    assert!(!result.1);
    Ok(())
}

#[tokio::test]
async fn test_route_simple_create_is_write() -> Result<(), Box<dyn std::error::Error>> {
    let splitter = ReadWriteSplitter::new(1);
    let result = splitter.route_simple("CREATE TABLE test (id INT PRIMARY KEY)")?;
    assert_eq!(result.0, QueryClass::Write);
    assert!(result.1);
    Ok(())
}

#[tokio::test]
async fn test_query_class_helpers() -> Result<(), Box<dyn std::error::Error>> {
    let read = QueryClass::Read;
    let write = QueryClass::Write;
    assert!(read.is_read());
    assert!(!read.is_write());
    assert!(!write.is_read());
    assert!(write.is_write());
    Ok(())
}
