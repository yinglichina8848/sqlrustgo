-- =====================================================
-- 实验 1: 基础 SELECT 查询
-- =====================================================
-- 目标: 掌握 SELECT 语句的基本用法
-- 数据库: MySQL / SQLRustGo 兼容

-- 1.1 查询所有列
SELECT * FROM users;

-- 1.2 查询指定列
SELECT username, email FROM users;

-- 1.3 使用列别名
SELECT 
    username AS '用户名',
    email AS '邮箱',
    created_at AS '创建时间'
FROM users;

-- 1.4 使用 DISTINCT 去重
SELECT DISTINCT status FROM orders;

-- 1.5 限制结果数量 (MySQL 语法)
SELECT * FROM products LIMIT 10;

-- =====================================================
-- 教学提示:
-- - SELECT 是 SQL 最常用的语句
-- - * 表示查询所有列（生产环境建议指定列名）
-- - LIMIT 用于控制返回行数
-- =====================================================
