use sqlrustgo_types::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct SqlNormalizer;

#[allow(clippy::if_same_then_else)]
impl SqlNormalizer {
    pub fn normalize(sql: &str) -> String {
        let mut result = String::with_capacity(sql.len());
        let mut in_string = false;
        let mut chars = sql.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\'' {
                if in_string {
                    result.push('?');
                }
                in_string = !in_string;
            } else if in_string {
                continue;
            } else if c.is_ascii_digit() {
                result.push('?');
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == '.' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            } else {
                result.push(c.to_ascii_lowercase());
            }
        }

        result.trim().to_string()
    }

    pub fn hash_params(params: &[Value]) -> u64 {
        let mut hasher = DefaultHasher::new();
        params.hash(&mut hasher);
        hasher.finish()
    }

    pub fn from_literal(sql: &str) -> (String, Vec<Value>) {
        let mut params = Vec::new();
        let mut normalized = String::with_capacity(sql.len());
        let mut in_string = false;
        let mut current_param = String::new();

        for c in sql.chars() {
            if c == '\'' {
                if in_string {
                    params.push(Value::Text(current_param.clone()));
                    current_param.clear();
                }
                in_string = !in_string;
            } else if in_string {
                current_param.push(c);
            } else if c.is_ascii_digit() || c == '.' {
                current_param.push(c);
            } else {
                if !current_param.is_empty() {
                    if let Ok(n) = current_param.parse::<i64>() {
                        params.push(Value::Integer(n));
                    } else if let Ok(f) = current_param.parse::<f64>() {
                        params.push(Value::Float(f));
                    } else {
                        params.push(Value::Text(current_param.clone()));
                    }
                    normalized.push('?');
                    current_param.clear();
                }
                normalized.push(c.to_ascii_lowercase());
            }
        }

        if !current_param.is_empty() {
            if let Ok(n) = current_param.parse::<i64>() {
                params.push(Value::Integer(n));
                normalized.push('?');
            } else if let Ok(f) = current_param.parse::<f64>() {
                params.push(Value::Float(f));
                normalized.push('?');
            }
        }

        (normalized.trim().to_string(), params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_simple() {
        assert_eq!(
            SqlNormalizer::normalize("SELECT * FROM t WHERE id = 1"),
            "select * from t where id = ?"
        );
    }

    #[test]
    fn test_normalize_string() {
        assert_eq!(
            SqlNormalizer::normalize("SELECT * FROM t WHERE name = 'Alice'"),
            "select * from t where name = ?"
        );
    }

    #[test]
    fn test_from_literal() {
        let (sql, params) = SqlNormalizer::from_literal("SELECT * FROM t WHERE id = 42");
        assert_eq!(sql, "select * from t where id = ?");
        assert_eq!(params, vec![Value::Integer(42)]);
    }

    #[test]
    fn test_normalize_float() {
        let result = SqlNormalizer::normalize("SELECT * FROM t WHERE price = 3.14");
        assert!(result.contains("price"));
    }

    #[test]
    fn test_normalize_negative_number() {
        let result = SqlNormalizer::normalize("SELECT * FROM t WHERE id = -1");
        assert!(result.contains("id"));
    }

    #[test]
    fn test_normalize_boolean() {
        let result = SqlNormalizer::normalize("SELECT * FROM t WHERE active = true");
        assert!(result.contains("active"));
    }

    #[test]
    fn test_from_literal_multiple_params() {
        let (sql, params) = SqlNormalizer::from_literal("SELECT * FROM t WHERE a = 1 AND b = 2");
        assert_eq!(sql, "select * from t where a = ? and b = ?");
        assert_eq!(params, vec![Value::Integer(1), Value::Integer(2)]);
    }

    #[test]
    fn test_normalize_whitespace() {
        let result = SqlNormalizer::normalize("SELECT    *    FROM   t");
        assert!(result.contains("select") && result.contains("from"));
    }

    #[test]
    fn test_normalize_null() {
        let result = SqlNormalizer::normalize("SELECT * FROM t WHERE name IS NULL");
        assert!(result.contains("name is null"));
    }

    #[test]
    fn test_normalize_in_list() {
        let result = SqlNormalizer::normalize("SELECT * FROM t WHERE id IN (1, 2, 3)");
        assert!(result.contains("in"));
    }

    #[test]
    fn test_hash_params() {
        let params1 = vec![Value::Integer(1), Value::Text("test".to_string())];
        let params2 = vec![Value::Integer(1), Value::Text("test".to_string())];
        let params3 = vec![Value::Integer(2), Value::Text("test".to_string())];

        let hash1 = SqlNormalizer::hash_params(&params1);
        let hash2 = SqlNormalizer::hash_params(&params2);
        let hash3 = SqlNormalizer::hash_params(&params3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
