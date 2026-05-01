-- === SKIP ===

-- SQLCorpus: ALTER TABLE Variations
-- Tests for various ALTER TABLE patterns

-- === SETUP ===
CREATE TABLE t1 (id INTEGER PRIMARY KEY, name TEXT);

-- === CASE: alter_add_column ===
ALTER TABLE t1 ADD COLUMN age INTEGER;
-- EXPECT: success

-- === CASE: alter_add_column_with_default ===
ALTER TABLE t1 ADD COLUMN status TEXT DEFAULT 'active';
-- EXPECT: success

-- === CASE: alter_rename_table ===
ALTER TABLE t1 RENAME TO t1_renamed;
-- EXPECT: success

-- === CASE: alter_add_primary_key ===
CREATE TABLE t2 (id INTEGER, name TEXT);
ALTER TABLE t2 ADD PRIMARY KEY (id);
-- EXPECT: success

-- === CASE: alter_add_unique ===
CREATE TABLE t3 (id INTEGER PRIMARY KEY, email TEXT);
ALTER TABLE t3 ADD UNIQUE (email);
-- EXPECT: success

-- === CASE: alter_add_check ===
CREATE TABLE t4 (id INTEGER PRIMARY KEY, age INTEGER);
ALTER TABLE t4 ADD CHECK (age >= 0);
-- EXPECT: success

-- === CASE: alter_add_foreign_key ===
CREATE TABLE t5 (id INTEGER PRIMARY KEY);
CREATE TABLE t6 (id INTEGER PRIMARY KEY, ref_id INTEGER);
ALTER TABLE t6 ADD FOREIGN KEY (ref_id) REFERENCES t5(id);
-- EXPECT: success