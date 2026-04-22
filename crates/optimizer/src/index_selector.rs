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
    use sqlrustgo_storage::predicate::Expr;
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

    #[test]
    fn test_analyze_lt_predicate() {
        let pred = Predicate::lt("id", Value::Integer(100));
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_some());
        let (col, _) = result.unwrap();
        assert_eq!(col, "id");
    }

    #[test]
    fn test_analyze_lte_predicate() {
        let pred = Predicate::lte("age", Value::Integer(65));
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_some());
        let (col, p) = result.unwrap();
        assert_eq!(col, "age");
        match p {
            Predicate::Lte(_, _) => {}
            _ => panic!("Expected Lte predicate"),
        }
    }

    #[test]
    fn test_analyze_gt_predicate() {
        let pred = Predicate::gt("salary", Value::Integer(50000));
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_some());
        let (col, _) = result.unwrap();
        assert_eq!(col, "salary");
    }

    #[test]
    fn test_analyze_gte_predicate() {
        let pred = Predicate::gte("quantity", Value::Integer(10));
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_some());
        let (col, p) = result.unwrap();
        assert_eq!(col, "quantity");
        match p {
            Predicate::Gte(_, _) => {}
            _ => panic!("Expected Gte predicate"),
        }
    }

    #[test]
    fn test_analyze_and_predicate_left_first() {
        // Test that AND prefers left side
        let pred = Predicate::and(
            Predicate::eq("id", Value::Integer(1)),
            Predicate::gt("age", Value::Integer(18)),
        );
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_some());
        let (col, _) = result.unwrap();
        assert_eq!(col, "id"); // Left side should be preferred
    }

    #[test]
    fn test_analyze_or_predicate_returns_none() {
        let pred = Predicate::or(
            Predicate::eq("id", Value::Integer(1)),
            Predicate::eq("id", Value::Integer(2)),
        );
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_in_predicate_returns_none() {
        let pred = Predicate::In(
            Box::new(Expr::Column("id".to_string())),
            vec![
                Expr::Value(Value::Integer(1)),
                Expr::Value(Value::Integer(2)),
            ],
        );
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_is_null_predicate_returns_none() {
        let pred = Predicate::IsNull(Box::new(Expr::Column("email".to_string())));
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_is_not_null_predicate_returns_none() {
        let pred = Predicate::IsNotNull(Box::new(Expr::Column("email".to_string())));
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_not_predicate_returns_none() {
        let pred = Predicate::not(Predicate::eq("id", Value::Integer(1)));
        let result = analyze_predicate_for_index(&pred);
        assert!(result.is_none());
    }

    #[test]
    fn test_estimate_index_benefit() {
        let benefit = estimate_index_benefit(10000, 0.1);
        assert!(benefit > 0.0); // Index should be beneficial
        let seq_cost = 10000.0;
        let index_cost = (10000.0 * 0.1) + (10000.0 * 0.01);
        assert_eq!(benefit, seq_cost - index_cost);
    }

    #[test]
    fn test_estimate_index_benefit_high_selectivity() {
        // High selectivity (0.5) means index may not be beneficial
        let benefit = estimate_index_benefit(1000, 0.5);
        let seq_cost = 1000.0;
        let index_cost = (1000.0 * 0.5) + (1000.0 * 0.01);
        assert_eq!(benefit, seq_cost - index_cost);
    }

    #[test]
    fn test_estimate_index_benefit_zero_rows() {
        let benefit = estimate_index_benefit(0, 0.0);
        assert_eq!(benefit, 0.0);
    }

    #[test]
    fn test_index_selection_seqscan() {
        let selection = IndexSelection::SeqScan;
        match selection {
            IndexSelection::SeqScan => {}
            _ => panic!("Expected SeqScan"),
        }
    }

    #[test]
    fn test_index_selection_indexscan() {
        let selection = IndexSelection::IndexScan {
            column: "id".to_string(),
            predicate: Predicate::eq("id", Value::Integer(1)),
        };
        match selection {
            IndexSelection::IndexScan { column, .. } => {
                assert_eq!(column, "id");
            }
            _ => panic!("Expected IndexScan"),
        }
    }

    #[test]
    fn test_debug_trait() {
        let selection = IndexSelection::SeqScan;
        let debug_str = format!("{:?}", selection);
        assert!(debug_str.contains("SeqScan"));
    }
}
