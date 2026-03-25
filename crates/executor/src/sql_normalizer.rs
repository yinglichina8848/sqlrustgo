use sqlrustgo_types::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct SqlNormalizer;

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
                    }
                    current_param.clear();
                    normalized.push('?');
                }
                normalized.push(c.to_ascii_lowercase());
            }
        }

        if !current_param.is_empty() {
            if let Ok(n) = current_param.parse::<i64>() {
                params.push(Value::Integer(n));
            } else if let Ok(_) = current_param.parse::<f64>() {
                // float case
            }
            normalized.push('?');
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
}
