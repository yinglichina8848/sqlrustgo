-- =====================================================
-- 实验 9: DELETE 数据删除
-- =====================================================
-- 目标: 掌握数据删除

-- 9.1 删除指定记录
DELETE FROM users WHERE id = 1;

-- 9.2 条件删除
DELETE FROM orders 
WHERE status = 'cancelled' 
AND created_at < '2023-01-01';

-- 9.3 基于子查询删除
DELETE FROM order_items
WHERE order_id IN (
    SELECT id FROM orders 
    WHERE status = 'cancelled'
);

-- 9.4 删除所有记录（慎用！）
-- DELETE FROM audit_logs;  -- 危险操作

-- 9.5 清空表（更快）
-- TRUNCATE TABLE audit_logs;  -- 危险操作

-- =====================================================
-- 教学提示:
-- - DELETE 必须加 WHERE 条件
-- - TRUNCATE 比 DELETE 快，但无法回滚
-- - 删除前务必备份或确认
-- =====================================================
