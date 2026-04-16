-- === SKIP ===

-- SQLCorpus: WHERE Clause Variations
-- Tests for various WHERE conditions

-- === SETUP ===
CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price INTEGER, category TEXT, stock INTEGER);
INSERT INTO products VALUES (1, 'Apple', 100, 'Fruit', 50);
INSERT INTO products VALUES (2, 'Banana', 50, 'Fruit', 100);
INSERT INTO products VALUES (3, 'Carrot', 30, 'Vegetable', 75);
INSERT INTO products VALUES (4, 'Chicken', 500, 'Meat', 20);
INSERT INTO products VALUES (5, 'Rice', 80, 'Grain', 200);
INSERT INTO products VALUES (6, 'Milk', 120, 'Dairy', 30);
INSERT INTO products VALUES (7, 'Cheese', 300, 'Dairy', 15);
INSERT INTO products VALUES (8, 'Tomato', 40, 'Vegetable', 80);

-- === CASE: where_equality ===
SELECT * FROM products WHERE category = 'Fruit';
-- EXPECT: 2 rows

-- === CASE: where_inequality ===
SELECT * FROM products WHERE price <> 100;
-- EXPECT: 7 rows

-- === CASE: where_less_than ===
SELECT * FROM products WHERE price < 100;
-- EXPECT: 4 rows

-- === CASE: where_less_than_or_equal ===
SELECT * FROM products WHERE price <= 100;
-- EXPECT: 5 rows

-- === CASE: where_greater_than ===
SELECT * FROM products WHERE price > 300;
-- EXPECT: 2 rows

-- === CASE: where_greater_than_or_equal ===
SELECT * FROM products WHERE price >= 300;
-- EXPECT: 3 rows

-- === CASE: where_and ===
SELECT * FROM products WHERE category = 'Fruit' AND price > 50;
-- EXPECT: 1 rows

-- === CASE: where_or ===
SELECT * FROM products WHERE category = 'Fruit' OR category = 'Vegetable';
-- EXPECT: 4 rows

-- === CASE: where_not ===
SELECT * FROM products WHERE NOT category = 'Fruit';
-- EXPECT: 6 rows

-- === CASE: where_and_or ===
SELECT * FROM products WHERE (category = 'Fruit' OR category = 'Vegetable') AND price > 40;
-- EXPECT: 3 rows

-- === CASE: where_in_list ===
SELECT * FROM products WHERE category IN ('Fruit', 'Dairy');
-- EXPECT: 3 rows

-- === CASE: where_not_in_list ===
SELECT * FROM products WHERE category NOT IN ('Fruit', 'Vegetable');
-- EXPECT: 3 rows

-- === CASE: where_between ===
SELECT * FROM products WHERE price BETWEEN 50 AND 150;
-- EXPECT: 4 rows

-- === CASE: where_not_between ===
SELECT * FROM products WHERE price NOT BETWEEN 50 AND 150;
-- EXPECT: 4 rows

-- === CASE: where_like_exact ===
SELECT * FROM products WHERE name LIKE 'Apple';
-- EXPECT: 1 rows

-- === CASE: where_like_prefix ===
SELECT * FROM products WHERE name LIKE 'C%';
-- EXPECT: 2 rows

-- === CASE: where_like_suffix ===
SELECT * FROM products WHERE name LIKE '%e';
-- EXPECT: 3 rows

-- === CASE: where_like_contains ===
SELECT * FROM products WHERE name LIKE '%an%';
-- EXPECT: 2 rows

-- === CASE: where_is_null ===
SELECT * FROM products WHERE stock IS NULL;
-- EXPECT: 0 rows

-- === CASE: where_is_not_null ===
SELECT * FROM products WHERE stock IS NOT NULL;
-- EXPECT: 8 rows

-- === CASE: where_multiple_conditions ===
SELECT * FROM products WHERE category = 'Dairy' AND stock > 10 AND price < 400;
-- EXPECT: 2 rows

-- === CASE: where_price_zero ===
SELECT * FROM products WHERE price = 0;
-- EXPECT: 0 rows

-- === CASE: where_stock_low ===
SELECT * FROM products WHERE stock < 20;
-- EXPECT: 2 rows

-- === CASE: where_exists ===
SELECT * FROM products WHERE EXISTS (SELECT 1 FROM products WHERE price > 400);
-- EXPECT: 8 rows

-- === CASE: where_not_exists ===
SELECT * FROM products WHERE NOT EXISTS (SELECT 1 FROM products WHERE price > 1000);
-- EXPECT: 8 rows

-- === CASE: where_subquery ===
SELECT * FROM products WHERE price > (SELECT AVG(price) FROM products);
-- EXPECT: 3 rows