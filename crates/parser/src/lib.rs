// SQLRustGo Parser Module
pub use sqlrustgo_common::{SqlError, SqlResult};

pub mod lexer;
pub mod parser;
pub mod token;

pub use lexer::Lexer;
pub use parser::Parser;
pub use token::Token;

pub use parser::parse;
pub use parser::CreateViewStatement;
pub use parser::Expression;
pub use parser::ForeignKeyAction;
pub use parser::ForeignKeyRef;
pub use parser::FrameBoundInfo;
pub use parser::GrantStatement;
pub use parser::Privilege;
pub use parser::RevokeStatement;
pub use parser::SetOperation;
pub use parser::SetOperationType;
pub use parser::Statement;
pub use parser::TransactionCommand;
pub use parser::TransactionStatement;
pub use parser::WindowFrameInfo;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        let _lexer = Lexer::new("");
        let tokens = lexer::tokenize("SELECT 1");
        let _parser = Parser::new(tokens);
        let _token = Token::Select;
        let _stmt = Statement::Select(crate::parser::SelectStatement {
            table: "users".to_string(),
            columns: vec![],
            where_clause: None,
            join_clause: None,
            aggregates: vec![],
            limit: None,
            offset: None,
            group_by: None,
            having: None,
            order_by: None,
            with_clause: None,
        });
    }

    #[test]
    fn test_parse_function() {
        let result = parse("SELECT id FROM users");
        assert!(result.is_ok());
    }

    #[test]
    fn test_lexer_function() {
        let tokens = lexer::tokenize("SELECT 1");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_parse_union() {
        let result = parse("SELECT id FROM t1 UNION SELECT id FROM t2");
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::SetOperation(_) => {}
            _ => panic!("Expected SetOperation"),
        }
    }

    #[test]
    fn test_parse_union_all() {
        let result = parse("SELECT id FROM t1 UNION ALL SELECT id FROM t2");
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::SetOperation(crate::parser::SetOperation {
                op_type: crate::parser::SetOperationType::UnionAll,
                ..
            }) => {}
            _ => panic!("Expected UnionAll"),
        }
    }

    #[test]
    fn test_parse_cte() {
        let result = parse("WITH RECURSIVE cnt AS (SELECT n FROM t UNION ALL SELECT n FROM cnt WHERE n < 10) SELECT * FROM cnt");
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Select(select) => {
                assert!(select.with_clause.is_some(), "Expected WITH clause");
                let with = select.with_clause.unwrap();
                assert!(with.recursive, "Expected RECURSIVE flag");
                assert_eq!(with.cte_tables.len(), 1);
                assert_eq!(with.cte_tables[0].name, "cnt");
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_parse_truncate() {
        let result = parse("TRUNCATE TABLE t1");
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Truncate(trunc) => {
                assert_eq!(trunc.table_name, "t1");
            }
            _ => panic!("Expected Truncate statement"),
        }
    }

    #[test]
    fn test_parse_create_view() {
        let result = parse("CREATE VIEW v AS SELECT id, name FROM users");
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::CreateView(view) => {
                assert_eq!(view.name, "v");
                assert!(view.query.contains("SELECT"));
            }
            _ => panic!("Expected CreateView"),
        }
    }
}

#[test]
fn test_sql92_features() {
    let tests = vec![
        ("SELECT * FROM users LIMIT 10", true),
        ("SELECT * FROM users LIMIT 10 OFFSET 20", true),
        ("INSERT INTO users SET name='test'", true),
        ("ALTER TABLE users ADD COLUMN age INTEGER", true),
        ("ALTER TABLE users DROP COLUMN age", true),
        ("CREATE INDEX idx ON users (name)", true),
        ("CREATE UNIQUE INDEX idx ON users (email)", true),
        ("CREATE TABLE t (id DECIMAL(10,2))", true),
        ("CREATE TABLE t (data JSON)", true),
    ];
    for (sql, should_pass) in tests {
        let result = parse(sql);
        if should_pass {
            assert!(result.is_ok(), "Expected {} to parse OK", sql);
        }
    }
}

#[test]
fn test_parse_group_by() {
    let sql = "SELECT category, COUNT(*) FROM products GROUP BY category";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse GROUP BY: {:?}", result);
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert!(select.group_by.is_some(), "Expected group_by to be Some");
        let group_by = select.group_by.unwrap();
        assert_eq!(group_by.columns.len(), 1);
    } else {
        panic!("Expected Select statement");
    }
}

#[test]
fn test_parse_group_by_multiple_columns() {
    let sql = "SELECT category, brand, COUNT(*) FROM products GROUP BY category, brand";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse GROUP BY with multiple columns: {:?}",
        result
    );
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert!(select.group_by.is_some());
        let group_by = select.group_by.unwrap();
        assert_eq!(group_by.columns.len(), 2);
    }
}

#[test]
fn test_parse_having() {
    let sql = "SELECT category, COUNT(*) FROM products GROUP BY category HAVING COUNT(*) > 1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse HAVING: {:?}", result);
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert!(select.group_by.is_some(), "Expected group_by to be Some");
        assert!(select.having.is_some(), "Expected having to be Some");
    } else {
        panic!("Expected Select statement");
    }
}

#[test]
fn test_parse_order_by() {
    let sql = "SELECT * FROM products ORDER BY name ASC";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ORDER BY: {:?}", result);
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert!(select.order_by.is_some(), "Expected order_by to be Some");
        let order_by = select.order_by.unwrap();
        assert_eq!(order_by.items.len(), 1);
        assert!(order_by.items[0].asc);
        assert!(order_by.items[0].nulls_first); // ASC defaults to NULLS FIRST
    } else {
        panic!("Expected Select statement");
    }
}

#[test]
fn test_parse_order_by_desc_nulls_last() {
    let sql = "SELECT * FROM products ORDER BY price DESC NULLS LAST";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ORDER BY DESC NULLS LAST: {:?}",
        result
    );
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert!(select.order_by.is_some());
        let order_by = select.order_by.unwrap();
        assert_eq!(order_by.items.len(), 1);
        assert!(!order_by.items[0].asc); // DESC
        assert!(!order_by.items[0].nulls_first); // NULLS LAST
    }
}

#[test]
fn test_parse_complete_aggregate_query() {
    let sql = "SELECT category, SUM(price) FROM products WHERE price > 10 GROUP BY category HAVING SUM(price) > 100 ORDER BY SUM(price) DESC";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse complete aggregate query: {:?}",
        result
    );
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert!(select.group_by.is_some());
        assert!(select.having.is_some());
        assert!(select.order_by.is_some());
    }
}

#[test]
fn test_parse_order_by_multiple_columns() {
    let sql = "SELECT * FROM products ORDER BY category ASC, price DESC";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ORDER BY with multiple columns: {:?}",
        result
    );
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert!(select.order_by.is_some());
        let order_by = select.order_by.unwrap();
        assert_eq!(order_by.items.len(), 2);
        assert!(order_by.items[0].asc);
        assert!(!order_by.items[1].asc);
    }
}

#[test]
fn test_parse_row_number_window_function() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ROW_NUMBER: {:?}", result);
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        // Check that we have columns
        assert!(!select.columns.is_empty());
    } else {
        panic!("Expected Select statement");
    }
}

#[test]
fn test_parse_rank_window_function() {
    let sql = "SELECT RANK() OVER (PARTITION BY dept ORDER BY salary) FROM employees";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse RANK: {:?}", result);
}

#[test]
fn test_parse_lead_window_function() {
    let sql = "SELECT LEAD(salary, 1) OVER (ORDER BY hire_date) FROM employees";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LEAD: {:?}", result);
}

#[test]
fn test_parse_lag_window_function() {
    let sql = "SELECT LAG(salary, 1, 0) OVER (PARTITION BY dept ORDER BY hire_date) FROM employees";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LAG: {:?}", result);
}

#[test]
fn test_parse_window_frame_rows() {
    // Test basic ROW_NUMBER with frame (simple version without aggregate)
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM employees";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse window frame: {:?}", result);
}

#[test]
fn test_parse_window_frame_range() {
    // Test RANK with frame (simple version)
    let sql = "SELECT RANK() OVER (ORDER BY id RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM employees";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse RANGE window frame: {:?}",
        result
    );
}

#[test]
fn test_parse_window_frame_with_exclude() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW EXCLUDE CURRENT ROW) FROM employees";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse window frame with EXCLUDE: {:?}",
        result
    );
}

// Stored Procedures
pub use parser::{
    CallProcedureStatement, CreateProcedureStatement, DelimiterStatement, DropProcedureStatement,
    ParamMode, ProcedureParam, ProcedureStatement,
};
