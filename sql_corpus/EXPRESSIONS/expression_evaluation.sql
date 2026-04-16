-- === SKIP ===

-- === Expression Evaluation Test Suite ===

-- === CASE: Arithmetic precedence ===
-- EXPECT: 1 row
SELECT 10 + 5 * 2 as result;

-- === CASE: Parentheses override precedence ===
-- EXPECT: 1 row
SELECT (10 + 5) * 2 as result;

-- === CASE: Integer vs real division ===
-- EXPECT: 1 row
SELECT 10 / 3 as int_div, 10.0 / 3 as real_div;

-- === CASE: Modulo with negative numbers ===
-- EXPECT: 1 row
SELECT -10 % 3 as neg_mod;

-- === CASE: String concatenation precedence ===
-- EXPECT: 1 row
SELECT 'Hello' || ' ' || 'World' as greeting;

-- === CASE: Boolean short-circuit AND ===
-- EXPECT: 1 row
SELECT 1 = 1 AND 2 = 2 as short_circuit;

-- === CASE: Boolean short-circuit OR ===
-- EXPECT: 1 row
SELECT 1 = 2 OR 2 = 2 as short_circuit_or;

-- === CASE: NULL in arithmetic ===
-- EXPECT: 1 row
SELECT 10 + NULL as null_result;

-- === CASE: NULL in string concat ===
-- EXPECT: 1 row
SELECT 'Hello' || NULL || 'World' as concat_null;

-- === CASE: Division by zero handling ===
-- EXPECT: 1 row
SELECT CASE WHEN 3 != 0 THEN 10 / 3 END as safe_division;

-- === CASE: Type coercion in arithmetic ===
-- EXPECT: 1 row
SELECT '10' + 5 as coerced_add;

-- === CASE: Comparison chain ===
-- EXPECT: 1 row
SELECT 5 < 10 < 15 as comparison_chain;

-- === CASE: CASE expression evaluation order ===
-- EXPECT: 1 row
SELECT CASE WHEN 1 = 1 THEN 'first' WHEN 1 = 2 THEN 'second' ELSE 'third' END as case_result;

-- === CASE: Function evaluation order ===
-- EXPECT: 1 row
SELECT UPPER(LOWER('HeLLo')) as nested_functions;

-- === CASE: Aggregate vs scalar in same query ===
-- EXPECT: 10 rows
SELECT id, name, (SELECT COUNT(*) FROM users) as total_users FROM users;
