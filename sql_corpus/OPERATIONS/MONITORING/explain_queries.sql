-- === SQLRustGo Monitoring and Diagnostics Test Suite ===
-- Tests for EXPLAIN, SHOW commands, and information_schema

-- === SETUP ===
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT);
CREATE INDEX idx_users_email ON users(email);
INSERT INTO users VALUES (1, 'Alice', 'alice@example.com');
INSERT INTO users VALUES (2, 'Bob', 'bob@example.com');

CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, amount REAL);
CREATE INDEX idx_orders_user ON orders(user_id);
INSERT INTO orders VALUES (1, 1, 100.50);
INSERT INTO orders VALUES (2, 2, 250.00);

-- === CASE: Explain Basic Select ===
-- EXPECT: success
EXPLAIN SELECT * FROM users WHERE id = 1;

-- === CASE: Explain Analyze ===
-- EXPECT: success
EXPLAIN ANALYZE SELECT * FROM users WHERE id = 1;

-- === CASE: Explain with Format ===
-- EXPECT: success
EXPLAIN FORMAT=JSON SELECT * FROM users WHERE id = 1;

-- === CASE: Explain Select with Index ===
-- EXPECT: success
EXPLAIN SELECT * FROM users WHERE email = 'alice@example.com';

-- === CASE: Show Indexes ===
-- EXPECT: 1 row
SHOW INDEXES FROM users;

-- === CASE: Show Indexes from Table ===
-- EXPECT: 1 row
SHOW INDEXES FROM orders;

-- === CASE: Show Create Table ===
-- EXPECT: success
SHOW CREATE TABLE users;

-- === CASE: Information Schema Tables ===
-- EXPECT: rows > 0
SELECT * FROM information_schema.tables WHERE table_name = 'users';

-- === CASE: Information Schema Columns ===
-- EXPECT: 3 rows
SELECT * FROM information_schema.columns WHERE table_name = 'users';

-- === CASE: Information Schema Indexes ===
-- EXPECT: 1 row
SELECT * FROM information_schema.indexes WHERE table_name = 'users';

-- === CASE: Information Schema Statistics ===
-- EXPECT: rows > 0
SELECT * FROM information_schema.statistics WHERE table_name = 'users';

-- === CASE: Explain Join Query ===
-- EXPECT: success
EXPLAIN SELECT u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id;

-- === CASE: Explain Aggregate Query ===
-- EXPECT: success
EXPLAIN SELECT user_id, SUM(amount) FROM orders GROUP BY user_id;

-- === CASE: Show Tables ===
-- EXPECT: 2 rows
SHOW TABLES;

-- === CASE: Show Tables Like ===
-- EXPECT: 1 row
SHOW TABLES LIKE 'users';

-- === CASE: Show Databases ===
-- EXPECT: rows > 0
SHOW DATABASES;

-- === CASE: Show Columns ===
-- EXPECT: 3 rows
SHOW COLUMNS FROM users;

-- === CASE: Show Full Columns ===
-- EXPECT: success
SHOW FULL COLUMNS FROM users;

-- === CASE: Buffer Pool Status ===
-- EXPECT: rows > 0
SHOW STATUS LIKE 'buffer_pool%';

-- === CASE: Table Fragmentation Check ===
-- EXPECT: success
CHECK TABLE users;

-- === TEARDOWN ===
DROP TABLE orders;
DROP TABLE users;
