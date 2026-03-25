-- =====================================================
-- 实验 4: 聚合函数与 GROUP BY
-- =====================================================
-- 目标: 掌握数据统计与分组

-- 4.1 计数 COUNT
SELECT COUNT(*) FROM orders;
SELECT COUNT(DISTINCT user_id) FROM orders;

-- 4.2 求和 SUM
SELECT SUM(amount) FROM orders WHERE status = 'completed';

-- 4.3 平均 AVG
SELECT AVG(price) FROM products;

-- 4.4 最大最小值
SELECT 
    MAX(price) AS 最高价,
    MIN(price) AS 最低价,
    AVG(price) AS 平均价
FROM products;

-- 4.5 GROUP BY 分组
SELECT 
    category,
    COUNT(*) AS product_count,
    AVG(price) AS avg_price
FROM products
GROUP BY category;

-- 4.6 GROUP BY + HAVING
SELECT 
    user_id,
    COUNT(*) AS order_count,
    SUM(amount) AS total_amount
FROM orders
GROUP BY user_id
HAVING COUNT(*) > 5;

-- =====================================================
-- 教学提示:
-- - 聚合函数对一组值进行计算
-- - GROUP BY 将数据分组
-- - HAVING 过滤分组后的数据（类似 WHERE）
-- =====================================================
