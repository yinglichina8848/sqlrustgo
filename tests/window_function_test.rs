// Window Functions Tests (BP2-6)
//! Tests for LEAD, LAG, FIRST_VALUE, LAST_VALUE, NTILE, NTH_VALUE
//!
//! BP2 Gate: cargo test --test window_function_test

use sqlrustgo_parser::{parse, Expression, Statement};

/// Test parsing of LEAD window function
#[test]
fn test_parse_lead() {
    let sql = "SELECT LEAD(amount) OVER (ORDER BY date) FROM sales";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for LEAD: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            // Find the window call expression
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(wc)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!window_calls.is_empty(), "Expected window call in SELECT");
            assert_eq!(window_calls[0].func_name.to_uppercase(), "LEAD");
        }
        _ => panic!("Expected SELECT statement"),
    }
}

/// Test parsing of LAG window function
#[test]
fn test_parse_lag() {
    let sql = "SELECT LAG(value) OVER (PARTITION BY dept ORDER BY date) FROM metrics";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for LAG: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(wc)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!window_calls.is_empty());
            assert_eq!(window_calls[0].func_name.to_uppercase(), "LAG");
            // Check partition by
            assert!(!window_calls[0].window_spec.partition_by.is_empty());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

/// Test parsing of FIRST_VALUE window function
#[test]
fn test_parse_first_value() {
    let sql = "SELECT FIRST_VALUE(amount) OVER (ORDER BY date) FROM sales";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for FIRST_VALUE: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(wc)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!window_calls.is_empty());
            assert_eq!(window_calls[0].func_name.to_uppercase(), "FIRST_VALUE");
        }
        _ => panic!("Expected SELECT statement"),
    }
}

/// Test parsing of LAST_VALUE window function
#[test]
fn test_parse_last_value() {
    let sql = "SELECT LAST_VALUE(amount) OVER (ORDER BY date) FROM sales";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for LAST_VALUE: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(wc)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!window_calls.is_empty());
            assert_eq!(window_calls[0].func_name.to_uppercase(), "LAST_VALUE");
        }
        _ => panic!("Expected SELECT statement"),
    }
}

/// Test parsing of NTILE window function
#[test]
fn test_parse_ntile() {
    let sql = "SELECT NTILE(4) OVER (ORDER BY score) FROM rankings";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for NTILE: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(wc)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!window_calls.is_empty());
            assert_eq!(window_calls[0].func_name.to_uppercase(), "NTILE");
            // NTILE takes a numeric argument
            assert!(!window_calls[0].args.is_empty());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

/// Test parsing of NTH_VALUE window function
#[test]
fn test_parse_nth_value() {
    let sql = "SELECT NTH_VALUE(amount, 2) OVER (ORDER BY date) FROM sales";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for NTH_VALUE: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(wc)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!window_calls.is_empty());
            assert_eq!(window_calls[0].func_name.to_uppercase(), "NTH_VALUE");
            // NTH_VALUE takes 2 arguments: expression and n
            assert_eq!(window_calls[0].args.len(), 2);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

/// Test LEAD with offset argument
#[test]
fn test_parse_lead_with_offset() {
    let sql = "SELECT LEAD(amount, 1) OVER (ORDER BY date) FROM sales";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Parse failed for LEAD with offset: {:?}",
        result
    );

    match result.unwrap() {
        Statement::Select(select) => {
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(wc)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!window_calls.is_empty());
            // LEAD with offset has 2 args
            assert_eq!(window_calls[0].args.len(), 2);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

/// Test LAG with default value
#[test]
fn test_parse_lag_with_default() {
    let sql = "SELECT LAG(amount, 1, 0) OVER (ORDER BY date) FROM sales";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Parse failed for LAG with default: {:?}",
        result
    );

    match result.unwrap() {
        Statement::Select(select) => {
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(wc)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!window_calls.is_empty());
            // LAG with offset and default has 3 args
            assert_eq!(window_calls[0].args.len(), 3);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

/// Test multiple window functions in one query
#[test]
fn test_parse_multiple_window_functions() {
    let sql = "SELECT \
        ROW_NUMBER() OVER (ORDER BY date) as rn, \
        LEAD(amount) OVER (ORDER BY date) as next_amount, \
        LAG(amount) OVER (ORDER BY date) as prev_amount, \
        FIRST_VALUE(amount) OVER (ORDER BY date) as first_amount, \
        LAST_VALUE(amount) OVER (ORDER BY date) as last_amount \
        FROM sales";

    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Parse failed for multiple window functions: {:?}",
        result
    );

    match result.unwrap() {
        Statement::Select(select) => {
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(wc.func_name.to_uppercase())
                    } else {
                        None
                    }
                })
                .collect();

            assert_eq!(window_calls.len(), 5);
            assert_eq!(window_calls[0], "ROW_NUMBER");
            assert_eq!(window_calls[1], "LEAD");
            assert_eq!(window_calls[2], "LAG");
            assert_eq!(window_calls[3], "FIRST_VALUE");
            assert_eq!(window_calls[4], "LAST_VALUE");
        }
        _ => panic!("Expected SELECT statement"),
    }
}

/// Test window function with ORDER BY
#[test]
fn test_parse_window_with_order_by() {
    let sql = "SELECT AVG(amount) OVER (ORDER BY date, time) FROM sales";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Parse failed for window with ORDER BY: {:?}",
        result
    );

    match result.unwrap() {
        Statement::Select(select) => {
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(&wc.window_spec.order_by)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!window_calls.is_empty());
            // Should have 2 order by expressions (date, time)
            assert_eq!(window_calls[0].len(), 2);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

/// Test window function with PARTITION BY
#[test]
fn test_parse_window_with_partition_by() {
    let sql = "SELECT SUM(amount) OVER (PARTITION BY department, region ORDER BY date) FROM sales";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Parse failed for window with PARTITION BY: {:?}",
        result
    );

    match result.unwrap() {
        Statement::Select(select) => {
            let window_calls: Vec<_> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if let Some(Expression::WindowCall(wc)) = &col.expression {
                        Some(&wc.window_spec.partition_by)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!window_calls.is_empty());
            // Should have 2 partition by expressions
            assert_eq!(window_calls[0].len(), 2);
        }
        _ => panic!("Expected SELECT statement"),
    }
}
