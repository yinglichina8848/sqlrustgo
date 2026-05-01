-- === SKIP ===

-- SQLCorpus: CREATE TABLE Variations
-- Tests for various CREATE TABLE patterns

-- === CASE: create_simple ===
CREATE TABLE t1 (id INTEGER PRIMARY KEY, name TEXT);
-- EXPECT: success

-- === CASE: create_with_all_types ===
CREATE TABLE t2 (i INTEGER, f REAL, s TEXT, b BLOB);
-- EXPECT: success

-- === CASE: create_with_not_null ===
CREATE TABLE t3 (id INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL);
-- EXPECT: success

-- === CASE: create_with_default ===
CREATE TABLE t4 (id INTEGER PRIMARY KEY, value INTEGER DEFAULT 0, name TEXT DEFAULT 'unknown');
-- EXPECT: success

-- === CASE: create_with_unique ===
CREATE TABLE t5 (id INTEGER PRIMARY KEY, email TEXT UNIQUE);
-- EXPECT: success

-- === CASE: create_with_check ===
CREATE TABLE t6 (id INTEGER PRIMARY KEY, age INTEGER CHECK (age >= 0));
-- EXPECT: success

-- === CASE: create_with_multiple_indexes ===
CREATE TABLE t7 (id INTEGER PRIMARY KEY, a INTEGER, b INTEGER, UNIQUE(a, b));
-- EXPECT: success

-- === CASE: create_if_not_exists ===
CREATE TABLE IF NOT EXISTS t1 (id INTEGER PRIMARY KEY);
-- EXPECT: success

-- === CASE: create_with_autoincrement ===
CREATE TABLE t8 (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT);
-- EXPECT: success

-- === CASE: create_temp ===
CREATE TEMPORARY TABLE t9 (id INTEGER PRIMARY KEY, name TEXT);
-- EXPECT: success

-- === CASE: create_with_primary_key ===
CREATE TABLE t10 (id INTEGER, name TEXT, PRIMARY KEY (id));
-- EXPECT: success

-- === CASE: create_with_foreign_key ===
CREATE TABLE t11 (id INTEGER PRIMARY KEY, ref_id INTEGER REFERENCES t1(id));
-- EXPECT: success

-- === CASE: create_with_without_rowid ===
CREATE TABLE t12 (id INTEGER PRIMARY KEY, name TEXT) WITHOUT ROWID;
-- EXPECT: success