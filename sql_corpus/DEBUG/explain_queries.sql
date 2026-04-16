-- === EXPLAIN Query Analysis Test Suite ===

-- === CASE: EXPLAIN basic SELECT ===
-- EXPECT: success
EXPLAIN SELECT * FROM users WHERE id = 1;

-- === CASE: EXPLAIN SELECT with JOIN ===
-- EXPECT: success
EXPLAIN SELECT u.id, o.order_id FROM users u JOIN orders o ON u.id = o.user_id;

-- === CASE: EXPLAIN SELECT with subquery ===
-- EXPECT: success
EXPLAIN SELECT * FROM users WHERE id IN (SELECT user_id FROM orders);

-- === CASE: EXPLAIN SELECT with aggregate ===
-- EXPECT: success
EXPLAIN SELECT COUNT(*) FROM users GROUP BY id;

-- === CASE: EXPLAIN SELECT with ORDER BY ===
-- EXPECT: success
EXPLAIN SELECT * FROM users ORDER BY id DESC;

-- === CASE: EXPLAIN SELECT with LIMIT ===
-- EXPECT: success
EXPLAIN SELECT * FROM users LIMIT 10;

-- === CASE: EXPLAIN INSERT ===
-- EXPECT: success
EXPLAIN INSERT INTO users (id, name, email) VALUES (1000, 'Explain', 'explain@test.com');

-- === CASE: EXPLAIN UPDATE ===
-- EXPECT: success
EXPLAIN UPDATE users SET email = 'updated@test.com' WHERE id = 1;

-- === CASE: EXPLAIN DELETE ===
-- EXPECT: success
EXPLAIN DELETE FROM users WHERE id = 1;

-- === CASE: EXPLAIN with index hint ===
-- EXPECT: success
EXPLAIN SELECT * FROM users USE INDEX (PRIMARY) WHERE id > 5;

-- === CASE: EXPLAIN QUERY PLAN ===
-- EXPECT: success
EXPLAIN QUERY PLAN SELECT * FROM users WHERE id > 10;

-- === CASE: EXPLAIN with DISTINCT ===
-- EXPECT: success
EXPLAIN SELECT DISTINCT name FROM users;

-- === CASE: EXPLAIN with GROUP BY ===
-- EXPECT: success
EXPLAIN SELECT id, COUNT(*) FROM users GROUP BY id;

-- === CASE: EXPLAIN with HAVING ===
-- EXPECT: success
EXPLAIN SELECT user_id, COUNT(*) FROM orders GROUP BY user_id HAVING COUNT(*) > 2;

-- === CASE: EXPLAIN with UNION ===
-- EXPECT: success
EXPLAIN SELECT * FROM users WHERE id <= 5 UNION SELECT * FROM users WHERE id > 5;
