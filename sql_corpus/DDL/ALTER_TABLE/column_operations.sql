-- === Column Operations Test Suite ===

-- === CASE: Add column ===
-- EXPECT: success
ALTER TABLE users ADD COLUMN phone TEXT;

-- === CASE: Add column with DEFAULT ===
-- EXPECT: success
ALTER TABLE users ADD COLUMN status TEXT DEFAULT 'active';

-- === CASE: Add column with NOT NULL ===
-- EXPECT: success
ALTER TABLE users ADD COLUMN verified INTEGER DEFAULT 0 NOT NULL;

-- === CASE: Drop column ===
-- EXPECT: success
ALTER TABLE users DROP COLUMN phone;

-- === CASE: Rename column ===
-- EXPECT: success
ALTER TABLE users RENAME COLUMN email TO contact_email;

-- === CASE: Rename table ===
-- EXPECT: success
ALTER TABLE users RENAME TO app_users;

-- === CASE: Rename table back ===
-- EXPECT: success
ALTER TABLE app_users RENAME TO users;

-- === CASE: Add multiple columns ===
-- EXPECT: success
ALTER TABLE users
  ADD COLUMN address TEXT,
  ADD COLUMN city TEXT DEFAULT 'Unknown';

-- === CASE: Drop column if exists ===
-- EXPECT: success
ALTER TABLE users DROP COLUMN IF EXISTS address;

-- === CASE: Add column with CHECK ===
-- EXPECT: success
ALTER TABLE users ADD COLUMN level INTEGER DEFAULT 1 CHECK (level >= 1 AND level <= 10);

-- === CASE: Add column with UNIQUE ===
-- EXPECT: success
ALTER TABLE users ADD COLUMN nickname TEXT UNIQUE;

-- === CASE: Alter column set DEFAULT ===
-- EXPECT: success
ALTER TABLE users ALTER COLUMN status SET DEFAULT 'inactive';

-- === CASE: Alter column drop DEFAULT ===
-- EXPECT: success
ALTER TABLE users ALTER COLUMN status DROP DEFAULT;

-- === CASE: Alter column set NOT NULL ===
-- EXPECT: success
ALTER TABLE users ALTER COLUMN nickname SET NOT NULL;

-- === CASE: Alter column drop NOT NULL ===
-- EXPECT: success
ALTER TABLE users ALTER COLUMN nickname DROP NOT NULL;

-- === CASE: Add foreign key column ===
-- EXPECT: success
ALTER TABLE orders ADD COLUMN reference_id INTEGER REFERENCES users(id);
