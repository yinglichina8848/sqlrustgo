-- === JSON Functions Test Suite ===

-- === CASE: JSON function ===
-- EXPECT: 1 row
SELECT JSON('{"key": "value"}') as json_val;

-- === CASE: JSON_ARRAY ===
-- EXPECT: 1 row
SELECT JSON_ARRAY(1, 2, 3, 'text', NULL) as json_arr;

-- === CASE: JSON_ARRAY_LENGTH ===
-- EXPECT: 1 row
SELECT JSON_ARRAY_LENGTH('[1, 2, 3, 4]') as arr_len;

-- === CASE: JSON_EXTRACT ===
-- EXPECT: 1 row
SELECT JSON_EXTRACT('{"name": "John", "age": 30}', '$.name') as name;

-- === CASE: JSON_INSERT ===
-- EXPECT: 1 row
SELECT JSON_INSERT('{"name": "John"}', '$.age', 30) as json_val;

-- === CASE: JSON_REPLACE ===
-- EXPECT: 1 row
SELECT JSON_REPLACE('{"name": "John", "age": 30}', '$.age', 31) as json_val;

-- === CASE: JSON_REMOVE ===
-- EXPECT: 1 row
SELECT JSON_REMOVE('{"name": "John", "age": 30}', '$.age') as json_val;

-- === CASE: JSON_SET ===
-- EXPECT: 1 row
SELECT JSON_SET('{"name": "John"}', '$.age', 30) as json_val;

-- === CASE: JSON_TYPE ===
-- EXPECT: 1 row
SELECT JSON_TYPE('{"name": "John"}') as json_type;

-- === CASE: JSON_VALID ===
-- EXPECT: 1 row
SELECT JSON_VALID('{"valid": true}') as is_valid;

-- === CASE: JSON_QUOTE ===
-- EXPECT: 1 row
SELECT JSON_QUOTE('text') as quoted;

-- === CASE: JSON_OBJECT ===
-- EXPECT: 1 row
SELECT JSON_OBJECT('name', 'John', 'age', 30) as json_obj;

-- === CASE: JSON_GROUP_ARRAY ===
-- EXPECT: 1 row
SELECT JSON_GROUP_ARRAY(name) as names FROM users WHERE id <= 5;

-- === CASE: JSON_GROUP_OBJECT ===
-- EXPECT: 1 row
SELECT JSON_GROUP_OBJECT(name, email) as name_emails FROM users WHERE id <= 3;

-- === CASE: JSON_EACH ===
-- EXPECT: 4 rows
SELECT * FROM JSON_EACH('{"a": 1, "b": 2, "c": 3}');

-- === CASE: JSON_TREE ===
-- EXPECT: 5 rows
SELECT * FROM JSON_TREE('{"name": "John", "address": {"city": "NYC"}}');

-- === CASE: JSON_DEPTH ===
-- EXPECT: 1 row
SELECT JSON_DEPTH('{"a": {"b": {"c": 1}}}') as depth;

-- === CASE: JSON_LENGTH ===
-- EXPECT: 1 row
SELECT JSON_LENGTH('{"a": 1, "b": 2}') as len;
