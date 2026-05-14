-- MERGE Statement Test Cases
-- Compatibility: MySQL 8.0+ (MERGE is a standard SQL feature)

-- ============================================
-- 1. Basic MERGE Operations
-- ============================================

-- MERGE: Update when matched
CREATE TABLE target_users (id INTEGER PRIMARY KEY, name TEXT, email TEXT);
CREATE TABLE source_users (id INTEGER, name TEXT, email TEXT);
INSERT INTO target_users VALUES (1, 'Alice', 'alice@example.com');
INSERT INTO source_users VALUES (1, 'Alice Updated', 'alice.new@example.com');
MERGE INTO target_users USING source_users ON target_users.id = source_users.id WHEN MATCHED THEN UPDATE SET name = source_users.name, email = source_users.email;

-- MERGE: Insert when not matched
CREATE TABLE target_products (id INTEGER PRIMARY KEY, name TEXT, price INTEGER);
CREATE TABLE source_products (id INTEGER, name TEXT, price INTEGER);
INSERT INTO target_products VALUES (1, 'Widget', 100);
INSERT INTO source_products VALUES (2, 'Gadget', 200);
MERGE INTO target_products USING source_products ON target_products.id = source_products.id WHEN NOT MATCHED THEN INSERT (id, name, price) VALUES (source_products.id, source_products.name, source_products.price);

-- MERGE: Both matched and not matched
CREATE TABLE target_orders (id INTEGER PRIMARY KEY, status TEXT, total INTEGER);
CREATE TABLE source_orders (id INTEGER, status TEXT, total INTEGER);
INSERT INTO target_orders VALUES (1, 'pending', 100);
INSERT INTO source_orders VALUES (1, 'shipped', 100), (2, 'new', 300);
MERGE INTO target_orders USING source_orders ON target_orders.id = source_orders.id WHEN MATCHED THEN UPDATE SET status = source_orders.status WHEN NOT MATCHED THEN INSERT (id, status, total) VALUES (source_orders.id, source_orders.status, source_orders.total);

-- ============================================
-- 2. MERGE with Complex Conditions
-- ============================================

-- MERGE with composite ON condition
CREATE TABLE target_inventory (product_id INTEGER, warehouse_id INTEGER, quantity INTEGER);
CREATE TABLE source_inventory (product_id INTEGER, warehouse_id INTEGER, quantity INTEGER);
INSERT INTO target_inventory VALUES (1, 1, 50);
INSERT INTO source_inventory VALUES (1, 1, 75), (1, 2, 25);
MERGE INTO target_inventory USING source_inventory ON target_inventory.product_id = source_inventory.product_id AND target_inventory.warehouse_id = source_inventory.warehouse_id WHEN MATCHED THEN UPDATE SET quantity = source_inventory.quantity WHEN NOT MATCHED THEN INSERT (product_id, warehouse_id, quantity) VALUES (source_inventory.product_id, source_inventory.warehouse_id, source_inventory.quantity);

-- MERGE with version check (AND condition)
CREATE TABLE target_items (id INTEGER PRIMARY KEY, version INTEGER, data TEXT);
CREATE TABLE source_items (id INTEGER, version INTEGER, data TEXT);
INSERT INTO target_items VALUES (1, 1, 'old');
INSERT INTO source_items VALUES (1, 2, 'newer');
MERGE INTO target_items USING source_items ON target_items.id = source_items.id AND target_items.version < source_items.version WHEN MATCHED THEN UPDATE SET data = source_items.data, version = source_items.version;

-- ============================================
-- 3. MERGE with Multiple Columns
-- ============================================

-- MERGE updating multiple columns
CREATE TABLE target_stats (id INTEGER PRIMARY KEY, a INTEGER, b INTEGER, c INTEGER);
CREATE TABLE source_stats (id INTEGER, a INTEGER, b INTEGER, c INTEGER);
INSERT INTO target_stats VALUES (1, 1, 2, 3);
INSERT INTO source_stats VALUES (1, 10, 20, 30);
MERGE INTO target_stats USING source_stats ON target_stats.id = source_stats.id WHEN MATCHED THEN UPDATE SET a = source_stats.a, b = source_stats.b, c = source_stats.c;

-- ============================================
-- 4. MERGE with No Matches
-- ============================================

-- MERGE where no rows match
CREATE TABLE target_empty (id INTEGER PRIMARY KEY, value TEXT);
CREATE TABLE source_empty (id INTEGER, value TEXT);
INSERT INTO target_empty VALUES (1, 'original');
INSERT INTO source_empty VALUES (99, 'orphan');
MERGE INTO target_empty USING source_empty ON target_empty.id = source_empty.id WHEN MATCHED THEN UPDATE SET value = source_empty.value;

-- ============================================
-- 5. MERGE with Multiple Source Rows
-- ============================================

-- MERGE where multiple source rows match (last wins)
CREATE TABLE target_multi (id INTEGER PRIMARY KEY, value TEXT);
CREATE TABLE source_multi (id INTEGER, value TEXT);
INSERT INTO target_multi VALUES (1, 'single');
INSERT INTO source_multi VALUES (1, 'first'), (1, 'second'), (1, 'third');
MERGE INTO target_multi USING source_multi ON target_multi.id = source_multi.id WHEN MATCHED THEN UPDATE SET value = source_multi.value;

-- ============================================
-- 6. MERGE with All Source Matched
-- ============================================

-- MERGE where all source rows have matches
CREATE TABLE target_all (id INTEGER PRIMARY KEY, name TEXT);
CREATE TABLE source_all (id INTEGER, name TEXT);
INSERT INTO target_all VALUES (1, 't1'), (2, 't2'), (3, 't3');
INSERT INTO source_all VALUES (1, 's1'), (2, 's2'), (3, 's3');
MERGE INTO target_all USING source_all ON target_all.id = source_all.id WHEN MATCHED THEN UPDATE SET name = source_all.name;

-- ============================================
-- 7. MERGE with Literals in VALUES
-- ============================================

-- MERGE with literal values in INSERT
CREATE TABLE target_literal (id INTEGER PRIMARY KEY, data TEXT, count INTEGER);
CREATE TABLE source_literal (id INTEGER, data TEXT);
INSERT INTO target_literal VALUES (1, 'exists', 0);
INSERT INTO source_literal VALUES (2, 'new');
MERGE INTO target_literal USING source_literal ON target_literal.id = source_literal.id WHEN NOT MATCHED THEN INSERT (id, data, count) VALUES (source_literal.id, source_literal.data, 1);

-- ============================================
-- 8. MERGE with Qualified Column References
-- ============================================

-- MERGE with fully qualified column references
CREATE TABLE target_qualified (id INTEGER PRIMARY KEY, name TEXT, status TEXT);
CREATE TABLE source_qualified (id INTEGER, name TEXT, status TEXT);
INSERT INTO target_qualified VALUES (1, 'old_name', 'pending');
INSERT INTO source_qualified VALUES (1, 'new_name', 'complete');
MERGE INTO target_qualified USING source_qualified ON target_qualified.id = source_qualified.id WHEN MATCHED THEN UPDATE SET target_qualified.name = source_qualified.name, target_qualified.status = source_qualified.status;

-- Cleanup
DROP TABLE IF EXISTS target_users;
DROP TABLE IF EXISTS source_users;
DROP TABLE IF EXISTS target_products;
DROP TABLE IF EXISTS source_products;
DROP TABLE IF EXISTS target_orders;
DROP TABLE IF EXISTS source_orders;
DROP TABLE IF EXISTS target_inventory;
DROP TABLE IF EXISTS source_inventory;
DROP TABLE IF EXISTS target_items;
DROP TABLE IF EXISTS source_items;
DROP TABLE IF EXISTS target_stats;
DROP TABLE IF EXISTS source_stats;
DROP TABLE IF EXISTS target_empty;
DROP TABLE IF EXISTS source_empty;
DROP TABLE IF EXISTS target_multi;
DROP TABLE IF EXISTS source_multi;
DROP TABLE IF EXISTS target_all;
DROP TABLE IF EXISTS source_all;
DROP TABLE IF EXISTS target_literal;
DROP TABLE IF EXISTS source_literal;
DROP TABLE IF EXISTS target_qualified;
DROP TABLE IF EXISTS source_qualified;