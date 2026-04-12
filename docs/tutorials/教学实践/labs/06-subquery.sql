-- =====================================================
-- 实验 6: 子查询
-- =====================================================
-- 目标: 掌握嵌套查询

-- 6.1 WHERE 中的子查询
SELECT * FROM products 
WHERE price > (SELECT AVG(price) FROM products);

-- 6.2 IN 中的子查询
SELECT * FROM users 
WHERE id IN (
    SELECT DISTINCT user_id FROM orders 
    WHERE amount > 1000
);

-- 6.3 EXISTS 子查询
SELECT * FROM users u
WHERE EXISTS (
    SELECT 1 FROM orders o 
    WHERE o.user_id = u.id AND o.amount > 500
);

-- 6.4 FROM 中的子查询（派生表）
SELECT 
    category,
    avg_price
FROM (
    SELECT 
        category,
        AVG(price) AS avg_price
    FROM products
    GROUP BY category
) AS category_stats
WHERE avg_price > 100;

-- 6.5 关联子查询
SELECT 
    o.*,
    (SELECT COUNT(*) FROM order_items oi WHERE oi.order_id = o.id) AS item_count
FROM orders o;

-- =====================================================
-- 教学提示:
-- - 子查询是嵌套在主查询中的查询
-- - 可用在 WHERE、FROM、SELECT 等位置
-- - 关联子查询依赖外部查询的值
-- - 子查询可提高代码可读性，但注意性能
-- =====================================================
