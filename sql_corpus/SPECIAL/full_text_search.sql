-- === SKIP ===

-- === Full Text Search Test Suite ===

-- === CASE: CREATE VIRTUAL TABLE with FTS ===
-- EXPECT: success
CREATE VIRTUAL TABLE articles_fts USING fts5(title, content);

-- === CASE: INSERT into FTS table ===
-- EXPECT: success
INSERT INTO articles_fts(title, content) VALUES ('SQL Basics', 'Introduction to SQL queries and commands');

-- === CASE: Simple MATCH query ===
-- EXPECT: 2 rows
SELECT * FROM articles_fts WHERE articles_fts MATCH 'SQL';

-- === CASE: MATCH with multiple terms ===
-- EXPECT: 1 row
SELECT * FROM articles_fts WHERE articles_fts MATCH 'SQL queries';

-- === CASE: MATCH with OR ===
-- EXPECT: 2 rows
SELECT * FROM articles_fts WHERE articles_fts MATCH 'SQL OR Python';

-- === CASE: MATCH with AND ===
-- EXPECT: 1 row
SELECT * FROM articles_fts WHERE articles_fts MATCH 'SQL AND Introduction';

-- === CASE: MATCH with NOT ===
-- EXPECT: 1 row
SELECT * FROM articles_fts WHERE articles_fts MATCH 'SQL NOT Python';

-- === CASE: MATCH with phrase ===
-- EXPECT: 1 row
SELECT * FROM articles_fts WHERE articles_fts MATCH '"Introduction to"';

-- === CASE: MATCH with prefix ===
-- EXPECT: 2 rows
SELECT * FROM articles_fts WHERE articles_fts MATCH 'SQ*';

-- === CASE: DROP FTS table ===
-- EXPECT: success
DROP TABLE articles_fts;

-- === CASE: FTS5 with tokenizer ===
-- EXPECT: success
CREATE VIRTUAL TABLE docs_fts USING fts5(content, tokenize='porter');

-- === CASE: FTS5 with column size index ===
-- EXPECT: success
CREATE VIRTUAL TABLE test_fts USING fts5(content, content='test_table', content_rowid='id');

-- === CASE: MATCH with column specification ===
-- EXPECT: 1 row
SELECT * FROM articles_fts WHERE title MATCH 'SQL';

-- === CASE: MATCH with ORDER BY rank ===
-- EXPECT: 2 rows
SELECT * FROM articles_fts WHERE articles_fts MATCH 'SQL' ORDER BY rank;

-- === CASE: FTS snippet ===
-- EXPECT: 2 rows
SELECT snippet(articles_fts, 0, '[', ']', '...', 32) FROM articles_fts WHERE articles_fts MATCH 'SQL';

-- === CASE: FTS5 with Unicode61 tokenizer ===
-- EXPECT: success
CREATE VIRTUAL TABLE unicode_fts USING fts5(content, tokenize='unicode61');
