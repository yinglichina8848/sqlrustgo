// SQLRustGo Parser Module
pub use sqlrustgo_common::{SqlError, SqlResult};

pub mod error;
// expression module removed - types unified with parser.rs
pub mod lexer;
pub mod parser;
pub mod token;

pub use error::{ParseError, ParseResult};

pub use lexer::Lexer;
pub use parser::Parser;
pub use token::Token;

// expression module - for incremental refactoring
// Expression types re-exported from parser.rs to maintain API compatibility
pub use parser::parse;
pub use parser::CreateViewStatement;
pub use parser::Expression;
pub use parser::ForeignKeyAction;
pub use parser::ForeignKeyRef;
pub use parser::GrantStatement;
pub use parser::IndexHint;
pub use parser::IndexHintType;
pub use parser::KillStatement;
pub use parser::KillType;
pub use parser::Privilege;
pub use parser::RevokeStatement;
pub use parser::SetOperation;
pub use parser::SetOperationType;
pub use parser::Statement;
pub use parser::TableConstraint;
pub use parser::TransactionCommand;
pub use parser::TransactionStatement;
pub use parser::{FrameBoundInfo, OrderByItem, WindowFrameInfo};

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
            tables: vec!["users".to_string()],
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
            index_hints: vec![],
            derived_tables: vec![],
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
    let sql = "SELECT RANK() OVER (ORDER BY id RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM employees";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse RANGE window frame: {:?}",
        result
    );
}

#[test]
fn test_parse_create_user() {
    let sql = "CREATE USER 'admin'@'%' IDENTIFIED BY 'secret'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE USER: {:?}", result);
}

#[test]
fn test_parse_drop_user() {
    let sql = "DROP USER 'admin'@'%'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP USER: {:?}", result);
}

#[test]
fn test_parse_copy() {
    let sql = "COPY users FROM '/data/users.csv' (FORMAT PARQUET)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse COPY: {:?}", result);
}

#[test]
fn test_parse_delimiter() {
    let sql = "DELIMITER $$";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DELIMITER: {:?}", result);
}

#[test]
fn test_parse_kill() {
    let sql = "KILL CONNECTION 123";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse KILL: {:?}", result);
}

#[test]
fn test_parse_execute() {
    let sql = "EXECUTE my_stmt";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse EXECUTE: {:?}", result);
}

#[test]
fn test_parse_deallocate() {
    let sql = "DEALLOCATE PREPARE my_stmt";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DEALLOCATE: {:?}", result);
}

#[test]
fn test_parse_show_status() {
    let sql = "SHOW STATUS";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SHOW STATUS: {:?}", result);
}

#[test]
fn test_parse_merge() {
    let sql = "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET name = source.name";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse MERGE: {:?}", result);
}

#[test]
fn test_parse_alter_table_add_column() {
    let sql = "ALTER TABLE users ADD COLUMN age INTEGER";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE ADD COLUMN: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_drop_column() {
    let sql = "ALTER TABLE users DROP COLUMN age";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE DROP COLUMN: {:?}",
        result
    );
}

#[test]
fn test_parse_create_unique_index() {
    let sql = "CREATE UNIQUE INDEX idx ON users (email)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE UNIQUE INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_begin_transaction() {
    let sql = "BEGIN";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse BEGIN: {:?}", result);
}

#[test]
fn test_parse_commit() {
    let sql = "COMMIT";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse COMMIT: {:?}", result);
}

#[test]
fn test_parse_rollback() {
    let sql = "ROLLBACK";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ROLLBACK: {:?}", result);
}

#[test]
fn test_parse_insert_set() {
    let sql = "INSERT INTO users SET name='test', email='test@example.com'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse INSERT SET: {:?}", result);
}

#[test]
fn test_parse_insert_values() {
    let sql = "INSERT INTO users (id, name) VALUES (1, 'test')";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse INSERT VALUES: {:?}",
        result
    );
}

#[test]
fn test_parse_update() {
    let sql = "UPDATE users SET name='test' WHERE id = 1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse UPDATE: {:?}", result);
}

#[test]
fn test_parse_delete() {
    let sql = "DELETE FROM users WHERE id = 1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DELETE: {:?}", result);
}

#[test]
fn test_parse_truncate() {
    let sql = "TRUNCATE TABLE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse TRUNCATE: {:?}", result);
}

#[test]
fn test_parse_drop_table() {
    let sql = "DROP TABLE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP TABLE: {:?}", result);
}

#[test]
fn test_parse_revoke() {
    let sql = "REVOKE INSERT ON users FROM admin";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REVOKE: {:?}", result);
}

#[test]
fn test_parse_show_processlist() {
    let sql = "SHOW PROCESSLIST";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW PROCESSLIST: {:?}",
        result
    );
}

#[test]
fn test_parse_explain() {
    let sql = "EXPLAIN SELECT * FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse EXPLAIN: {:?}", result);
}

#[test]
fn test_parse_call_procedure() {
    let sql = "CALL my_proc(1, 'test')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CALL: {:?}", result);
}

#[test]
fn test_parse_prepare() {
    let sql = "PREPARE stmt FROM 'SELECT * FROM users'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse PREPARE: {:?}", result);
}

#[test]
fn test_parse_limit() {
    let sql = "SELECT * FROM users LIMIT 10";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LIMIT: {:?}", result);
}

#[test]
fn test_parse_limit_offset() {
    let sql = "SELECT * FROM users LIMIT 10 OFFSET 5";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LIMIT OFFSET: {:?}", result);
}

#[test]
fn test_parse_inner_join() {
    let sql = "SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse INNER JOIN: {:?}", result);
}

#[test]
fn test_parse_left_join() {
    let sql = "SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LEFT JOIN: {:?}", result);
}

#[test]
fn test_parse_between() {
    let sql = "SELECT * FROM users WHERE age BETWEEN 18 AND 65";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse BETWEEN: {:?}", result);
}

#[test]
fn test_parse_like() {
    let sql = "SELECT * FROM users WHERE name LIKE 'John%'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LIKE: {:?}", result);
}

#[test]
fn test_parse_is_null() {
    let sql = "SELECT * FROM users WHERE email IS NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse IS NULL: {:?}", result);
}

#[test]
fn test_parse_is_not_null() {
    let sql = "SELECT * FROM users WHERE email IS NOT NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse IS NOT NULL: {:?}", result);
}

#[test]
fn test_parse_union() {
    let sql = "SELECT id FROM users UNION SELECT id FROM admins";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse UNION: {:?}", result);
}

#[test]
fn test_parse_union_all() {
    let sql = "SELECT id FROM users UNION ALL SELECT id FROM admins";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse UNION ALL: {:?}", result);
}

#[test]
fn test_parse_alias() {
    let sql = "SELECT u.id, u.name AS user_name FROM users AS u";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse alias: {:?}", result);
}

#[test]
fn test_parse_cte_simple() {
    let sql = "WITH cte AS (SELECT id FROM users) SELECT * FROM cte";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CTE: {:?}", result);
}

// Stored Procedures
pub use parser::{
    CallProcedureStatement, CreateProcedureStatement, DelimiterStatement, DropProcedureStatement,
    ParamMode, ProcedureParam, ProcedureStatement,
};
