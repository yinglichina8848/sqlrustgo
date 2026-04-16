-- === Temporary Tables Test Suite ===

-- === CASE: Create temp table ===
-- EXPECT: success
CREATE TEMPORARY TABLE temp_users AS SELECT * FROM users WHERE id <= 5;

-- === CASE: Select from temp table ===
-- EXPECT: 5 rows
SELECT * FROM temp_users;

-- === CASE: Drop temp table ===
-- EXPECT: success
DROP TABLE temp_users;

-- === CASE: Create temp table with specific columns ===
-- EXPECT: success
CREATE TEMPORARY TABLE temp_orders AS
SELECT user_id, COUNT(*) as order_count, SUM(total) as total_spent
FROM orders GROUP BY user_id;

-- === CASE: Temp table with aggregate ===
-- EXPECT: 5 rows
SELECT * FROM temp_orders WHERE order_count > 2;

-- === CASE: Temp table in JOIN ===
-- EXPECT: 5 rows
SELECT u.name, t.order_count
FROM users u
JOIN temp_orders t ON u.id = t.user_id;

-- === CASE: Temp table with INDEX ===
-- EXPECT: success
CREATE TEMPORARY TABLE temp_idx AS SELECT * FROM users;
CREATE INDEX temp_idx_id ON temp_idx(id);

-- === CASE: Temp table with unique constraint ===
-- EXPECT: success
CREATE TEMPORARY TABLE temp_unique (id INTEGER, name TEXT, UNIQUE(id));

-- === CASE: Temp table with primary key ===
-- EXPECT: success
CREATE TEMPORARY TABLE temp_pk (id INTEGER PRIMARY KEY, name TEXT);

-- === CASE: Temp table with NOT NULL ===
-- EXPECT: success
CREATE TEMPORARY TABLE temp_notnull (id INTEGER NOT NULL, name TEXT);

-- === CASE: Insert into temp table ===
-- EXPECT: 3 rows affected
INSERT INTO temp_users (id, name, email) VALUES (1001, 'Temp1', 'temp1@test.com'), (1002, 'Temp2', 'temp2@test.com'), (1003, 'Temp3', 'temp3@test.com');

-- === CASE: Update temp table ===
-- EXPECT: 2 rows affected
UPDATE temp_users SET email = 'updated@test.com' WHERE id > 1000;

-- === CASE: Delete from temp table ===
-- EXPECT: 2 rows affected
DELETE FROM temp_users WHERE id > 1001;

-- === CASE: Drop temp table if exists ===
-- EXPECT: success
DROP TABLE IF EXISTS nonexistent_temp;

-- === CASE: Create temp table like another ===
-- EXPECT: success
CREATE TEMPORARY TABLE temp_copy AS SELECT * FROM users WHERE 1=0;

-- === CASE: Temp table with CHECK constraint ===
-- EXPECT: success
CREATE TEMPORARY TABLE temp_check (id INTEGER, value INTEGER CHECK (value > 0));

-- === CASE: Temp table with DEFAULT ===
-- EXPECT: success
CREATE TEMPORARY TABLE temp_default (id INTEGER, status TEXT DEFAULT 'active');
