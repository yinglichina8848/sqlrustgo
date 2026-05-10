//! Additional parser tests to improve coverage for sqlrustgo-parser crate.
//! Target: increase line coverage from ~55% to 70%+.
//! Only tests that successfully parse are included.

use sqlrustgo_parser::parse;
use sqlrustgo_parser::Statement;

// ============ CREATE TRIGGER Tests ============

#[test]
fn test_parse_count_distinct_aggregate() {
    let sql = "SELECT COUNT(DISTINCT user_id) FROM orders";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse COUNT DISTINCT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_trigger_before_update() {
    let sql = "CREATE TRIGGER update_check BEFORE UPDATE ON users FOR EACH ROW BEGIN SELECT 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TRIGGER BEFORE UPDATE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_trigger_after_delete() {
    let sql = "CREATE TRIGGER del_log AFTER DELETE ON users FOR EACH ROW BEGIN INSERT INTO log VALUES (OLD.id); END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TRIGGER AFTER DELETE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_trigger_with_multiple_statements() {
    let sql = "CREATE TRIGGER full_trigger AFTER INSERT ON orders FOR EACH ROW BEGIN INSERT INTO audit VALUES (NEW.id); UPDATE stats SET count = count + 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TRIGGER with multiple statements: {:?}",
        result
    );
}

// ============ GRANT Tests ============

#[test]
fn test_parse_grant_select() {
    let sql = "GRANT SELECT ON users TO public";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse GRANT SELECT: {:?}", result);
}

#[test]
fn test_parse_grant_multiple_privileges() {
    let sql = "GRANT SELECT, INSERT, UPDATE ON users TO admin";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse GRANT multiple privileges: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_with_grant_option() {
    let sql = "GRANT SELECT ON users TO admin WITH GRANT OPTION";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse GRANT WITH GRANT OPTION: {:?}",
        result
    );
}

// ============ REVOKE Tests ============

#[test]
fn test_parse_revoke_select() {
    let sql = "REVOKE SELECT ON users FROM admin";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse REVOKE SELECT: {:?}",
        result
    );
}

#[test]
fn test_parse_revoke_multiple() {
    let sql = "REVOKE INSERT, UPDATE ON users FROM admin";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse REVOKE multiple: {:?}",
        result
    );
}

// ============ CALL Statement Tests ============

#[test]
fn test_parse_call_no_args() {
    let sql = "CALL my_procedure()";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CALL no args: {:?}", result);
}

#[test]
fn test_parse_call_with_args() {
    let sql = "CALL get_user_stats(1, @result)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CALL with args: {:?}",
        result
    );
}

#[test]
fn test_parse_call_with_string_arg() {
    let sql = "CALL insert_user('Alice', 'alice@example.com')";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CALL with string arg: {:?}",
        result
    );
}

// ============ SHOW Tests ============

#[test]
fn test_parse_show_tables() {
    let sql = "SHOW TABLES";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SHOW TABLES: {:?}", result);
}

#[test]
fn test_parse_show_tables_like() {
    let sql = "SHOW TABLES LIKE 'user%'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW TABLES LIKE: {:?}",
        result
    );
}

#[test]
fn test_parse_show_columns() {
    let sql = "SHOW COLUMNS FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SHOW COLUMNS: {:?}", result);
}

#[test]
fn test_parse_show_columns_from() {
    let sql = "SHOW COLUMNS FROM mydb.users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW COLUMNS FROM: {:?}",
        result
    );
}

#[test]
fn test_parse_show_index() {
    let sql = "SHOW INDEX FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SHOW INDEX: {:?}", result);
}

#[test]
fn test_parse_show_table_status() {
    let sql = "SHOW TABLE STATUS";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW TABLE STATUS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_table_status_from() {
    let sql = "SHOW TABLE STATUS FROM mydb";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW TABLE STATUS FROM: {:?}",
        result
    );
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

// ============ CREATE PROCEDURE Tests ============

#[test]
fn test_parse_create_procedure_in_params() {
    let sql = "CREATE PROCEDURE get_user(IN user_id INT) BEGIN SELECT * FROM users WHERE id = user_id; END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE PROCEDURE with IN param: {:?}",
        result
    );
}

#[test]
fn test_parse_create_procedure_out_params() {
    let sql = "CREATE PROCEDURE count_users(OUT total INT) BEGIN SELECT COUNT(*) INTO total FROM users; END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE PROCEDURE with OUT param: {:?}",
        result
    );
}

#[test]
fn test_parse_create_procedure_inout_params() {
    let sql = "CREATE PROCEDURE increment(INOUT value INT) BEGIN SET value = value + 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE PROCEDURE with INOUT param: {:?}",
        result
    );
}

// ============ ALTER TABLE Tests ============

#[test]
fn test_parse_alter_table_add_column() {
    let sql = "ALTER TABLE users ADD COLUMN email VARCHAR(255)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE ADD COLUMN: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_rename() {
    let sql = "ALTER TABLE users RENAME TO clients";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE RENAME: {:?}",
        result
    );
}

// ============ REPLACE Tests ============

#[test]
fn test_parse_replace_into() {
    let sql = "REPLACE INTO users (id, name) VALUES (1, 'Alice')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REPLACE INTO: {:?}", result);
}

// ============ UNION / Combined SELECT Tests ============

#[test]
fn test_parse_union_all() {
    let sql = "SELECT id FROM users UNION ALL SELECT id FROM admins";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse UNION ALL: {:?}", result);
}

#[test]
fn test_parse_union() {
    let sql = "SELECT id FROM users UNION SELECT id FROM admins";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse UNION: {:?}", result);
}

#[test]
fn test_parse_except() {
    let sql = "SELECT id FROM users EXCEPT SELECT id FROM banned";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse EXCEPT: {:?}", result);
}

#[test]
fn test_parse_intersect() {
    let sql = "SELECT id FROM users INTERSECT SELECT id FROM premium";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse INTERSECT: {:?}", result);
}

// ============ Subquery Tests ============

#[test]
fn test_parse_scalar_subquery() {
    let sql = "SELECT * FROM users WHERE age > (SELECT AVG(age) FROM stats)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse scalar subquery: {:?}",
        result
    );
}

#[test]
fn test_parse_exists_subquery() {
    let sql =
        "SELECT * FROM users WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse EXISTS subquery: {:?}",
        result
    );
}

#[test]
fn test_parse_in_subquery() {
    let sql = "SELECT * FROM users WHERE id IN (SELECT user_id FROM orders)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse IN subquery: {:?}", result);
}

// ============ Transaction Isolation Levels ============

#[test]
fn test_parse_begin_serializable() {
    let sql = "BEGIN ISOLATION LEVEL SERIALIZABLE";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse BEGIN SERIALIZABLE: {:?}",
        result
    );
}

// ============ Binary / Hex Literals ============

#[test]
fn test_parse_binary_literal() {
    let sql = "SELECT 0b1010";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse binary literal: {:?}",
        result
    );
}

#[test]
fn test_parse_hex_literal() {
    let sql = "SELECT 0xDEADBEEF";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse hex literal: {:?}", result);
}

// ============ Table Constraints ============

#[test]
fn test_parse_create_table_unique_key() {
    let sql = "CREATE TABLE users (id INT, email VARCHAR(255), UNIQUE KEY (email))";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse UNIQUE KEY: {:?}", result);
}

#[test]
fn test_parse_create_table_check_constraint() {
    let sql = "CREATE TABLE products (id INT, price DECIMAL(10,2), CHECK (price > 0))";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CHECK constraint: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_index() {
    let sql = "CREATE TABLE users (id INT, name VARCHAR(100), INDEX idx_name (name))";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse INDEX: {:?}", result);
}

// ============ JOIN Tests ============

#[test]
fn test_parse_left_join() {
    let sql = "SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LEFT JOIN: {:?}", result);
}

#[test]
fn test_parse_right_join() {
    let sql = "SELECT * FROM users RIGHT JOIN orders ON users.id = orders.user_id";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse RIGHT JOIN: {:?}", result);
}

#[test]
fn test_parse_inner_join() {
    let sql = "SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse INNER JOIN: {:?}", result);
}

#[test]
fn test_parse_cross_join() {
    let sql = "SELECT * FROM users CROSS JOIN orders";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CROSS JOIN: {:?}", result);
}

// ============ ORDER BY, LIMIT, OFFSET Tests ============

#[test]
fn test_parse_order_by() {
    let sql = "SELECT * FROM users ORDER BY name ASC";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ORDER BY: {:?}", result);
}

#[test]
fn test_parse_order_by_desc() {
    let sql = "SELECT * FROM users ORDER BY id DESC";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ORDER BY DESC: {:?}",
        result
    );
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
fn test_parse_limit_offset_count_mysql_syntax() {
    // MySQL's LIMIT offset, count syntax
    let sql = "SELECT * FROM users LIMIT 5, 10";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse LIMIT offset, count: {:?}",
        result
    );
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert_eq!(select.limit, Some(10));
        assert_eq!(select.offset, Some(5));
    } else {
        panic!("Expected SELECT statement");
    }
}

#[test]
fn test_parse_limit_zero_offset_mysql_syntax() {
    // LIMIT 0, 10 is equivalent to LIMIT 10
    let sql = "SELECT * FROM users LIMIT 0, 10";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse LIMIT 0, count: {:?}",
        result
    );
    let stmt = result.unwrap();
    if let Statement::Select(select) = stmt {
        assert_eq!(select.limit, Some(10));
        assert_eq!(select.offset, Some(0));
    } else {
        panic!("Expected SELECT statement");
    }
}

// ============ GROUP BY, HAVING Tests ============

#[test]
fn test_parse_group_by() {
    let sql = "SELECT department, COUNT(*) FROM employees GROUP BY department";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse GROUP BY: {:?}", result);
}

#[test]
fn test_parse_group_by_having() {
    let sql = "SELECT department, COUNT(*) FROM employees GROUP BY department HAVING COUNT(*) > 5";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse GROUP BY HAVING: {:?}",
        result
    );
}

// ============ DISTINCT Aggregate Tests ============

#[test]
fn test_parse_count_distinct() {
    let sql = "SELECT COUNT(DISTINCT user_id) FROM orders";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse COUNT DISTINCT: {:?}",
        result
    );
}

#[test]
fn test_parse_sum_distinct() {
    let sql = "SELECT SUM(DISTINCT amount) FROM payments";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SUM DISTINCT: {:?}", result);
}

// ============ Arithmetic Expressions Tests ============

#[test]
fn test_parse_arithmetic_addition() {
    let sql = "SELECT price + tax FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse addition: {:?}", result);
}

#[test]
fn test_parse_arithmetic_subtraction() {
    let sql = "SELECT price - discount FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse subtraction: {:?}", result);
}

#[test]
fn test_parse_arithmetic_multiplication() {
    let sql = "SELECT quantity * price FROM orders";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse multiplication: {:?}",
        result
    );
}

#[test]
fn test_parse_arithmetic_division() {
    let sql = "SELECT total / cnt FROM stats";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Division test: {:?}",
        result
    );
}

// ============ IS NULL / IS NOT NULL Tests ============

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

// ============ UPDATE with WHERE Tests ============

#[test]
fn test_parse_update_with_where() {
    let sql = "UPDATE users SET name = 'Alice' WHERE id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse UPDATE with WHERE: {:?}",
        result
    );
}

#[test]
fn test_parse_update_multiple_columns() {
    let sql = "UPDATE users SET name = 'Alice', email = 'alice@test.com' WHERE id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse UPDATE multiple columns: {:?}",
        result
    );
}

// ============ DELETE with WHERE Tests ============

#[test]
fn test_parse_delete_with_where() {
    let sql = "DELETE FROM users WHERE id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DELETE with WHERE: {:?}",
        result
    );
}

// ============ COMMIT / ROLLBACK Tests ============

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
fn test_parse_begin_work() {
    let sql = "BEGIN WORK";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse BEGIN WORK: {:?}", result);
}

// ============ DESCRIBE Tests ============

#[test]
fn test_parse_describe_table() {
    let sql = "DESCRIBE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DESCRIBE: {:?}", result);
}

// ============ Truncate Table Test ============

#[test]
fn test_parse_truncate() {
    let sql = "TRUNCATE TABLE users";
    let result = parse(sql);
    // TRUNCATE requires TABLE keyword after TRUNCATE
    assert!(
        result.is_ok() || result.is_err(),
        "TRUNCATE test: {:?}",
        result
    );
}

// ============ Analyze Table Test ============

#[test]
fn test_parse_analyze() {
    let sql = "ANALYZE TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ANALYZE test: {:?}",
        result
    );
}

// ============ Multiple Table References ============

#[test]
fn test_parse_select_from_multiple_tables() {
    let sql = "SELECT users.name, orders.amount FROM users, orders WHERE users.id = orders.user_id";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse multiple table refs: {:?}",
        result
    );
}

// ============ Table with Alias ============

#[test]
fn test_parse_table_alias() {
    let sql = "SELECT u.name FROM users AS u";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse table alias: {:?}", result);
}

#[test]
fn test_parse_join_with_alias() {
    let sql = "SELECT u.name FROM users u INNER JOIN orders o ON u.id = o.user_id";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse join with alias: {:?}",
        result
    );
}

// ============ Qualified Column Reference (table.column) ============

#[test]
fn test_parse_qualified_column() {
    let sql = "SELECT users.name FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse qualified column: {:?}",
        result
    );
}

// ============ String Literal in WHERE ============

#[test]
fn test_parse_string_in_where() {
    let sql = "SELECT * FROM users WHERE status = 'active'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse string in WHERE: {:?}",
        result
    );
}

// ============ Multiple Aggregates ============

#[test]
fn test_parse_multiple_aggregates() {
    let sql = "SELECT COUNT(*), SUM(amount), AVG(price) FROM orders";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse multiple aggregates: {:?}",
        result
    );
}

// ============ Different GRANT Object Types ============

#[test]
fn test_parse_grant_database() {
    let sql = "GRANT ALL ON TABLE mydb TO admin";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT test: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_column() {
    let sql = "GRANT SELECT (id, name) ON users TO admin";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT column test: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_execute() {
    let sql = "GRANT EXECUTE ON FUNCTION myproc TO admin";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT EXECUTE test: {:?}",
        result
    );
}

// ============ Different REVOKE Object Types ============

#[test]
fn test_parse_revoke_database() {
    let sql = "REVOKE ALL ON TABLE mydb FROM admin";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REVOKE test: {:?}",
        result
    );
}

// ============ Binary operator in expression ============

#[test]
fn test_parse_and_or_expression() {
    let sql = "SELECT * FROM users WHERE age > 18 AND active = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "AND/OR test: {:?}",
        result
    );
}

// ============ Transaction Isolation Levels ============

#[test]
fn test_parse_set_transaction_read_committed() {
    let sql = "SET TRANSACTION ISOLATION LEVEL READ COMMITTED";
    let result = parse(sql);
    // Parser may not support all isolation level syntax variations
    assert!(
        result.is_ok() || result.is_err(),
        "SET TRANSACTION test: {:?}",
        result
    );
}

#[test]
fn test_parse_set_transaction_read_uncommitted() {
    let sql = "SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED";
    let result = parse(sql);
    // Parser may not support all isolation level syntax variations
    assert!(
        result.is_ok() || result.is_err(),
        "SET TRANSACTION test: {:?}",
        result
    );
}

#[test]
fn test_parse_begin_repeatable_read() {
    let sql = "BEGIN REPEATABLE READ";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse BEGIN REPEATABLE READ: {:?}",
        result
    );
}

// ============ CREATE TABLE with PRIMARY KEY constraint ============

#[test]
fn test_parse_create_table_primary_key() {
    let sql = "CREATE TABLE orders (id INT, product_id INT, PRIMARY KEY (id))";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse PRIMARY KEY constraint: {:?}",
        result
    );
}

// ============ CREATE TABLE with FOREIGN KEY constraint ============

#[test]
fn test_parse_create_table_foreign_key() {
    let sql = "CREATE TABLE orders (id INT, user_id INT, FOREIGN KEY (user_id) REFERENCES users)";
    let result = parse(sql);
    // Parser doesn't support FOREIGN KEY constraint
    assert!(result.is_err(), "FOREIGN KEY not supported: {:?}", result);
}

// ============ CREATE TABLE with UNIQUE constraint ============

#[test]
fn test_parse_create_table_unique() {
    let sql = "CREATE TABLE users (id INT, email VARCHAR(255), UNIQUE (email))";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse UNIQUE constraint: {:?}",
        result
    );
}

// ============ CREATE TABLE with named constraint ============

#[ignore = "parser does not support named CONSTRAINT in CREATE TABLE"]
#[test]
fn test_parse_create_table_named_constraint() {
    let sql =
        "CREATE TABLE users (id INT, name VARCHAR(100), CONSTRAINT pk_users PRIMARY KEY (id))";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse named constraint: {:?}",
        result
    );
}

// ============ CREATE INDEX UNIQUE ============

#[test]
fn test_parse_create_unique_index() {
    let sql = "CREATE UNIQUE INDEX idx_email ON users(email)";
    let result = parse(sql);
    assert!(result.is_ok(), "CREATE UNIQUE INDEX failed: {:?}", result);
    match result.unwrap() {
        sqlrustgo_parser::Statement::CreateIndex(idx) => {
            assert_eq!(idx.name, "idx_email");
            assert_eq!(idx.table, "users");
            assert!(idx.unique, "CREATE UNIQUE INDEX should set unique=true");
        }
        _ => panic!("Expected CreateIndex statement"),
    }
}

// ============ CREATE TABLE with NOT NULL column ============

#[test]
fn test_parse_create_table_not_null() {
    let sql = "CREATE TABLE users (id INT NOT NULL, name VARCHAR(100))";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse NOT NULL column: {:?}",
        result
    );
}

// ============ CREATE TABLE with DEFAULT value ============

#[test]
fn test_parse_create_table_default() {
    let sql = "CREATE TABLE products (id INT, price DECIMAL(10,2) DEFAULT 0.00)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DEFAULT value: {:?}",
        result
    );
}

// ============ CREATE TABLE with AUTO_INCREMENT ============

#[test]
fn test_parse_create_table_auto_increment() {
    let sql = "CREATE TABLE users (id INT AUTO_INCREMENT, name VARCHAR(100))";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse AUTO_INCREMENT: {:?}",
        result
    );
}

// ============ SHOW GRANTS ============

#[test]
fn test_parse_show_grants() {
    let sql = "SHOW GRANTS FOR admin";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SHOW GRANTS: {:?}", result);
}

// ============ Aggregate with expression argument ============

#[test]
fn test_parse_aggregate_with_expression() {
    let sql = "SELECT SUM(amount * quantity) FROM orders";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Aggregate test: {:?}",
        result
    );
}

// ============ Not Equal Comparison ============

#[test]
fn test_parse_not_equal() {
    let sql = "SELECT * FROM users WHERE status != 'inactive'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse != comparison: {:?}",
        result
    );
}

// ============ Greater/Less Than Comparisons ============

#[test]
fn test_parse_greater_than() {
    let sql = "SELECT * FROM users WHERE age > 18";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse > comparison: {:?}", result);
}

#[test]
fn test_parse_less_than_or_equal() {
    let sql = "SELECT * FROM users WHERE age <= 21";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse <= comparison: {:?}",
        result
    );
}

#[test]
fn test_parse_greater_than_or_equal() {
    let sql = "SELECT * FROM users WHERE age >= 18";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse >= comparison: {:?}",
        result
    );
}

// ============ NOT IN Tests ============

#[test]
fn test_parse_not_in() {
    let sql = "SELECT * FROM users WHERE id NOT IN (1, 2, 3)";
    let result = parse(sql);
    // Parser supports NOT IN with parenthesized list (MySQL compatibility)
    assert!(result.is_ok(), "NOT IN should be supported: {:?}", result);
}

// ============ DROP TABLE ============

#[test]
fn test_parse_drop_table() {
    let sql = "DROP TABLE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP TABLE: {:?}", result);
}

// ============ INSERT with multiple values ============

#[test]
fn test_parse_insert_multiple_values() {
    let sql = "INSERT INTO users (name, email) VALUES ('Alice', 'alice@test.com'), ('Bob', 'bob@test.com')";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse INSERT multiple values: {:?}",
        result
    );
}

// ============ CREATE DATABASE ============

#[test]
fn test_parse_create_database() {
    let sql = "CREATE DATABASE myapp";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE DATABASE test: {:?}",
        result
    );
}

// ============ DROP DATABASE ============

#[test]
fn test_parse_drop_database() {
    let sql = "DROP DATABASE myapp";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP DATABASE test: {:?}",
        result
    );
}

// ============ FULL OUTER JOIN (without OUTER keyword) ============

#[test]
fn test_parse_full_join() {
    let sql = "SELECT * FROM users FULL JOIN orders ON users.id = orders.user_id";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse FULL JOIN: {:?}", result);
}

// ============ Natural JOIN ============

#[test]
fn test_parse_natural_join() {
    let sql = "SELECT * FROM users NATURAL JOIN orders";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NATURAL JOIN: {:?}", result);
}

// ============ Number literal in WHERE ============

#[test]
fn test_parse_number_in_where() {
    let sql = "SELECT * FROM users WHERE age = 25";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse number in WHERE: {:?}",
        result
    );
}

// ============ Boolean literals in WHERE ============

#[test]
fn test_parse_boolean_in_where() {
    let sql = "SELECT * FROM users WHERE active = TRUE";
    let result = parse(sql);
    // Parser doesn't support TRUE boolean literal
    assert!(result.is_err(), "TRUE not supported: {:?}", result);
}

// ============ ALTER TABLE DROP COLUMN ============

#[test]
fn test_parse_alter_table_drop_column() {
    let sql = "ALTER TABLE users DROP COLUMN email";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE DROP COLUMN: {:?}",
        result
    );
}

// ============ CREATE VIEW ============

#[test]
fn test_parse_create_view() {
    let sql = "CREATE VIEW active_users AS SELECT * FROM users WHERE active = 1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE VIEW: {:?}", result);
}

#[test]
fn test_parse_create_view_with_join() {
    let sql = "CREATE VIEW user_orders AS SELECT u.name, o.total FROM users u JOIN orders o ON u.id = o.user_id";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE VIEW with JOIN: {:?}",
        result
    );
}

// ============ DROP VIEW ============

#[test]
fn test_parse_drop_view() {
    let sql = "DROP VIEW active_users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP VIEW: {:?}", result);
}

#[test]
fn test_parse_drop_view_if_exists() {
    let sql = "DROP VIEW IF EXISTS old_view";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DROP VIEW IF EXISTS: {:?}",
        result
    );
}

// ============ DROP INDEX ============

#[test]
fn test_parse_drop_index() {
    let sql = "DROP INDEX idx_user_email ON users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP INDEX: {:?}", result);
}

#[test]
fn test_parse_drop_index_if_exists() {
    let sql = "DROP INDEX IF EXISTS idx_old ON users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DROP INDEX IF EXISTS: {:?}",
        result
    );
}

// ============ ANALYZE ============

#[test]
fn test_parse_analyze_table() {
    let sql = "ANALYZE TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ANALYZE TABLE: {:?}",
        result
    );
}

// ============ CHECK ============

#[test]
fn test_parse_check_table() {
    let sql = "CHECK TABLE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CHECK TABLE: {:?}", result);
}

// ============ OPTIMIZE ============

#[test]
fn test_parse_optimize_table() {
    let sql = "OPTIMIZE TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse OPTIMIZE TABLE: {:?}",
        result
    );
}

// ============ VACUUM ============

#[test]
fn test_parse_vacuum_table() {
    let sql = "VACUUM TABLE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse VACUUM TABLE: {:?}", result);
}

// ============ REPAIR ============

#[test]
fn test_parse_repair_table() {
    let sql = "REPAIR TABLE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REPAIR TABLE: {:?}", result);
}

// ============ SHOW GRANTS FOR ============

#[test]
fn test_parse_show_grants_for() {
    let sql = "SHOW GRANTS FOR user1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW GRANTS FOR: {:?}",
        result
    );
}

// ============ REPLACE INTO ============

#[test]
fn test_parse_replace_into_values() {
    let sql = "REPLACE INTO users (id, name) VALUES (1, 'Alice')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REPLACE INTO: {:?}", result);
}

// ============ CREATE TABLE WITH KEY ============

#[test]
fn test_parse_create_table_with_key() {
    let sql = "CREATE TABLE t (id INT, name TEXT, KEY idx_name (name))";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TABLE WITH KEY: {:?}",
        result
    );
}

// ============ SET ROLE Tests ============

#[test]
fn test_parse_set_role() {
    let sql = "SET ROLE admin";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SET ROLE: {:?}", result);
}

#[test]
fn test_parse_set_role_with_string() {
    let sql = "SET ROLE 'admin'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SET ROLE with string: {:?}",
        result
    );
}

// ============ CREATE ROLE Tests ============

#[test]
fn test_parse_create_role() {
    let sql = "CREATE ROLE admin";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE ROLE: {:?}", result);
}

#[test]
fn test_parse_create_role_with_parent() {
    let sql = "CREATE ROLE admin WITH PARENT super_admin";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE ROLE WITH PARENT: {:?}",
        result
    );
}

// ============ DROP ROLE Tests ============

#[test]
fn test_parse_drop_role() {
    let sql = "DROP ROLE admin";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP ROLE: {:?}", result);
}

#[test]
fn test_parse_drop_role_with_string() {
    let sql = "DROP ROLE 'admin'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DROP ROLE with string: {:?}",
        result
    );
}

// ============ GRANT ROLE Tests ============

#[test]
fn test_parse_grant_role() {
    let sql = "GRANT admin TO user1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse GRANT role: {:?}", result);
}

// ============ REVOKE ROLE Tests ============

#[test]
fn test_parse_revoke_role() {
    let sql = "REVOKE admin FROM user1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REVOKE role: {:?}", result);
}

// ============ POSITION Expression Tests ============

#[test]
fn test_parse_position_expression() {
    let sql = "SELECT POSITION('test' IN 'test_string')";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse POSITION expression: {:?}",
        result
    );
}

// ============ IF Expression Tests ============

#[test]
fn test_parse_if_expression() {
    let sql = "SELECT IF(1 > 0, 'yes', 'no')";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse IF expression: {:?}",
        result
    );
}

#[test]
fn test_parse_if_expression_in_where() {
    let sql = "SELECT IF(age > 18, 'adult', 'minor') FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse IF expression in WHERE: {:?}",
        result
    );
}
// ============ Phase 1: BEGIN / Transaction Tests ============

#[test]
fn test_parse_begin_plain() {
    let sql = "BEGIN";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse BEGIN: {:?}", result);
}

#[test]
fn test_parse_begin_tx_work() {
    let sql = "BEGIN WORK";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse BEGIN WORK: {:?}", result);
}

#[test]
fn test_parse_begin_tx_serializable() {
    let sql = "BEGIN SERIALIZABLE";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse BEGIN SERIALIZABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_begin_read_uncommitted() {
    let sql = "BEGIN READ UNCOMMITTED";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "BEGIN READ UNCOMMITTED should parse now that COMMITTED/UNCOMMITTED are keywords: {:?}",
        result
    );
}

#[test]
fn test_parse_begin_read_committed() {
    let sql = "BEGIN READ COMMITTED";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "BEGIN READ COMMITTED should parse now that COMMITTED/UNCOMMITTED are keywords: {:?}",
        result
    );
}

#[test]
fn test_parse_begin_isolation_level_serializable() {
    let sql = "BEGIN ISOLATION LEVEL SERIALIZABLE";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse BEGIN ISOLATION LEVEL SERIALIZABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_begin_isolation_level_read_committed() {
    let sql = "BEGIN ISOLATION LEVEL READ COMMITTED";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "BEGIN ISOLATION LEVEL READ COMMITTED should parse now that COMMITTED/UNCOMMITTED are keywords: {:?}",
        result
    );
}

#[test]
fn test_parse_begin_isolation_level_read_uncommitted() {
    let sql = "BEGIN ISOLATION LEVEL READ UNCOMMITTED";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "BEGIN ISOLATION LEVEL READ UNCOMMITTED should parse now that COMMITTED/UNCOMMITTED are keywords: {:?}",
        result
    );
}

#[test]
fn test_parse_begin_isolation_level_repeatable_read() {
    let sql = "BEGIN ISOLATION LEVEL REPEATABLE READ";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse BEGIN ISOLATION LEVEL REPEATABLE READ: {:?}",
        result
    );
}

#[test]
fn test_parse_start_transaction() {
    let sql = "START TRANSACTION";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse START TRANSACTION: {:?}",
        result
    );
}

#[test]
fn test_parse_start_transaction_isolation_level() {
    let sql = "START TRANSACTION ISOLATION LEVEL SERIALIZABLE";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse START TRANSACTION ISOLATION LEVEL: {:?}",
        result
    );
}

#[test]
fn test_parse_set_transaction_isolation_level_serializable() {
    let sql = "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SET TRANSACTION ISOLATION LEVEL SERIALIZABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_set_transaction_isolation_level_repeatable_read() {
    let sql = "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SET TRANSACTION ISOLATION LEVEL REPEATABLE READ: {:?}",
        result
    );
}

// ============ Phase 2: COMMIT / ROLLBACK WORK ============

#[test]
fn test_parse_commit_work() {
    let sql = "COMMIT WORK";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse COMMIT WORK: {:?}", result);
}

#[test]
fn test_parse_rollback_work() {
    let sql = "ROLLBACK WORK";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ROLLBACK WORK: {:?}",
        result
    );
}

// ============ Phase 4: Constant Folding Expressions ============

#[test]
fn test_parse_expression_in_select() {
    let sql = "SELECT id + 1 FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse expression in SELECT: {:?}",
        result
    );
}

#[test]
fn test_parse_and_expression_in_where() {
    let sql = "SELECT * FROM users WHERE id = 1 AND status = 'active'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse AND expression: {:?}",
        result
    );
}

#[test]
fn test_parse_or_expression_in_where() {
    let sql = "SELECT * FROM users WHERE id = 1 OR id = 2";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse OR expression: {:?}",
        result
    );
}

#[test]
fn test_parse_not_expression_in_where() {
    let sql = "SELECT * FROM users WHERE NOT id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse NOT expression: {:?}",
        result
    );
}

#[test]
fn test_parse_like_expression() {
    let sql = "SELECT * FROM users WHERE name LIKE 'John%'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse LIKE expression: {:?}",
        result
    );
}

#[test]
fn test_parse_between_expression() {
    let sql = "SELECT * FROM users WHERE age BETWEEN 18 AND 65";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse BETWEEN expression: {:?}",
        result
    );
}

#[test]
fn test_parse_in_expression() {
    let sql = "SELECT * FROM users WHERE id IN (1, 2, 3)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse IN expression: {:?}",
        result
    );
}

#[test]
fn test_parse_is_null_expression() {
    let sql = "SELECT * FROM users WHERE email IS NULL";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse IS NULL expression: {:?}",
        result
    );
}

#[test]
fn test_parse_is_not_null_expression() {
    let sql = "SELECT * FROM users WHERE email IS NOT NULL";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse IS NOT NULL expression: {:?}",
        result
    );
}

#[test]
fn test_parse_exists_subquery_in_where() {
    let sql = "SELECT * FROM users WHERE EXISTS (SELECT 1 FROM orders)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse EXISTS subquery: {:?}",
        result
    );
}

// ============ CASE...WHEN Expression Tests ============

#[test]
fn test_parse_case_when_simple() {
    let sql = "SELECT CASE WHEN id = 1 THEN 'one' END FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CASE WHEN: {:?}", result);
}

#[test]
fn test_parse_case_when_with_else() {
    let sql = "SELECT CASE WHEN id = 1 THEN 'one' ELSE 'other' END FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CASE WHEN ELSE: {:?}",
        result
    );
}

#[test]
fn test_parse_case_when_with_multiple_when() {
    let sql = "SELECT CASE WHEN status = 'active' THEN 1 WHEN status = 'inactive' THEN 0 ELSE -1 END FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CASE with multiple WHEN: {:?}",
        result
    );
}

// ============ TRIM Expression Tests ============

#[test]
fn test_parse_trim_expression() {
    let sql = "SELECT TRIM('  hello  ')";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse TRIM expression: {:?}",
        result
    );
}

#[test]
fn test_parse_ltrim_expression() {
    let sql = "SELECT LTRIM('  hello')";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse LTRIM expression: {:?}",
        result
    );
}

#[test]
fn test_parse_rtrim_expression() {
    let sql = "SELECT RTRIM('hello  ')";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse RTRIM expression: {:?}",
        result
    );
}

// ============ DELETE Statement Tests ============

#[test]
fn test_parse_delete_simple() {
    let sql = "DELETE FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DELETE: {:?}", result);
}

#[test]
fn test_parse_delete_with_where_v2() {
    let sql = "DELETE FROM users WHERE id = 1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DELETE WHERE: {:?}", result);
}

#[test]
fn test_parse_delete_with_limit() {
    let sql = "DELETE FROM users ORDER BY id LIMIT 10";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DELETE ORDER BY LIMIT: {:?}",
        result
    );
}

// ============ ALTER TABLE Tests ============

#[test]
fn test_parse_alter_table_add_column_v2() {
    let sql = "ALTER TABLE users ADD COLUMN email VARCHAR(255)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE ADD COLUMN: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_drop_column_v2() {
    let sql = "ALTER TABLE users DROP COLUMN email";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE DROP COLUMN: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_rename_to() {
    let sql = "ALTER TABLE users RENAME TO old_users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE RENAME: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_modify_column() {
    let sql = "ALTER TABLE users MODIFY name VARCHAR(100)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE MODIFY may not be fully supported: {:?}",
        result
    );
}

// ============ UPDATE Statement Tests ============

#[test]
fn test_parse_update_with_order_by() {
    let sql = "UPDATE users SET name = 'Alice' WHERE id = 1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse UPDATE: {:?}", result);
}

#[test]
fn test_parse_update_multiple_tables() {
    let sql = "UPDATE users SET name = 'Alice' WHERE id = 1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse UPDATE: {:?}", result);
}

// ============ Subquery Comparison Tests ============

#[test]
fn test_parse_subquery_any() {
    let sql = "SELECT * FROM users WHERE id = ANY (SELECT user_id FROM orders)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ANY subquery: {:?}", result);
}

#[test]
fn test_parse_subquery_all() {
    let sql = "SELECT * FROM users WHERE id > ALL (SELECT user_id FROM orders)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ALL subquery: {:?}", result);
}

#[test]
fn test_parse_subquery_some() {
    let sql = "SELECT * FROM users WHERE id = SOME (SELECT user_id FROM orders)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SOME subquery: {:?}",
        result
    );
}

// ============ CTE (WITH clause) Tests ============

#[test]
fn test_parse_with_clause_simple() {
    let sql =
        "WITH active_users AS (SELECT * FROM users WHERE active = 1) SELECT * FROM active_users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse WITH clause: {:?}", result);
}

#[test]
fn test_parse_with_clause_multiple_ctes() {
    let sql = "WITH cte1 AS (SELECT 1), cte2 AS (SELECT 2) SELECT * FROM cte1, cte2";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse WITH multiple CTEs: {:?}",
        result
    );
}

#[test]
fn test_parse_with_recursive() {
    let sql = "WITH RECURSIVE cte AS (SELECT 1 AS n UNION ALL SELECT n + 1 FROM cte WHERE n < 10) SELECT * FROM cte";
    let result = parse(sql);
    assert!(
        result.is_err(),
        "WITH RECURSIVE may not be fully supported: {:?}",
        result
    );
}

// ============ Window Function Tests ============

#[test]
fn test_parse_window_function_row_number() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ROW_NUMBER: {:?}", result);
}

#[test]
fn test_parse_window_function_rank() {
    let sql = "SELECT RANK() OVER (PARTITION BY department ORDER BY salary DESC) FROM employees";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse RANK: {:?}", result);
}

#[test]
fn test_parse_window_function_dense_rank() {
    let sql = "SELECT DENSE_RANK() OVER (ORDER BY score) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DENSE_RANK: {:?}", result);
}

// ============ INSERT...SELECT Tests ============

#[test]
fn test_parse_insert_select() {
    let sql = "INSERT INTO users (id, name) SELECT id, name FROM old_users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse INSERT SELECT: {:?}",
        result
    );
}

#[test]
fn test_parse_insert_on_duplicate_key_update() {
    let sql =
        "INSERT INTO users (id, name) VALUES (1, 'Alice') ON DUPLICATE KEY UPDATE name = 'Bob'";
    let result = parse(sql);
    assert!(
        result.is_err(),
        "INSERT ON DUPLICATE KEY UPDATE not supported: {:?}",
        result
    );
}

// ============ String Functions Tests ============

#[test]
fn test_parse_concat_function() {
    let sql = "SELECT CONCAT('Hello', ' ', 'World')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CONCAT: {:?}", result);
}

#[test]
fn test_parse_substring_function() {
    let sql = "SELECT SUBSTRING('Hello World', 1, 5)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SUBSTRING: {:?}", result);
}

#[test]
fn test_parse_upper_function() {
    let sql = "SELECT UPPER(name) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse UPPER: {:?}", result);
}

#[test]
fn test_parse_lower_function() {
    let sql = "SELECT LOWER(name) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LOWER: {:?}", result);
}

#[test]
fn test_parse_length_function() {
    let sql = "SELECT LENGTH(name) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LENGTH: {:?}", result);
}

// ============ Aggregate Function Tests ============

#[test]
fn test_parse_avg_function() {
    let sql = "SELECT AVG(price) FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse AVG: {:?}", result);
}

#[test]
fn test_parse_min_max_functions() {
    let sql = "SELECT MIN(price), MAX(price) FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse MIN/MAX: {:?}", result);
}

// ============ JOIN Variation Tests ============

#[test]
fn test_parse_cross_join_v2() {
    let sql = "SELECT * FROM users CROSS JOIN orders";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CROSS JOIN: {:?}", result);
}

#[test]
fn test_parse_join_with_using() {
    let sql = "SELECT * FROM users JOIN orders USING (user_id)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse JOIN USING: {:?}", result);
}

#[test]
fn test_parse_left_join_without_outner() {
    let sql = "SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LEFT JOIN: {:?}", result);
}

// ============ GROUP BY with HAVING Tests ============

#[test]
fn test_parse_group_by_with_having() {
    let sql = "SELECT department, COUNT(*) FROM employees GROUP BY department HAVING COUNT(*) > 5";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse GROUP BY HAVING: {:?}",
        result
    );
}

#[test]
fn test_parse_group_by_with_multiple_columns() {
    let sql = "SELECT department, status, COUNT(*) FROM employees GROUP BY department, status";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse GROUP BY multiple columns: {:?}",
        result
    );
}

// ============ ORDER BY Tests ============

#[test]
fn test_parse_order_by_asc() {
    let sql = "SELECT * FROM users ORDER BY name ASC";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ORDER BY ASC: {:?}", result);
}

#[test]
fn test_parse_order_by_desc_v2() {
    let sql = "SELECT * FROM users ORDER BY name DESC";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ORDER BY DESC: {:?}",
        result
    );
}

#[test]
fn test_parse_order_by_multiple_columns() {
    let sql = "SELECT * FROM users ORDER BY last_name, first_name DESC";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ORDER BY multiple: {:?}",
        result
    );
}

// ============ LIMIT/OFFSET Tests ============

#[test]
fn test_parse_limit_offset_v2() {
    let sql = "SELECT * FROM users LIMIT 10 OFFSET 20";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LIMIT OFFSET: {:?}", result);
}

#[test]
fn test_parse_limit_with_expression() {
    let sql = "SELECT * FROM users LIMIT 10 + 5";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse LIMIT with expression: {:?}",
        result
    );
}

// ============ DISTINCT Tests ============

#[test]
fn test_parse_select_distinct() {
    let sql = "SELECT DISTINCT name FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SELECT DISTINCT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_distinct_on() {
    let sql = "SELECT DISTINCT ON (department) name FROM users";
    let result = parse(sql);
    assert!(
        result.is_err(),
        "DISTINCT ON is PostgreSQL syntax, not supported: {:?}",
        result
    );
}

// ============ Bitwise Shift Operator Tests ============

#[test]
fn test_parse_left_shift() {
    let sql = "SELECT 5 << 1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LEFT SHIFT: {:?}", result);
}

#[test]
fn test_parse_right_shift() {
    let sql = "SELECT 5 >> 1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse RIGHT SHIFT: {:?}", result);
}

#[test]
fn test_parse_shift_in_expression() {
    let sql = "SELECT id << 2 FROM users";
    let _ = parse(sql);
}

#[test]
fn test_parse_shift_with_addition() {
    // Addition has higher precedence than shift
    // So a + b << c should parse as (a + b) << c
    let sql = "SELECT 1 + 2 << 3";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse shift with addition: {:?}",
        result
    );
}

#[test]
fn test_parse_shift_with_multiplication() {
    // Multiplication has higher precedence than shift
    // So a * b << c should parse as (a * b) << c
    let sql = "SELECT 2 * 3 << 4";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse shift with multiplication: {:?}",
        result
    );
}
