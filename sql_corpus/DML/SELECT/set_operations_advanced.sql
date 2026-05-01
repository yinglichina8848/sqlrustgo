-- === SKIP ===

-- === Set Operations Test Suite ===

-- === CASE: UNION ALL ===
-- EXPECT: 20 rows
SELECT id, name FROM users WHERE id <= 10
UNION ALL
SELECT id, name FROM users WHERE id <= 10;

-- === CASE: UNION DISTINCT ===
-- EXPECT: 10 rows
SELECT id, name FROM users WHERE id <= 10
UNION
SELECT id, name FROM users WHERE id <= 10;

-- === CASE: INTERSECT ===
-- EXPECT: 5 rows
SELECT id, name FROM users WHERE id <= 5
INTERSECT
SELECT id, name FROM users WHERE id >= 3 AND id <= 7;

-- === CASE: EXCEPT ===
-- EXPECT: 3 rows
SELECT id, name FROM users WHERE id <= 5
EXCEPT
SELECT id, name FROM users WHERE id >= 4;

-- === CASE: UNION with ORDER BY ===
-- EXPECT: 15 rows
SELECT id, name FROM users WHERE id <= 8
UNION
SELECT id, name FROM users WHERE id >= 6 AND id <= 10
ORDER BY id DESC;

-- === CASE: UNION with LIMIT ===
-- EXPECT: 5 rows
SELECT id, name FROM users
UNION
SELECT id, name FROM users
ORDER BY id
LIMIT 5;

-- === CASE: UNION with WHERE ===
-- EXPECT: 8 rows
SELECT id, name, email FROM users WHERE id <= 5 AND email LIKE '%@example.com'
UNION
SELECT id, name, email FROM users WHERE id > 5 AND id <= 10;

-- === CASE: UNION different columns ===
-- EXPECT: 10 rows
SELECT id, name, 'user' as type FROM users WHERE id <= 5
UNION ALL
SELECT id, name, 'admin' as type FROM users WHERE id > 5 AND id <= 10;

-- === CASE: UNION with aggregate ===
-- EXPECT: 2 rows
SELECT 'users' as source, COUNT(*) as cnt FROM users
UNION
SELECT 'orders' as source, COUNT(*) as cnt FROM orders;

-- === CASE: UNION with GROUP BY ===
-- EXPECT: 4 rows
SELECT 'group1' as grp, COUNT(*) as cnt FROM users WHERE id <= 5
UNION
SELECT 'group2' as grp, COUNT(*) as cnt FROM users WHERE id > 5;

-- === CASE: Multiple UNION ===
-- EXPECT: 15 rows
SELECT id FROM users WHERE id <= 5
UNION
SELECT id FROM users WHERE id > 5 AND id <= 10
UNION
SELECT id FROM users WHERE id > 10 AND id <= 15;

-- === CASE: UNION with subquery ===
-- EXPECT: 5 rows
SELECT * FROM (SELECT id, name FROM users WHERE id <= 5
UNION
SELECT id, name FROM users WHERE id > 5 AND id <= 10) t
WHERE id > 3;

-- === CASE: UNION with NULL ===
-- EXPECT: 12 rows
SELECT id, name, email FROM users WHERE id <= 6
UNION
SELECT id, name, CAST(NULL AS TEXT) FROM users WHERE id > 6 AND id <= 10;

-- === CASE: UNION with DISTINCT ===
-- EXPECT: 5 rows
SELECT DISTINCT name FROM (SELECT name FROM users WHERE id <= 5
UNION ALL
SELECT name FROM users WHERE id <= 5) t;

-- === CASE: EXCEPT with ORDER BY ===
-- EXPECT: 2 rows
SELECT id, name FROM users WHERE id <= 5
EXCEPT
SELECT id, name FROM users WHERE id >= 3
ORDER BY id;

-- === CASE: INTERSECT with LIMIT ===
-- EXPECT: 3 rows
SELECT id, name FROM users WHERE id <= 7
INTERSECT
SELECT id, name FROM users WHERE id >= 4 AND id <= 10
ORDER BY id
LIMIT 3;
