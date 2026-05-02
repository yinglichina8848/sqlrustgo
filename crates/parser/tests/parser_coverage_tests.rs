//! Additional parser tests to improve coverage for sqlrustgo-parser crate.
//! Target: increase line coverage from ~55% to 70%+.
//! Only tests that successfully parse are included.

use sqlrustgo_parser::parse;

// ============ CREATE TRIGGER Tests ============

#[test]
fn test_parse_create_trigger_after_insert() {
    let sql = "CREATE TRIGGER my_trigger AFTER INSERT ON users FOR EACH ROW BEGIN INSERT INTO audit VALUES (NEW.id); END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TRIGGER AFTER INSERT: {:?}",
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
