//! Aggregate and Type Conversion Tests
//!
//! P4 tests for aggregate functions and type conversions

#[cfg(test)]
mod tests {
    use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};

    use std::sync::{Arc, RwLock};

    fn create_engine() -> ExecutionEngine {
        ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
    }

    #[test]
    fn test_count_star() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE counts (id INTEGER, value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO counts VALUES (1, 10), (2, 20), (3, 30)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("SELECT COUNT(*) FROM counts").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_count_column() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE counts (id INTEGER, value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO counts VALUES (1, 10), (2, 20), (3, 30)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("SELECT COUNT(id) FROM counts").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_sum_aggregate() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE numbers (id INTEGER, value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO numbers VALUES (1, 100), (2, 200), (3, 300)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("SELECT SUM(value) FROM numbers").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_avg_aggregate() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE numbers (id INTEGER, value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO numbers VALUES (1, 10), (2, 20), (3, 30)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("SELECT AVG(value) FROM numbers").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_min_aggregate() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE numbers (id INTEGER, value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO numbers VALUES (1, 30), (2, 10), (3, 20)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("SELECT MIN(value) FROM numbers").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_max_aggregate() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE numbers (id INTEGER, value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO numbers VALUES (1, 30), (2, 10), (3, 20)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("SELECT MAX(value) FROM numbers").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_multiple_aggregates() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE stats (value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO stats VALUES (10), (20), (30)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("SELECT COUNT(*), SUM(value), AVG(value) FROM stats").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_aggregate_empty_table() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE empty_table (value INTEGER)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("SELECT COUNT(*), SUM(value) FROM empty_table").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_aggregate_single_row() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE single (value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO single VALUES (42)").unwrap())
            .unwrap();

        let result = engine
            .execute(
                parse(
                    "SELECT COUNT(*), SUM(value), AVG(value), MIN(value), MAX(value) FROM single",
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_aggregate_negative_values() {
        let mut engine = create_engine();
        engine
            .execute(parse("CREATE TABLE negatives (value INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO negatives VALUES (-10), (-20), (30)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("SELECT SUM(value), AVG(value) FROM negatives").unwrap())
            .unwrap();

        assert_eq!(result.rows.len(), 1);
    }
}
