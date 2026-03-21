-- =====================================================
-- 实验 5: JOIN 多表关联
-- =====================================================
-- 目标: 掌握表之间关联查询

-- 5.1 INNER JOIN 内连接
SELECT 
    o.id,
    u.username,
    o.amount,
    o.status
FROM orders o
INNER JOIN users u ON o.user_id = u.id;

-- 5.2 LEFT JOIN 左连接
SELECT 
    u.username,
    o.id AS order_id,
    o.amount
FROM users u
LEFT JOIN orders o ON u.id = o.user_id;

-- 5.3 RIGHT JOIN 右连接
SELECT 
    o.id,
    p.name,
    oi.quantity
FROM orders o
RIGHT JOIN order_items oi ON o.id = oi.order_id;

-- 5.4 多表 JOIN
SELECT 
    u.username,
    o.id AS order_id,
    p.name AS product_name,
    oi.quantity
FROM users u
INNER JOIN orders o ON u.id = o.user_id
INNER JOIN order_items oi ON o.id = oi.order_id
INNER JOIN products p ON oi.product_id = p.id;

-- 5.5 自连接
SELECT 
    a.username AS user,
    b.username AS manager
FROM users a
INNER JOIN users b ON a.manager_id = b.id;

-- =====================================================
-- 教学提示:
-- - JOIN 用于关联多个表的数据
-- - INNER JOIN: 只返回匹配的记录
-- - LEFT JOIN: 返回左表所有记录，右表无匹配为 NULL
-- - RIGHT JOIN: 返回右表所有记录
-- =====================================================
