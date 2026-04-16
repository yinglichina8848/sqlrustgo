-- === SKIP ===

-- === Constraint Test Suite ===

-- === CASE: Add primary key ===
-- EXPECT: success
CREATE TABLE test_pk (id INTEGER, name TEXT, PRIMARY KEY (id));

-- === CASE: Add foreign key constraint ===
-- EXPECT: success
CREATE TABLE test_fk (order_id INTEGER, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id));

-- === CASE: Add unique constraint ===
-- EXPECT: success
CREATE TABLE test_unique (id INTEGER, email TEXT, UNIQUE (email));

-- === CASE: Add check constraint ===
-- EXPECT: success
CREATE TABLE test_check (id INTEGER, age INTEGER, CHECK (age >= 0));

-- === CASE: Add not null constraint ===
-- EXPECT: success
CREATE TABLE test_notnull (id INTEGER, name TEXT NOT NULL);

-- === CASE: Multiple constraints ===
-- EXPECT: success
CREATE TABLE test_multi (
  id INTEGER PRIMARY KEY,
  email TEXT UNIQUE NOT NULL,
  age INTEGER CHECK (age >= 18),
  user_id INTEGER REFERENCES users(id)
);

-- === CASE: Drop primary key ===
-- EXPECT: success
ALTER TABLE test_pk DROP PRIMARY KEY;

-- === CASE: Drop foreign key ===
-- EXPECT: success
ALTER TABLE test_fk DROP FOREIGN KEY fk_user;

-- === CASE: Add column with constraint ===
-- EXPECT: success
ALTER TABLE users ADD COLUMN status TEXT DEFAULT 'active' CHECK (status IN ('active', 'inactive'));

-- === CASE: Create table with composite key ===
-- EXPECT: success
CREATE TABLE test_composite (
  col1 INTEGER,
  col2 TEXT,
  PRIMARY KEY (col1, col2)
);

-- === CASE: Create table with named constraint ===
-- EXPECT: success
CREATE TABLE test_named (
  id INTEGER CONSTRAINT pk_test PRIMARY KEY,
  name TEXT CONSTRAINT uk_test UNIQUE
);

-- === CASE: Add default constraint ===
-- EXPECT: success
CREATE TABLE test_default (
  id INTEGER,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP,
  status TEXT DEFAULT 'pending'
);

-- === CASE: Constraint with ON DELETE ===
-- EXPECT: success
CREATE TABLE test_on_delete (
  id INTEGER PRIMARY KEY,
  user_id INTEGER REFERENCES users(id) ON DELETE CASCADE
);

-- === CASE: Constraint with ON UPDATE ===
-- EXPECT: success
CREATE TABLE test_on_update (
  id INTEGER PRIMARY KEY,
  user_id INTEGER REFERENCES users(id) ON UPDATE CASCADE
);

-- === CASE: Self referential constraint ===
-- EXPECT: success
CREATE TABLE test_self_ref (
  id INTEGER PRIMARY KEY,
  parent_id INTEGER REFERENCES test_self_ref(id)
);
