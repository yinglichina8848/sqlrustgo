use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct TlpOracle {
    num_threads: usize,
}

impl TlpOracle {
    pub fn new(num_threads: usize) -> Self {
        Self { num_threads }
    }

    pub fn check<F, R>(&self, query: String, executor: F) -> OracleResult
    where
        F: Fn(&str) -> Result<R, String> + Send + Clone + 'static,
        R: Clone + Send + 'static + PartialEq,
    {
        let (tx, rx) = mpsc::channel();
        let mut handles = vec![];

        for _ in 0..self.num_threads {
            let tx = tx.clone();
            let query = query.clone();
            let executor = executor.clone();

            let handle = thread::spawn(move || {
                let result = executor(&query);
                let _ = tx.send(result);
            });
            handles.push(handle);
        }

        drop(tx);

        let mut results = Vec::new();
        for result in rx {
            results.push(result);
        }

        for handle in handles {
            let _ = handle.join();
        }

        if results.is_empty() {
            return OracleResult {
                consistent: false,
                error: Some("No results received".to_string()),
            };
        }

        let first_result = &results[0];
        let consistent = results.iter().all(|r| match (first_result, r) {
            (Ok(f), Ok(r)) => f == r,
            (Err(_), Err(_)) => true,
            _ => false,
        });

        OracleResult {
            consistent,
            error: if consistent {
                None
            } else {
                Some(format!(
                    "Inconsistent results across {} threads",
                    self.num_threads
                ))
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct OracleResult {
    pub consistent: bool,
    pub error: Option<String>,
}

impl OracleResult {
    pub fn ok() -> Self {
        Self {
            consistent: true,
            error: None,
        }
    }

    pub fn err(msg: &str) -> Self {
        Self {
            consistent: false,
            error: Some(msg.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tlp_oracle_consistent() {
        let oracle = TlpOracle::new(3);
        let executor = |_: &str| -> Result<i32, String> { Ok(42) };

        let result = oracle.check::<_, i32>("SELECT 1".to_string(), executor);
        assert!(result.consistent);
    }

    #[test]
    fn test_tlp_oracle_error_consistency() {
        let oracle = TlpOracle::new(3);

        let result =
            oracle.check::<_, String>("SELECT 1".to_string(), |_| Err("Same error".to_string()));
        assert!(result.consistent);
    }
}
