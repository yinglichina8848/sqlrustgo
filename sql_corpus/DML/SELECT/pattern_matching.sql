-- === Pattern Matching Test Suite ===

-- === CASE: LIKE with % ===
-- EXPECT: 4 rows
SELECT * FROM users WHERE email LIKE '%@example.com';

-- === CASE: LIKE with _ ===
-- EXPECT: 2 rows
SELECT * FROM users WHERE name LIKE '_lice';

-- === CASE: LIKE case sensitivity ===
-- EXPECT: 3 rows
SELECT * FROM users WHERE name LIKE '%oh%';

-- === CASE: NOT LIKE ===
-- EXPECT: 6 rows
SELECT * FROM users WHERE name NOT LIKE 'A%';

-- === CASE: LIKE with multiple % ===
-- EXPECT: 3 rows
SELECT * FROM users WHERE email LIKE '%@%.com%';

-- === CASE: GLOB case sensitive ===
-- EXPECT: 2 rows
SELECT * FROM users WHERE name GLOB '*lice';

-- === CASE: GLOB with character class ===
-- EXPECT: 3 rows
SELECT * FROM users WHERE name GLOB '*[abc]*';

-- === CASE: NOT GLOB ===
-- EXPECT: 7 rows
SELECT * FROM users WHERE name NOT GLOB '*xyz*';

-- === CASE: GLOB with range ===
-- EXPECT: 2 rows
SELECT * FROM users WHERE name GLOB '*[A-Z]*';

-- === CASE: LIKE ESCAPE ===
-- EXPECT: 2 rows
SELECT * FROM users WHERE name LIKE '%20%' ESCAPE '~';

-- === CASE: REGEXP basic ===
-- EXPECT: 3 rows
SELECT * FROM users WHERE email REGEXP '.*@example\\.com';

-- === CASE: REGEXP with anchor ===
-- EXPECT: 2 rows
SELECT * FROM users WHERE name REGEXP '^Alice';

-- === CASE: REGEXP with alternation ===
-- EXPECT: 4 rows
SELECT * FROM users WHERE name REGEXP 'Alice|Bob|Charlie';

-- === CASE: NOT REGEXP ===
-- EXPECT: 6 rows
SELECT * FROM users WHERE name NOT REGEXP '^A';

-- === CASE: LIKE in CASE ===
-- EXPECT: 10 rows
SELECT id, name,
  CASE
    WHEN email LIKE '%@example.com' THEN 'example'
    WHEN email LIKE '%@test.com' THEN 'test'
    ELSE 'other'
  END as email_type
FROM users;

-- === CASE: LIKE with subquery ===
-- EXPECT: 3 rows
SELECT * FROM users
WHERE name LIKE (SELECT pattern FROM name_patterns WHERE id = 1);

-- === CASE: LIKE with OR ===
-- EXPECT: 4 rows
SELECT * FROM users WHERE name LIKE 'A%' OR name LIKE 'B%';

-- === CASE: LIKE with AND ===
-- EXPECT: 2 rows
SELECT * FROM users WHERE name LIKE '%e%' AND email LIKE '%@example.com';
