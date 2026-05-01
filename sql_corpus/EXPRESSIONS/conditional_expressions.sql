-- === SKIP ===

-- === Conditional Expressions Test Suite ===

-- === CASE: Simple CASE ===
-- EXPECT: 10 rows
SELECT id, name,
  CASE id
    WHEN 1 THEN 'One'
    WHEN 2 THEN 'Two'
    WHEN 3 THEN 'Three'
    ELSE 'Other'
  END as number_name
FROM users;

-- === CASE: CASE with conditions ===
-- EXPECT: 10 rows
SELECT id, name,
  CASE
    WHEN id < 3 THEN 'Low'
    WHEN id < 7 THEN 'Medium'
    ELSE 'High'
  END as category
FROM users;

-- === CASE: CASE with aggregate ===
-- EXPECT: 1 row
SELECT
  CASE WHEN COUNT(*) > 5 THEN 'Many' ELSE 'Few' END as user_count
FROM users;

-- === CASE: CASE with NULL ===
-- EXPECT: 10 rows
SELECT id, name,
  CASE email
    WHEN NULL THEN 'No Email'
    ELSE email
  END as contact
FROM users;

-- === CASE: CASE in WHERE ===
-- EXPECT: 5 rows
SELECT * FROM users
WHERE CASE WHEN id > 5 THEN 1 ELSE 0 END = 1;

-- === CASE: CASE in ORDER BY ===
-- EXPECT: 10 rows
SELECT * FROM users
ORDER BY CASE WHEN id <= 5 THEN 0 ELSE 1 END, id;

-- === CASE: Nested CASE ===
-- EXPECT: 10 rows
SELECT id, name,
  CASE
    WHEN id <= 3 THEN
      CASE
        WHEN id = 1 THEN 'A'
        ELSE 'B'
      END
    ELSE 'C'
  END as sub_category
FROM users;

-- === CASE: CASE with LIKE ===
-- EXPECT: 5 rows
SELECT id, name,
  CASE
    WHEN name LIKE 'A%' THEN 'Starts with A'
    WHEN name LIKE 'B%' THEN 'Starts with B'
    ELSE 'Other'
  END as name_category
FROM users WHERE id <= 7;

-- === CASE: CASE with BETWEEN ===
-- EXPECT: 10 rows
SELECT id, name,
  CASE
    WHEN id BETWEEN 1 AND 3 THEN 'Group 1'
    WHEN id BETWEEN 4 AND 6 THEN 'Group 2'
    ELSE 'Group 3'
  END as groups
FROM users;

-- === CASE: CASE with multiple conditions ===
-- EXPECT: 5 rows
SELECT id, name, email,
  CASE
    WHEN id > 3 AND email LIKE '%@example.com' THEN 'VIP'
    WHEN id > 5 THEN 'Regular'
    ELSE 'Basic'
  END as status
FROM users WHERE id <= 8;

-- === CASE: CASE with subquery ===
-- EXPECT: 5 rows
SELECT id, name,
  CASE
    WHEN (SELECT COUNT(*) FROM orders WHERE user_id = users.id) > 2 THEN 'Active'
    ELSE 'Inactive'
  END as order_status
FROM users WHERE id <= 5;

-- === CASE: CASE in UPDATE ===
-- EXPECT: 5 rows affected
UPDATE users SET
  status = CASE
    WHEN id <= 3 THEN 'low'
    WHEN id <= 7 THEN 'medium'
    ELSE 'high'
  END
WHERE id <= 10;

-- === CASE: CASE with arithmetic ===
-- EXPECT: 1 row
SELECT
  CASE
    WHEN 10 > 5 THEN 10 * 2
    WHEN 10 > 3 THEN 10 / 2
    ELSE 10 + 2
  END as result;

-- === CASE: CASE with DISTINCT ===
-- EXPECT: 3 rows
SELECT DISTINCT
  CASE WHEN id <= 3 THEN 'Group A' ELSE 'Group B' END as grp
FROM users;

-- === CASE: CASE with GROUP BY ===
-- EXPECT: 2 rows
SELECT
  CASE WHEN id <= 5 THEN 'Group1' ELSE 'Group2' END as grp,
  COUNT(*) as cnt
FROM users
GROUP BY CASE WHEN id <= 5 THEN 'Group1' ELSE 'Group2' END;
