-- === SKIP ===

-- SQLCorpus: CAST and Type Conversion
-- Tests for type casting and conversion

-- === SETUP ===
CREATE TABLE casts (id INTEGER PRIMARY KEY, s TEXT, i INTEGER, r REAL);
INSERT INTO casts VALUES (1, '123', 456, 7.89);

-- === CASE: cast_to_text ===
SELECT CAST(i AS TEXT) FROM casts;
-- EXPECT: 1 rows

-- === CASE: cast_to_integer ===
SELECT CAST(s AS INTEGER) FROM casts;
-- EXPECT: 1 rows

-- === CASE: cast_to_real ===
SELECT CAST(i AS REAL) FROM casts;
-- EXPECT: 1 rows

-- === CASE: cast_to_blob ===
SELECT CAST(s AS BLOB) FROM casts;
-- EXPECT: 1 rows

-- === CASE: typeof ===
SELECT TYPEOF(s) FROM casts;
-- EXPECT: 1 rows

-- === CASE: typeof_null ===
SELECT TYPEOF(NULL);
-- EXPECT: 1 rows

-- === CASE: typeof_int ===
SELECT TYPEOF(42);
-- EXPECT: 1 rows

-- === CASE: typeof_real ===
SELECT TYPEOF(3.14);
-- EXPECT: 1 rows

-- === CASE: cast_from_real_to_int ===
SELECT CAST(r AS INTEGER) FROM casts;
-- EXPECT: 1 rows

-- === CASE: cast_from_int_to_real ===
SELECT CAST(i AS REAL) FROM casts;
-- EXPECT: 1 rows

-- === CASE: implicit_cast ===
SELECT 1 + 2.5;
-- EXPECT: 1 rows

-- === CASE: implicit_text_concat ===
SELECT 'Hello' || ' ' || 'World';
-- EXPECT: 1 rows