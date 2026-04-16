-- === Self Join Test Suite ===

-- === CASE: Self join basic ===
-- EXPECT: 10 rows
SELECT a.id, a.name as name1, b.id, b.name as name2
FROM users a, users b
WHERE a.id < b.id AND a.id <= 3;

-- === CASE: Self join with aggregate ===
-- EXPECT: 5 rows
SELECT a.id, a.name, COUNT(b.id) as similar_users
FROM users a
LEFT JOIN users b ON a.id != b.id AND a.name = b.name
WHERE a.id <= 5
GROUP BY a.id, a.name;

-- === CASE: Self join with condition ===
-- EXPECT: 6 rows
SELECT a.id, a.name, b.id, b.name
FROM users a
JOIN users b ON a.id < b.id
WHERE a.id <= 4;

-- === CASE: Self join using USING ===
-- EXPECT: 5 rows
SELECT a.id, a.name, b.name as related_name
FROM users a
JOIN users b ON a.id < b.id AND a.name = b.name
WHERE a.id <= 5;

-- === CASE: Self join with LEFT ===
-- EXPECT: 5 rows
SELECT a.id, a.name, b.id as related_id, b.name as related_name
FROM users a
LEFT JOIN users b ON a.id = b.id + 1
WHERE a.id <= 5;

-- === CASE: Self join with multiple conditions ===
-- EXPECT: 4 rows
SELECT a.id, a.name, a.email, b.id, b.name
FROM users a
JOIN users b ON a.id < b.id AND a.email = b.email
WHERE a.id <= 4;

-- === CASE: Self join with subquery ===
-- EXPECT: 3 rows
SELECT a.id, a.name
FROM users a
WHERE EXISTS (SELECT 1 FROM users b WHERE b.id > a.id AND b.name = a.name)
AND a.id <= 5;

-- === CASE: Self join with GROUP BY ===
-- EXPECT: 3 rows
SELECT a.id, a.name, COUNT(b.id) as cnt
FROM users a
LEFT JOIN users b ON a.name = b.name AND a.id != b.id
WHERE a.id <= 5
GROUP BY a.id, a.name;

-- === CASE: Self join with ORDER BY ===
-- EXPECT: 6 rows
SELECT a.id, a.name, b.id, b.name
FROM users a
JOIN users b ON a.id < b.id
WHERE a.id <= 4
ORDER BY a.id, b.id DESC;

-- === CASE: Self join with LIMIT ===
-- EXPECT: 3 rows
SELECT a.id, a.name, b.id, b.name
FROM users a
JOIN users b ON a.id < b.id
WHERE a.id <= 5
ORDER BY a.id
LIMIT 3;

-- === CASE: Self join with DISTINCT ===
-- EXPECT: 3 rows
SELECT DISTINCT a.name, b.email
FROM users a
JOIN users b ON a.id = b.id + 1
WHERE a.id <= 5;

-- === CASE: Self join with COALESCE ===
-- EXPECT: 5 rows
SELECT a.id, a.name, COALESCE(b.name, 'none') as paired_name
FROM users a
LEFT JOIN users b ON a.id = b.id + 1
WHERE a.id <= 5;

-- === CASE: Self join with CASE ===
-- EXPECT: 6 rows
SELECT a.id, a.name,
  CASE WHEN b.id IS NOT NULL THEN 'paired' ELSE 'solo' END as status
FROM users a
LEFT JOIN users b ON a.id = b.id + 1
WHERE a.id <= 5;

-- === CASE: Triple self join ===
-- EXPECT: 3 rows
SELECT a.id as a_id, b.id as b_id, c.id as c_id
FROM users a
JOIN users b ON a.id < b.id AND a.id <= 2
JOIN users c ON b.id < c.id AND b.id <= 3
WHERE a.id = 1;
