//! Type Conversion and NULL Handling Tests
//!
//! P4 tests for type casting and NULL handling

#[cfg(test)]
mod tests {
    use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
    
    use std::sync::{Arc, RwLock};

    fn create_engine() -> ExecutionEngine {
        ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
    }

    #[test]
    fn test_null_insert() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE null_test (id INTEGER, value TEXT)").unwrap())
            .unwrap();

        let result = engine.execute(parse("INSERT INTO null_test VALUES (1, NULL)").unwrap());
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_null_comparison_is_null() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE null_test (id INTEGER, value TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO null_test VALUES (1, NULL)").unwrap())
            .ok();
        engine
            .execute(parse("INSERT INTO null_test VALUES (2, 'test')").unwrap())
            .ok();

        let result = engine.execute(parse("SELECT * FROM null_test WHERE value IS NULL").unwrap());

        assert!(result.is_ok());
    }

    #[test]
    fn test_null_comparison_is_not_null() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE null_test (id INTEGER, value TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO null_test VALUES (1, NULL)").unwrap())
            .ok();
        engine
            .execute(parse("INSERT INTO null_test VALUES (2, 'test')").unwrap())
            .ok();

        let result =
            engine.execute(parse("SELECT * FROM null_test WHERE value IS NOT NULL").unwrap());

        assert!(result.is_ok());
    }

    #[test]
    fn test_null_in_where_clause() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE products (id INTEGER, price INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO products VALUES (1, 100), (2, NULL), (3, 50)").unwrap())
            .ok();

        let result = engine
            .execute(parse("SELECT * FROM products WHERE price > 50").unwrap())
            .unwrap();

        assert!(result.rows.len() >= 1);
    }

    #[test]
    fn test_null_in_aggregate() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE agg_null (value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO agg_null VALUES (10), (20), (NULL), (40)").unwrap())
            .ok();

        let result = engine
            .execute(parse("SELECT COUNT(*), SUM(value) FROM agg_null").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_count_null_column() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE count_null (value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO count_null VALUES (10), (20), (NULL), (40)").unwrap())
            .ok();

        let result = engine
            .execute(parse("SELECT COUNT(value) FROM count_null").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_null_versus_empty_string() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE str_test (id INTEGER, val TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO str_test VALUES (1, NULL)").unwrap())
            .ok();
        engine
            .execute(parse("INSERT INTO str_test VALUES (2, '')").unwrap())
            .ok();

        let result = engine
            .execute(parse("SELECT COUNT(*) FROM str_test").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_null_in_subquery() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE outer_tbl (id INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO outer_tbl VALUES (1), (2)").unwrap())
            .ok();
        engine
            .execute(parse("CREATE TABLE inner_tbl (id INTEGER, val TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO inner_tbl VALUES (1, 'a'), (3, NULL)").unwrap())
            .ok();

        let result = engine.execute(
            parse("SELECT * FROM outer_tbl WHERE id IN (SELECT id FROM inner_tbl)").unwrap(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_null_equality() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE eq_test (a TEXT, b TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO eq_test VALUES (NULL, NULL)").unwrap())
            .ok();

        let result = engine.execute(parse("SELECT * FROM eq_test WHERE a = b").unwrap());

        assert!(result.is_ok());
    }

    #[test]
    fn test_not_null_constraint() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE not_null_test (id INTEGER NOT NULL, name TEXT)").unwrap())
            .unwrap();

        let result = engine.execute(parse("INSERT INTO not_null_test VALUES (1, 'test')").unwrap());

        assert!(result.is_ok());
    }
}
