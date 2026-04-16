-- === Range and Interval Test Suite ===

-- === CASE: BETWEEN with integers ===
-- EXPECT: 5 rows
SELECT * FROM users WHERE id BETWEEN 3 AND 7;

-- === CASE: NOT BETWEEN ===
-- EXPECT: 5 rows
SELECT * FROM users WHERE id NOT BETWEEN 3 AND 7;

-- === CASE: BETWEEN with strings ===
-- EXPECT: 3 rows
SELECT * FROM users WHERE name BETWEEN 'Alice' AND 'Charlie';

-- === CASE: BETWEEN with dates ===
-- EXPECT: 5 rows
SELECT * FROM orders WHERE order_date BETWEEN '2024-01-01' AND '2024-12-31';

-- === CASE: BETWEEN with floating point ===
-- EXPECT: 3 rows
SELECT * FROM orders WHERE total BETWEEN 100.00 AND 300.00;

-- === CASE: IN with list ===
-- EXPECT: 3 rows
SELECT * FROM users WHERE id IN (1, 3, 5, 7, 9);

-- === CASE: NOT IN with list ===
-- EXPECT: 7 rows
SELECT * FROM users WHERE id NOT IN (1, 3, 5);

-- === CASE: IN with subquery ===
-- EXPECT: 5 rows
SELECT * FROM users WHERE id IN (SELECT user_id FROM orders WHERE total > 150);

-- === CASE: NOT IN with subquery ===
-- EXPECT: 5 rows
SELECT * FROM users WHERE id NOT IN (SELECT user_id FROM orders WHERE total < 100);

-- === CASE: IN with NULL handling ===
-- EXPECT: 0 rows
SELECT * FROM users WHERE id IN (NULL, 1, 2);

-- === CASE: EXISTS with subquery ===
-- EXPECT: 5 rows
SELECT * FROM users u WHERE EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id);

-- === CASE: NOT EXISTS with subquery ===
-- EXPECT: 5 rows
SELECT * FROM users u WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id AND o.total > 200);

-- === CASE: ALL with subquery ===
-- EXPECT: 2 rows
SELECT * FROM users WHERE id = ALL (SELECT id FROM users WHERE id <= 2);

-- === CASE: ANY with subquery ===
-- EXPECT: 8 rows
SELECT * FROM users WHERE id = ANY (SELECT id FROM users WHERE id > 2);

-- === CASE: Subquery with multiple columns ===
-- EXPECT: 3 rows
SELECT * FROM orders WHERE (user_id, total) IN (SELECT id, 200 FROM users WHERE id <= 3);
