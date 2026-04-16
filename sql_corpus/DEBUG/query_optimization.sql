-- === Query Optimization Hints Test Suite ===

-- === CASE: USE INDEX hint ===
-- EXPECT: 5 rows
SELECT * FROM users USE INDEX (idx_users_email) WHERE id <= 5;

-- === CASE: FORCE INDEX hint ===
-- EXPECT: 5 rows
SELECT * FROM users FORCE INDEX (idx_users_email) WHERE id <= 5;

-- === CASE: IGNORE INDEX hint ===
-- EXPECT: 10 rows
SELECT * FROM users IGNORE INDEX (idx_users_email) WHERE id > 0;

-- === CASE: INDEX hint on join ===
-- EXPECT: 3 rows
SELECT * FROM users u
JOIN orders o USE INDEX (idx_orders_user) ON u.id = o.user_id
WHERE u.id <= 3;

-- === CASE: ORDER BY optimization ===
-- EXPECT: 10 rows
SELECT * FROM users ORDER BY id LIMIT 10;

-- === CASE: GROUP BY optimization ===
-- EXPECT: 5 rows
SELECT user_id, COUNT(*) FROM orders GROUP BY user_id;

-- === CASE: DISTINCT optimization ===
-- EXPECT: 5 rows
SELECT DISTINCT user_id FROM orders;

-- === CASE: LIMIT optimization ===
-- EXPECT: 3 rows
SELECT * FROM users LIMIT 3;

-- === CASE: Subquery optimization ===
-- EXPECT: 5 rows
SELECT * FROM users WHERE id IN (SELECT user_id FROM orders LIMIT 5);

-- === CASE: JOIN order optimization ===
-- EXPECT: 5 rows
SELECT u.id, o.order_id
FROM users u
INNER JOIN orders o ON u.id = o.user_id
WHERE u.id <= 5;

-- === CASE: Covering index query ===
-- EXPECT: 3 rows
SELECT id, name FROM users WHERE id <= 3;

-- === CASE: Index range scan ===
-- EXPECT: 5 rows
SELECT * FROM orders WHERE total BETWEEN 100 AND 300;

-- === CASE: Index prefix scan ===
-- EXPECT: 5 rows
SELECT * FROM users WHERE name LIKE 'A%';

-- === CASE: Query with proper indexing ===
-- EXPECT: 5 rows
SELECT u.id, u.name, COUNT(o.order_id) AS order_count
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
WHERE u.id <= 5
GROUP BY u.id, u.name;
