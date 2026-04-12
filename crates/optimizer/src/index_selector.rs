//! Index Selection Rule for optimizer

use sqlrustgo_storage::predicate::Predicate;

#[derive(Debug, Clone)]
pub enum IndexSelection {
    SeqScan,
    IndexScan {
        column: String,
        predicate: Predicate,
    },
}

pub fn analyze_predicate_for_index(predicate: &Predicate) -> Option<(String, Predicate)> {
    match predicate {
        Predicate::Eq(col, _) => {
            if let sqlrustgo_storage::predicate::Expr::Column(name) = col.as_ref() {
                return Some((name.clone(), predicate.clone()));
            }
            None
        }
        Predicate::Lt(col, _) => {
            if let sqlrustgo_storage::predicate::Expr::Column(name) = col.as_ref() {
                return Some((name.clone(), predicate.clone()));
            }
            None
        }
        Predicate::Lte(col, _) => {
            if let sqlrustgo_storage::predicate::Expr::Column(name) = col.as_ref() {
                return Some((name.clone(), predicate.clone()));
            }
            None
        }
        Predicate::Gt(col, _) => {
            if let sqlrustgo_storage::predicate::Expr::Column(name) = col.as_ref() {
                return Some((name.clone(), predicate.clone()));
            }
            None
        }
        Predicate::Gte(col, _) => {
            if let sqlrustgo_storage::predicate::Expr::Column(name) = col.as_ref() {
                return Some((name.clone(), predicate.clone()));
            }
            None
        }
        Predicate::And(left, right) => {
            if let Some((col, pred)) = analyze_predicate_for_index(left) {
                return Some((col, pred));
            }
            analyze_predicate_for_index(right)
        }
        _ => None,
    }
}

pub fn estimate_index_benefit(table_rows: usize, selectivity: f64) -> f64 {
    let seq_cost = table_rows as f64;
    let index_cost = (table_rows as f64 * selectivity) + (table_rows as f64 * 0.01);
    seq_cost - index_cost
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_types::Value;

    #[test]
    fn test_analyze_eq_predicate() {
        let pred = Predicate::eq("id", Value::Integer(42));
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_some());
        let (col, _) = result.unwrap();
        assert_eq!(col, "id");
    }

    #[test]
    fn test_analyze_and_predicate() {
        let pred = Predicate::and(
            Predicate::eq("id", Value::Integer(42)),
            Predicate::gt("age", Value::Integer(18)),
        );
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_some());
    }
}
