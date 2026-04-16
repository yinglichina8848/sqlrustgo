-- === UNION Operations Test Suite ===

-- === CASE: UNION basic ===
-- EXPECT: 15 rows
SELECT id, name FROM users WHERE id <= 5
UNION
SELECT id, name FROM users WHERE id > 5 AND id <= 10;

-- === CASE: UNION with different columns ===
-- EXPECT: 15 rows
SELECT id, name, 'user' as type FROM users WHERE id <= 5
UNION
SELECT id, CAST(NULL AS TEXT), CAST(id AS TEXT) FROM users WHERE id > 5 AND id <= 10;

-- === CASE: UNION ALL ===
-- EXPECT: 10 rows
SELECT id, name FROM users WHERE id <= 5
UNION ALL
SELECT id, name FROM users WHERE id <= 5;

-- === CASE: UNION with ORDER BY ===
-- EXPECT: 10 rows
SELECT id, name FROM users WHERE id <= 5
UNION
SELECT id, name FROM users WHERE id > 5 AND id <= 10
ORDER BY id DESC;

-- === CASE: UNION with LIMIT ===
-- EXPECT: 5 rows
SELECT id, name FROM users
UNION
SELECT id, name FROM users
ORDER BY id
LIMIT 5;

-- === CASE: UNION DISTINCT (default) ===
-- EXPECT: 5 rows
SELECT id, name FROM users WHERE id <= 5
UNION DISTINCT
SELECT id, name FROM users WHERE id <= 5;

-- === CASE: UNION with WHERE clause ===
-- EXPECT: 7 rows
SELECT id, name, email FROM users WHERE id <= 5 AND email LIKE '%@example.com'
UNION
SELECT id, name, email FROM users WHERE id > 5 AND id <= 10;

-- === CASE: UNION multiple SELECTs ===
-- EXPECT: 15 rows
SELECT id, name FROM users WHERE id <= 3
UNION
SELECT id, name FROM users WHERE id > 3 AND id <= 7
UNION
SELECT id, name FROM users WHERE id > 7 AND id <= 10;

-- === CASE: UNION with aggregate ===
-- EXPECT: 3 rows
SELECT 'users' as source, COUNT(*) as cnt FROM users WHERE id <= 5
UNION
SELECT 'orders' as source, COUNT(*) as cnt FROM orders WHERE user_id <= 5;

-- === CASE: UNION with JOIN ===
-- EXPECT: 8 rows
SELECT u.id, u.name, o.order_id FROM users u
JOIN orders o ON u.id = o.user_id WHERE u.id <= 3
UNION
SELECT u.id, u.name, o.order_id FROM users u
JOIN orders o ON u.id = o.user_id WHERE u.id > 3 AND u.id <= 6;

-- === CASE: UNION ALL with UNION ===
-- EXPECT: 13 rows
SELECT id, name FROM users WHERE id <= 5
UNION ALL
SELECT id, name FROM users WHERE id > 5 AND id <= 8
UNION
SELECT id, name FROM users WHERE id > 8 AND id <= 10;

-- === CASE: UNION with GROUP BY ===
-- EXPECT: 2 rows
SELECT 'group1' as grp, COUNT(*) as cnt FROM users WHERE id <= 5
UNION
SELECT 'group2' as grp, COUNT(*) as cnt FROM users WHERE id > 5;

-- === CASE: UNION with subquery ===
-- EXPECT: 5 rows
SELECT * FROM (SELECT id, name FROM users WHERE id <= 5
UNION
SELECT id, name FROM users WHERE id > 5 AND id <= 10) t
WHERE id <= 7;
