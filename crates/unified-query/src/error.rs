use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnifiedQueryError {
    #[error("SQL error: {0}")]
    Sql(String),
    #[error("Vector error: {0}")]
    Vector(String),
    #[error("Graph error: {0}")]
    Graph(String),
    #[error("Routing error: {0}")]
    Routing(String),
    #[error("Fusion error: {0}")]
    Fusion(String),
}

pub enum QueryResult<T> {
    Ok(T),
    Partial(Vec<String>),
    Err(String),
}

impl<T> QueryResult<T> {
    pub fn is_ok(&self) -> bool {
        matches!(self, QueryResult::Ok(_))
    }

    pub fn is_partial(&self) -> bool {
        matches!(self, QueryResult::Partial(_))
    }

    pub fn unwrap_or_default(&self) -> Option<T>
    where
        T: Clone,
    {
        match self {
            QueryResult::Ok(t) => Some((*t).clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_result_ok() {
        let result: QueryResult<i32> = QueryResult::Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or_default(), Some(42));
    }

    #[test]
    fn test_query_result_partial() {
        let result: QueryResult<i32> = QueryResult::Partial(vec!["warning1".to_string()]);
        assert!(result.is_partial());
        assert_eq!(result.unwrap_or_default(), None);
    }

    #[test]
    fn test_query_result_err() {
        let result: QueryResult<i32> = QueryResult::Err("error".to_string());
        assert!(!result.is_ok());
        assert!(!result.is_partial());
    }
}
