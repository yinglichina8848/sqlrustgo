-- =====================================================
-- 实验 2: WHERE 条件过滤
-- =====================================================
-- 目标: 掌握 WHERE 子句的条件过滤

-- 2.1 简单条件
SELECT * FROM users WHERE status = 'active';

-- 2.2 多条件 AND
SELECT * FROM products 
WHERE category = 'electronics' AND price > 1000;

-- 2.3 多条件 OR
SELECT * FROM orders 
WHERE status = 'pending' OR status = 'processing';

-- 2.4 NOT 否定条件
SELECT * FROM users WHERE NOT status = 'banned';

-- 2.5 IN 操作符
SELECT * FROM products 
WHERE category IN ('electronics', 'books', 'clothing');

-- 2.6 BETWEEN 范围查询
SELECT * FROM orders 
WHERE created_at BETWEEN '2024-01-01' AND '2024-12-31';

-- 2.7 LIKE 模糊匹配
SELECT * FROM users WHERE email LIKE '%@gmail.com';

-- 2.8 NULL 值判断
SELECT * FROM users WHERE phone IS NOT NULL;

-- =====================================================
-- 教学提示:
-- - WHERE 条件区分大小写（取决于数据库配置）
-- - AND 优先级高于 OR，可用括号改变优先级
-- - NULL 值判断必须使用 IS NULL / IS NOT NULL
-- =====================================================
