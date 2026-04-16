-- === Common Table Expression Advanced Test Suite ===

-- === CASE: Recursive CTE for factorial ===
-- EXPECT: 6 rows
WITH RECURSIVE factorial(n, fact) AS (
  SELECT 0, 1
  UNION ALL
  SELECT n + 1, (n + 1) * fact FROM factorial WHERE n < 5
)
SELECT * FROM factorial;

-- === CASE: Recursive CTE for Fibonacci ===
-- EXPECT: 10 rows
WITH RECURSIVE fib(a, b) AS (
  SELECT 0, 1
  UNION ALL
  SELECT b, a + b FROM fib WHERE b < 50
)
SELECT a FROM fib;

-- === CASE: Recursive CTE with depth ===
-- EXPECT: 5 rows
WITH RECURSIVE counter(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM counter WHERE n < 5
)
SELECT n, 'Level ' || n as level FROM counter;

-- === CASE: CTE with ORDER BY in definition ===
-- EXPECT: 5 rows
WITH ordered_users AS (
  SELECT id, name FROM users WHERE id <= 10 ORDER BY id DESC
)
SELECT * FROM ordered_users;

-- === CASE: CTE with LIMIT in definition ===
-- EXPECT: 3 rows
WITH limited_users AS (
  SELECT id, name FROM users ORDER BY id LIMIT 3
)
SELECT * FROM limited_users;

-- === CASE: Multiple CTEs with dependencies ===
-- EXPECT: 3 rows
WITH
  active_orders AS (
    SELECT user_id, COUNT(*) as order_count FROM orders WHERE total > 100 GROUP BY user_id
  ),
  high_value_customers AS (
    SELECT user_id, SUM(total) as total_spent FROM orders GROUP BY user_id HAVING SUM(total) > 500
  )
SELECT u.name, ao.order_count, hvc.total_spent
FROM users u
JOIN active_orders ao ON u.id = ao.user_id
JOIN high_value_customers hvc ON u.id = hvc.user_id;

-- === CASE: CTE with DISTINCT ===
-- EXPECT: 3 rows
WITH unique_emails AS (
  SELECT DISTINCT email FROM users WHERE email IS NOT NULL
)
SELECT COUNT(*) as cnt FROM unique_emails;

-- === CASE: CTE with self reference ===
-- EXPECT: 5 rows
WITH RECURSIVE org_chart AS (
  SELECT id, name, manager_id, 1 as level FROM employees WHERE manager_id IS NULL
  UNION ALL
  SELECT e.id, e.name, e.manager_id, oc.level + 1
  FROM employees e
  JOIN org_chart oc ON e.manager_id = oc.id
)
SELECT * FROM org_chart WHERE level <= 2;

-- === CASE: CTE with UNION ALL ===
-- EXPECT: 10 rows
WITH combined AS (
  SELECT id, name, 'A' as source FROM users WHERE id <= 5
  UNION ALL
  SELECT id, name, 'B' as source FROM users WHERE id > 5 AND id <= 10
)
SELECT * FROM combined ORDER BY id;

-- === CASE: CTE with HAVING ===
-- EXPECT: 3 rows
WITH user_stats AS (
  SELECT user_id, COUNT(*) as cnt, SUM(total) as total
  FROM orders GROUP BY user_id HAVING COUNT(*) > 2
)
SELECT u.name, us.cnt, us.total
FROM users u
JOIN user_stats us ON u.id = us.user_id;

-- === CASE: CTE with window function ===
-- EXPECT: 5 rows
WITH ranked_users AS (
  SELECT id, name, ROW_NUMBER() OVER (ORDER BY id) as rn
  FROM users WHERE id <= 10
)
SELECT * FROM ranked_users WHERE rn <= 5;

-- === CASE: CTE with JOIN between CTEs ===
-- EXPECT: 4 rows
WITH
  users_cte AS (SELECT id, name FROM users WHERE id <= 5),
  orders_cte AS (SELECT user_id, COUNT(*) as cnt FROM orders GROUP BY user_id)
SELECT u.id, u.name, o.cnt
FROM users_cte u
LEFT JOIN orders_cte o ON u.id = o.user_id;

-- === CASE: Nested CTEs ===
-- EXPECT: 3 rows
WITH outer_cte AS (
  WITH inner_cte AS (
    SELECT id, name FROM users WHERE id <= 5
  )
  SELECT * FROM inner_cte WHERE id > 2
)
SELECT * FROM outer_cte;
