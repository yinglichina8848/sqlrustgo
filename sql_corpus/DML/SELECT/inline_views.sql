-- === SKIP ===

-- === Inline View Test Suite ===

-- === CASE: Simple inline view ===
-- EXPECT: 5 rows
SELECT * FROM (SELECT id, name FROM users WHERE id <= 5);

-- === CASE: Inline view with alias ===
-- EXPECT: 5 rows
SELECT t.id, t.name FROM (SELECT id, name FROM users WHERE id <= 5) t;

-- === CASE: Inline view with aggregate ===
-- EXPECT: 3 rows
SELECT t.user_count, t.total FROM (SELECT COUNT(*) as user_count, SUM(total) as total FROM users u JOIN orders o ON u.id = o.user_id GROUP BY u.id) t;

-- === CASE: Inline view with JOIN ===
-- EXPECT: 8 rows
SELECT u.id, u.name, t.order_count FROM users u
JOIN (SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id) t ON u.id = t.user_id;

-- === CASE: Inline view with WHERE ===
-- EXPECT: 3 rows
SELECT * FROM (SELECT * FROM orders WHERE total > 150) t WHERE t.user_id <= 5;

-- === CASE: Inline view with ORDER BY ===
-- EXPECT: 5 rows
SELECT * FROM (SELECT * FROM users ORDER BY id DESC LIMIT 10) t ORDER BY id;

-- === CASE: Inline view with GROUP BY ===
-- EXPECT: 4 rows
SELECT t.category, COUNT(*) as cnt FROM (SELECT id, CASE WHEN id <= 3 THEN 'low' WHEN id <= 7 THEN 'mid' ELSE 'high' END as category FROM users) t GROUP BY t.category;

-- === CASE: Multiple inline views ===
-- EXPECT: 3 rows
SELECT a.id, b.order_count FROM (SELECT id FROM users WHERE id <= 5) a
JOIN (SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id) b ON a.id = b.user_id;

-- === CASE: Inline view with DISTINCT ===
-- EXPECT: 3 rows
SELECT DISTINCT t.email_domain FROM (SELECT email, SUBSTR(email, INSTR(email, '@') + 1) as email_domain FROM users WHERE email IS NOT NULL) t;

-- === CASE: Inline view with LIMIT ===
-- EXPECT: 3 rows
SELECT * FROM (SELECT * FROM orders ORDER BY total DESC LIMIT 5) t WHERE total > 100;

-- === CASE: Inline view in UPDATE ===
-- EXPECT: 5 rows affected
UPDATE users SET email = 'inline_' || email WHERE id IN (SELECT id FROM (SELECT id FROM users WHERE id <= 5));

-- === CASE: Inline view in DELETE ===
-- EXPECT: 2 rows affected
DELETE FROM users WHERE id IN (SELECT id FROM (SELECT id FROM users WHERE id > 500));

-- === CASE: Inline view with UNION ===
-- EXPECT: 10 rows
SELECT * FROM (SELECT id, name FROM users WHERE id <= 5 UNION SELECT id, name FROM users WHERE id > 5 AND id <= 10) t ORDER BY id;

-- === CASE: Nested inline view ===
-- EXPECT: 3 rows
SELECT * FROM (SELECT * FROM (SELECT * FROM users WHERE id <= 5) t1 WHERE id > 2) t2;

-- === CASE: Inline view with NULL handling ===
-- EXPECT: 8 rows
SELECT t.id, COALESCE(t.total_spent, 0) as total FROM (SELECT u.id, o.total as total_spent FROM users u LEFT JOIN orders o ON u.id = o.user_id) t;

-- === CASE: Inline view with subquery in SELECT ===
-- EXPECT: 5 rows
SELECT u.id, u.name, (SELECT COUNT(*) FROM (SELECT * FROM orders WHERE user_id = u.id)) as order_count FROM users u WHERE u.id <= 5;
