-- SUBQUERY Test Cases
-- Compatibility: MySQL 5.7+

-- ============================================
-- 1. Subquery in WHERE (scalar)
-- ============================================

SELECT name, price
FROM products
WHERE price > (SELECT AVG(price) FROM products);

SELECT name, age
FROM users
WHERE age < (SELECT AVG(age) FROM users);

SELECT name, total
FROM orders
WHERE total > (SELECT MAX(total) FROM orders WHERE status = 'pending');

-- ============================================
-- 2. Subquery with IN
-- ============================================

SELECT name
FROM categories
WHERE id IN (SELECT category_id FROM products WHERE price > 100);

SELECT name
FROM users
WHERE id IN (SELECT user_id FROM orders WHERE total > 500);

SELECT name
FROM products
WHERE id IN (SELECT product_id FROM order_items WHERE quantity > 10);

-- ============================================
-- 3. Subquery with NOT IN
-- ============================================

SELECT name
FROM categories
WHERE id NOT IN (SELECT category_id FROM products);

SELECT name
FROM users
WHERE id NOT IN (SELECT user_id FROM orders WHERE status = 'completed');

SELECT name
FROM products
WHERE category_id NOT IN (SELECT id FROM categories WHERE active = TRUE);

-- ============================================
-- 4. Subquery with EXISTS
-- ============================================

SELECT name
FROM users
WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id);

SELECT name
FROM products
WHERE EXISTS (SELECT 1 FROM order_items WHERE order_items.product_id = products.id AND quantity > 5);

SELECT name
FROM categories
WHERE NOT EXISTS (SELECT 1 FROM products WHERE products.category_id = categories.id);

-- ============================================
-- 5. Subquery with ANY/SOME
-- ============================================

SELECT name
FROM products
WHERE price > ANY (SELECT price FROM products WHERE category_id = 1);

SELECT name
FROM users
WHERE age = ANY (SELECT age FROM users WHERE status = 'active');

SELECT name
FROM orders
WHERE total > SOME (SELECT total FROM orders WHERE status = 'pending');

-- ============================================
-- 6. Subquery with ALL
-- ============================================

SELECT name
FROM products
WHERE price > ALL (SELECT price FROM products WHERE category_id = 2);

SELECT name
FROM users
WHERE age > ALL (SELECT age FROM users WHERE status = 'inactive');

SELECT name
FROM orders
WHERE total < ALL (SELECT total FROM orders WHERE status = 'refunded');

-- ============================================
-- 7. Correlated subquery in SELECT
-- ============================================

SELECT name,
    (SELECT COUNT(*) FROM orders WHERE orders.user_id = users.id) as order_count
FROM users;

SELECT name, price,
    (SELECT AVG(price) FROM products WHERE category_id = products.category_id) as category_avg
FROM products;

SELECT name,
    (SELECT MAX(created_at) FROM orders WHERE orders.user_id = users.id) as last_order_date
FROM users;

-- ============================================
-- 8. Correlated subquery in WHERE
-- ============================================

SELECT name
FROM products p
WHERE price > (SELECT AVG(price) FROM products WHERE category_id = p.category_id);

SELECT name
FROM users u
WHERE (SELECT COUNT(*) FROM orders WHERE user_id = u.id) > 5;

SELECT name
FROM orders o
WHERE total > (SELECT AVG(total) FROM orders WHERE user_id = o.user_id);

-- ============================================
-- 9. Subquery in FROM (derived table)
-- ============================================

SELECT *
FROM (SELECT name, price FROM products WHERE price > 50) as expensive_products;

SELECT category, AVG(price) as avg_price
FROM (SELECT p.name, p.price, c.name as category
      FROM products p
      JOIN categories c ON p.category_id = c.id) as product_stats
GROUP BY category;

SELECT *
FROM (SELECT user_id, COUNT(*) as cnt FROM orders GROUP BY user_id) as order_counts
WHERE cnt > 3;

-- ============================================
-- 10. Subquery in HAVING
-- ============================================

SELECT user_id,
    COUNT(*) as order_count,
    SUM(total) as total_spent
FROM orders
GROUP BY user_id
HAVING COUNT(*) > (SELECT AVG(cnt) FROM (SELECT COUNT(*) as cnt FROM orders GROUP BY user_id) as avg_count);

SELECT category_id,
    COUNT(*) as product_count,
    AVG(price) as avg_price
FROM products
GROUP BY category_id
HAVING AVG(price) > (SELECT AVG(price) FROM products);

-- ============================================
-- 11. Subquery with JOIN
-- ============================================

SELECT u.name, order_counts.cnt
FROM users u
JOIN (SELECT user_id, COUNT(*) as cnt FROM orders GROUP BY user_id) order_counts
ON u.id = order_counts.user_id;

SELECT p.name, p.price, cat_avg.avg_price
FROM products p
JOIN (SELECT category_id, AVG(price) as avg_price FROM products GROUP BY category_id) cat_avg
ON p.category_id = cat_avg.category_id;

-- ============================================
-- 12. Nested subqueries (3+ levels)
-- ============================================

SELECT name
FROM users
WHERE id IN (
    SELECT user_id
    FROM orders
    WHERE id IN (
        SELECT order_id
        FROM order_items
        WHERE product_id IN (
            SELECT id FROM products WHERE price > 100
        )
    )
);

SELECT name
FROM products
WHERE category_id IN (
    SELECT id
    FROM categories
    WHERE id IN (
        SELECT category_id
        FROM products
        GROUP BY category_id
        HAVING COUNT(*) > 5
    )
);

-- ============================================
-- 13. Subquery with LIMIT
-- ============================================

SELECT name
FROM products
WHERE id IN (SELECT product_id FROM order_items LIMIT 10);

SELECT name
FROM users
WHERE id IN (SELECT user_id FROM orders ORDER BY created_at DESC LIMIT 5);

-- ============================================
-- 14. Subquery with ORDER BY
-- ============================================

SELECT name
FROM products
WHERE id IN (
    SELECT product_id
    FROM order_items
    ORDER BY quantity DESC
);

SELECT name
FROM users
WHERE id IN (
    SELECT user_id
    FROM orders
    ORDER BY total DESC
    LIMIT 10
);

-- ============================================
-- 15. Subquery with DISTINCT
-- ============================================

SELECT name
FROM users
WHERE id IN (SELECT DISTINCT user_id FROM orders);

SELECT name
FROM categories
WHERE id IN (SELECT DISTINCT category_id FROM products WHERE price > 50);

-- ============================================
-- 16. Subquery with UNION
-- ============================================

SELECT name
FROM products
WHERE id IN (
    SELECT product_id FROM order_items WHERE quantity > 10
    UNION
    SELECT id FROM products WHERE price > 200
);

-- ============================================
-- 17. Row subquery
-- ============================================

SELECT name, price, category_id
FROM products
WHERE (price, category_id) = (SELECT MAX(price), category_id FROM products GROUP BY category_id);

SELECT name, age, department_id
FROM employees
WHERE (age, department_id) = (SELECT MIN(age), department_id FROM employees GROUP BY department_id);

-- ============================================
-- 18. Subquery with aggregate functions
-- ============================================

SELECT name,
    (SELECT SUM(quantity * price) FROM order_items WHERE order_id = orders.id) as order_value
FROM orders;

SELECT name,
    (SELECT COUNT(*) FROM order_items WHERE product_id = products.id) as times_ordered
FROM products;

SELECT name,
    (SELECT AVG(total) FROM orders WHERE user_id = users.id AND status = 'completed') as avg_completed
FROM users;

-- ============================================
-- 19. Subquery with NULL handling
-- ============================================

SELECT name
FROM products
WHERE category_id IN (SELECT id FROM categories WHERE parent_id IS NULL);

SELECT name
FROM users
WHERE id NOT IN (SELECT user_id FROM orders WHERE user_id IS NOT NULL);

SELECT name
FROM products
WHERE (SELECT COUNT(*) FROM order_items WHERE product_id = products.id) = 0;

-- ============================================
-- 20. Subquery with BETWEEN
-- ============================================

SELECT name
FROM products
WHERE price BETWEEN (SELECT MIN(price) FROM products) AND (SELECT MAX(price) FROM products);

SELECT name
FROM orders
WHERE total BETWEEN (SELECT AVG(total) FROM orders) * 0.5 AND (SELECT AVG(total) FROM orders) * 1.5;

-- ============================================
-- 21. Subquery with LIKE
-- ============================================

SELECT name
FROM users
WHERE name LIKE (SELECT CONCAT('%', (SELECT name FROM users LIMIT 1), '%'));

SELECT name
FROM products
WHERE name LIKE (SELECT name FROM products ORDER BY RAND() LIMIT 1);

-- ============================================
-- 22. Subquery with CASE
-- ============================================

SELECT name, price,
    CASE
        WHEN price > (SELECT AVG(price) FROM products) THEN 'Above Average'
        WHEN price < (SELECT AVG(price) FROM products) THEN 'Below Average'
        ELSE 'Average'
    END as price_category
FROM products;

SELECT name,
    CASE
        WHEN (SELECT COUNT(*) FROM orders WHERE user_id = users.id) > 10 THEN 'VIP'
        WHEN (SELECT COUNT(*) FROM orders WHERE user_id = users.id) > 5 THEN 'Regular'
        ELSE 'New'
    END as customer_type
FROM users;

-- ============================================
-- 23. Subquery with date functions
-- ============================================

SELECT name
FROM orders
WHERE created_at > (SELECT MIN(created_at) FROM orders WHERE status = 'pending');

SELECT name
FROM users
WHERE created_at < (SELECT created_at FROM users WHERE name = 'Admin' LIMIT 1);

SELECT name
FROM products
WHERE created_at BETWEEN (SELECT DATE_SUB(CURDATE(), INTERVAL 1 YEAR)) AND CURDATE();

-- ============================================
-- 24. Subquery with GROUP_CONCAT
-- ============================================

SELECT category_id,
    GROUP_CONCAT((SELECT name FROM users WHERE id = orders.user_id) ORDER BY total DESC) as top_customers
FROM orders
GROUP BY category_id;

-- ============================================
-- 25. Subquery with ALL in HAVING
-- ============================================

SELECT user_id
FROM orders
GROUP BY user_id
HAVING SUM(total) > ALL (SELECT total FROM orders WHERE status = 'refunded');

SELECT category_id
FROM products
GROUP BY category_id
HAVING COUNT(*) > ALL (SELECT COUNT(*) FROM products WHERE category_id = 1);

-- ============================================
-- 26. Subquery for UPDATE
-- ============================================

UPDATE products SET price = price * 1.1 WHERE category_id IN (SELECT id FROM categories WHERE name = 'Electronics');

UPDATE users SET status = 'vip' WHERE (SELECT SUM(total) FROM orders WHERE orders.user_id = users.id) > 1000;

-- ============================================
-- 27. Subquery for DELETE
-- ============================================

DELETE FROM users WHERE id IN (SELECT user_id FROM orders WHERE total = 0 AND status = 'cancelled');

DELETE FROM products WHERE category_id IN (SELECT id FROM categories WHERE active = FALSE);

-- ============================================
-- 28. Subquery for INSERT
-- ============================================

INSERT INTO product_stats (category_name, product_count)
SELECT name, (SELECT COUNT(*) FROM products WHERE category_id = categories.id)
FROM categories;

-- ============================================
-- 29. Subquery with CTE (MySQL 8.0+)
-- ============================================

-- WITH high_value_orders AS (
--     SELECT user_id, SUM(total) as total
--     FROM orders
--     GROUP BY user_id
--     HAVING SUM(total) > 1000
-- )
-- SELECT u.name, h.total
-- FROM users u
-- JOIN high_value_orders h ON u.id = h.user_id;

-- ============================================
-- 30. Subquery with ROW_NUMBER (MySQL 8.0+)
-- ============================================

-- SELECT name, price
-- FROM (
--     SELECT name, price, ROW_NUMBER() OVER (PARTITION BY category_id ORDER BY price DESC) as rn
--     FROM products
-- ) ranked
-- WHERE rn <= 3;
