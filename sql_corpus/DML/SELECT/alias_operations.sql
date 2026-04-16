-- === Table and Column Aliases Test Suite ===

-- === CASE: Simple column alias ===
-- EXPECT: 10 rows
SELECT id, name AS user_name FROM users;

-- === CASE: Column alias with spaces ===
-- EXPECT: 10 rows
SELECT id, name AS "User Name" FROM users;

-- === CASE: Table alias ===
-- EXPECT: 10 rows
SELECT u.id, u.name FROM users AS u;

-- === CASE: Table alias with join ===
-- EXPECT: 5 rows
SELECT u.id, u.name, o.order_id
FROM users AS u
JOIN orders AS o ON u.id = o.user_id;

-- === CASE: Multiple table aliases ===
-- EXPECT: 5 rows
SELECT a.id, a.name, b.id, b.name
FROM users AS a
JOIN users AS b ON a.id < b.id
WHERE a.id <= 3;

-- === CASE: Alias in aggregate ===
-- EXPECT: 5 rows
SELECT user_id, SUM(total) AS total_spent FROM orders GROUP BY user_id;

-- === CASE: Alias in GROUP BY ===
-- EXPECT: 5 rows
SELECT user_id AS uid, COUNT(*) AS cnt FROM orders GROUP BY uid;

-- === CASE: Alias in ORDER BY ===
-- EXPECT: 10 rows
SELECT id, name FROM users ORDER BY name AS "User Name";

-- === CASE: Alias in HAVING ===
-- EXPECT: 3 rows
SELECT user_id, COUNT(*) AS cnt FROM orders GROUP BY user_id HAVING cnt > 2;

-- === CASE: Nested alias ===
-- EXPECT: 5 rows
SELECT id, name AS n FROM users WHERE n LIKE 'A%';

-- === CASE: Alias with expression ===
-- EXPECT: 10 rows
SELECT id, name, LENGTH(name) AS name_length FROM users;

-- === CASE: Alias replacing column ===
-- EXPECT: 10 rows
SELECT id, name AS username, email AS contact FROM users;

-- === CASE: Table alias in subquery ===
-- EXPECT: 5 rows
SELECT outer_u.id, outer_u.name
FROM users AS outer_u
WHERE outer_u.id IN (
  SELECT inner_u.id FROM users AS inner_u WHERE inner_u.id <= 5
);

-- === CASE: Multiple aliases same table ===
-- EXPECT: 6 rows
SELECT a.name AS person_a, b.name AS person_b
FROM users AS a
JOIN users AS b ON a.id < b.id
WHERE a.id <= 3;

-- === CASE: Alias with CONCAT ===
-- EXPECT: 10 rows
SELECT id, name || ' (' || email || ')' AS contact_info FROM users;
