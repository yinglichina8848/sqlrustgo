-- SQLCorpus: Subquery Tests
-- Tests for subqueries in various contexts

-- === SETUP ===
CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price INTEGER, category_id INTEGER);
INSERT INTO products VALUES (1, 'Widget', 100, 1), (2, 'Gadget', 200, 1), (3, 'Tool', 150, 2), (4, 'Device', 300, 2), (5, 'Supply', 50, 3);

CREATE TABLE categories (id INTEGER PRIMARY KEY, name TEXT);
INSERT INTO categories VALUES (1, 'Electronics'), (2, 'Hardware'), (3, 'Supplies');

CREATE TABLE sales (id INTEGER PRIMARY KEY, product_id INTEGER, quantity INTEGER, sale_price INTEGER);
INSERT INTO sales VALUES (1, 1, 10, 100), (2, 2, 5, 200), (3, 3, 8, 150), (4, 1, 3, 100);

-- === CASE: scalar_subquery_in_select ===
SELECT name, (SELECT COUNT(*) FROM sales WHERE product_id = products.id) AS sale_count FROM products;
-- EXPECT: 5 rows

-- === CASE: scalar_subquery_in_where ===
SELECT name FROM products WHERE price > (SELECT AVG(price) FROM products);
-- EXPECT: 2 rows

-- === CASE: correlated_subquery ===
SELECT name FROM products p WHERE price > (SELECT AVG(price) FROM products WHERE category_id = p.category_id);
-- EXPECT: 2 rows

-- === CASE: in_subquery ===
SELECT name FROM products WHERE id IN (SELECT product_id FROM sales WHERE quantity > 5);
-- EXPECT: 2 rows

-- === CASE: not_in_subquery ===
SELECT name FROM products WHERE id NOT IN (SELECT product_id FROM sales WHERE quantity > 5);
-- EXPECT: 3 rows

-- === CASE: exists_subquery ===
SELECT name FROM products WHERE EXISTS (SELECT 1 FROM sales WHERE product_id = products.id AND quantity > 5);
-- EXPECT: 2 rows

-- === CASE: not_exists_subquery ===
SELECT name FROM products WHERE NOT EXISTS (SELECT 1 FROM sales WHERE product_id = products.id);
-- EXPECT: 3 rows

-- === CASE: subquery_in_from ===
SELECT sub.category, sub.avg_price FROM (SELECT category_id, AVG(price) AS avg_price FROM products GROUP BY category_id) AS sub;
-- EXPECT: 3 rows

-- === CASE: subquery_as_column ===
SELECT p.name, (SELECT c.name FROM categories c WHERE c.id = p.category_id) AS category_name FROM products p;
-- EXPECT: 5 rows

-- === CASE: comparison_in_subquery ===
SELECT name FROM products WHERE price = (SELECT MIN(price) FROM products);
-- EXPECT: 1 rows

-- === CASE: all_subquery ===
SELECT name FROM products WHERE price > ALL (SELECT price FROM products WHERE category_id = 1);
-- EXPECT: 2 rows

-- === CASE: any_subquery ===
SELECT name FROM products WHERE price > ANY (SELECT price FROM products WHERE category_id = 2);
-- EXPECT: 3 rows

-- === CASE: nested_subquery ===
SELECT name FROM products WHERE id IN (SELECT product_id FROM sales WHERE quantity IN (SELECT MAX(quantity) FROM sales));
-- EXPECT: 1 rows
