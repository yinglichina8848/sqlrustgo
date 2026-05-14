use sqlrustgo_parser::parse;
use sqlrustgo_parser::Statement;

#[test]
fn test_parse_drop_event() {
    let sql = "DROP EVENT my_event";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP EVENT: {:?}", result);
    match result {
        Ok(Statement::DropEvent(_)) => {}
        Ok(other) => panic!("Expected DropEvent, got {:?}", other),
        Err(e) => panic!("Parse error: {}", e),
    }
}

#[test]
fn test_parse_drop_event_if_exists() {
    let sql = "DROP EVENT IF EXISTS my_event";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DROP EVENT IF EXISTS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_events() {
    let sql = "SHOW EVENTS";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SHOW EVENTS: {:?}", result);
    match result {
        Ok(Statement::Show(_)) => {}
        Ok(other) => panic!("Expected Show statement, got {:?}", other),
        Err(e) => panic!("Parse error: {}", e),
    }
}

#[test]
fn test_parse_show_events_like() {
    let sql = "SHOW EVENTS LIKE 'my_event%'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW EVENTS LIKE: {:?}",
        result
    );
}

#[test]
fn test_parse_show_events_from_database() {
    let sql = "SHOW EVENTS FROM my_database";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW EVENTS FROM: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_every_interval() {
    let sql =
        "CREATE EVENT my_event ON SCHEDULE EVERY 1 HOUR DO BEGIN INSERT INTO log VALUES (1); END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE EVENT EVERY interval: {:?}",
        result
    );
}
