-- SQLCorpus: DROP TABLE Variations
-- Tests for DROP TABLE patterns

-- === SETUP ===
CREATE TABLE t1 (id INTEGER PRIMARY KEY);
CREATE TABLE t2 (id INTEGER PRIMARY KEY);

-- === CASE: drop_table ===
DROP TABLE t1;
-- EXPECT: success

-- === CASE: drop_if_exists ===
DROP TABLE IF EXISTS t1;
-- EXPECT: success

-- === CASE: drop_multiple ===
DROP TABLE t2;
-- EXPECT: success

-- === CASE: create_after_drop ===
DROP TABLE t1;
CREATE TABLE t1 (id INTEGER PRIMARY KEY, name TEXT);
-- EXPECT: success

-- === CASE: drop_cascade ===
CREATE TABLE parent (id INTEGER PRIMARY KEY);
CREATE TABLE child (id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parent(id));
DROP TABLE parent CASCADE;
-- EXPECT: success