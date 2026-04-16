-- === SKIP ===

-- === CTE (Common Table Expression) Test Suite ===

-- === CASE: Simple CTE ===
-- EXPECT: 5 rows
WITH active_users AS (
  SELECT id, name, email FROM users WHERE id <= 5
)
SELECT * FROM active_users;

-- === CASE: CTE with aggregate ===
-- EXPECT: 3 rows
WITH user_stats AS (
  SELECT user_id, COUNT(*) as order_count, SUM(total) as total_spent
  FROM orders GROUP BY user_id
)
SELECT u.name, us.order_count, us.total_spent
FROM users u
JOIN user_stats us ON u.id = us.user_id
WHERE u.id <= 3;

-- === CASE: CTE with JOIN ===
-- EXPECT: 6 rows
WITH recent_orders AS (
  SELECT * FROM orders WHERE total > 100
)
SELECT u.name, ro.order_id, ro.total
FROM users u
JOIN recent_orders ro ON u.id = ro.user_id
WHERE u.id <= 5;

-- === CASE: Multiple CTEs ===
-- EXPECT: 5 rows
WITH
  user_orders AS (
    SELECT user_id, COUNT(*) as cnt FROM orders GROUP BY user_id
  ),
  high_value_users AS (
    SELECT user_id FROM orders WHERE total > 200
  )
SELECT u.name, uo.cnt
FROM users u
JOIN user_orders uo ON u.id = uo.user_id
JOIN high_value_users hvu ON u.id = hvu.user_id;

-- === CASE: Recursive CTE ===
-- EXPECT: 5 rows
WITH RECURSIVE cnt(x) AS (
  SELECT 1
  UNION ALL
  SELECT x + 1 FROM cnt WHERE x < 5
)
SELECT x FROM cnt;

-- === CASE: CTE with UNION ===
-- EXPECT: 10 rows
WITH combined AS (
  SELECT id, name FROM users WHERE id <= 5
  UNION
  SELECT id, name FROM users WHERE id > 5 AND id <= 10
)
SELECT * FROM combined ORDER BY id;

-- === CASE: CTE with UPDATE ===
-- EXPECT: 3 rows affected
WITH to_update AS (
  SELECT id FROM users WHERE id <= 3
)
UPDATE users SET email = 'cte_updated@example.com'
WHERE id IN (SELECT id FROM to_update);

-- === CASE: CTE with INSERT ===
-- EXPECT: 3 rows affected
WITH new_users AS (
  SELECT 501 as id, 'CTE1' as name, 'cte1@example.com' as email
  UNION ALL
  SELECT 502, 'CTE2', 'cte2@example.com'
  UNION ALL
  SELECT 503, 'CTE3', 'cte3@example.com'
)
INSERT INTO users SELECT * FROM new_users;

-- === CASE: CTE with DELETE ===
-- EXPECT: 3 rows affected
WITH to_delete AS (
  SELECT id FROM users WHERE id >= 501
)
DELETE FROM users WHERE id IN (SELECT id FROM to_delete);

-- === CASE: Nested CTE ===
-- EXPECT: 3 rows
WITH outer_cte AS (
  WITH inner_cte AS (
    SELECT id, name FROM users WHERE id <= 5
  )
  SELECT * FROM inner_cte WHERE id > 2
)
SELECT * FROM outer_cte;

-- === CASE: CTE with DISTINCT ===
-- EXPECT: 3 rows
WITH unique_emails AS (
  SELECT DISTINCT email FROM users WHERE email IS NOT NULL
)
SELECT COUNT(*) as unique_email_count FROM unique_emails;

-- === CASE: CTE with ORDER BY ===
-- EXPECT: 5 rows
WITH ordered_users AS (
  SELECT id, name FROM users WHERE id <= 10 ORDER BY id DESC
)
SELECT * FROM ordered_users LIMIT 5;

-- === CASE: CTE with LIMIT ===
-- EXPECT: 3 rows
WITH top_users AS (
  SELECT user_id, SUM(total) as total FROM orders GROUP BY user_id ORDER BY total DESC LIMIT 3
)
SELECT u.name, tu.total
FROM users u
JOIN top_users tu ON u.id = tu.user_id;

-- === CASE: CTE with CASE ===
-- EXPECT: 5 rows
WITH categorized AS (
  SELECT id, name,
    CASE
      WHEN id <= 3 THEN 'low'
      WHEN id <= 7 THEN 'medium'
      ELSE 'high'
    END as category
  FROM users WHERE id <= 10
)
SELECT * FROM categorized WHERE category != 'low';
