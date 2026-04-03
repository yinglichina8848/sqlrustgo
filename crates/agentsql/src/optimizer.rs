use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub original_sql: String,
    pub optimized_sql: String,
    pub suggestions: Vec<OptimizationSuggestion>,
    pub estimated_improvement: PerformanceEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub id: String,
    pub category: SuggestionCategory,
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub estimated_savings: Option<String>,
    pub sql_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionCategory {
    Index,
    QueryRewrite,
    Schema,
    Configuration,
    Join,
    Aggregation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEstimate {
    pub before_ms: f64,
    pub after_ms: Option<f64>,
    pub improvement_percent: Option<f64>,
}

pub struct OptimizerService {
    rules: Vec<OptimizationRule>,
}

struct OptimizationRule {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    category: SuggestionCategory,
    #[allow(dead_code)]
    priority: Priority,
    check: fn(&str) -> Option<OptimizationSuggestion>,
    apply: fn(&str) -> String,
}

impl OptimizerService {
    pub fn new() -> Self {
        let rules = vec![
            OptimizationRule {
                name: "add_limit".to_string(),
                category: SuggestionCategory::QueryRewrite,
                priority: Priority::High,
                check: Self::check_missing_limit,
                apply: Self::apply_add_limit,
            },
            OptimizationRule {
                name: "avoid_select_star".to_string(),
                category: SuggestionCategory::QueryRewrite,
                priority: Priority::Medium,
                check: Self::check_select_star,
                apply: Self::apply_select_star,
            },
            OptimizationRule {
                name: "use_index".to_string(),
                category: SuggestionCategory::Index,
                priority: Priority::High,
                check: Self::check_missing_index,
                apply: Self::apply_index_hint,
            },
            OptimizationRule {
                name: "optimize_join_order".to_string(),
                category: SuggestionCategory::Join,
                priority: Priority::Medium,
                check: Self::check_join_order,
                apply: Self::apply_join_order,
            },
            OptimizationRule {
                name: "use_explicit_join".to_string(),
                category: SuggestionCategory::Join,
                priority: Priority::Low,
                check: Self::check_implicit_join,
                apply: Self::apply_explicit_join,
            },
        ];

        Self { rules }
    }

    pub fn optimize(&self, sql: &str) -> OptimizationResult {
        let mut suggestions = vec![];
        let mut optimized_sql = sql.to_string();

        for rule in &self.rules {
            if let Some(suggestion) = (rule.check)(sql) {
                suggestions.push(suggestion);
                optimized_sql = (rule.apply)(&optimized_sql);
            }
        }

        let before_ms = self.estimate_query_time(sql);
        let after_ms = self.estimate_query_time(&optimized_sql);
        let improvement = if before_ms > 0.0 {
            Some(((before_ms - after_ms) / before_ms) * 100.0)
        } else {
            None
        };

        OptimizationResult {
            original_sql: sql.to_string(),
            optimized_sql,
            suggestions,
            estimated_improvement: PerformanceEstimate {
                before_ms,
                after_ms: Some(after_ms),
                improvement_percent: improvement,
            },
        }
    }

    pub fn analyze(&self, sql: &str) -> Vec<OptimizationSuggestion> {
        self.rules
            .iter()
            .filter_map(|rule| (rule.check)(sql))
            .collect()
    }

    fn estimate_query_time(&self, sql: &str) -> f64 {
        let sql_lower = sql.to_lowercase();
        let mut base_time = 10.0;

        if sql_lower.contains("select") {
            base_time *= 2.0;
        }
        if sql_lower.contains("join") {
            base_time *= 3.0;
        }
        if sql_lower.contains("like") {
            base_time *= 2.0;
        }
        if !sql_lower.contains("where") && sql_lower.contains("select") {
            base_time *= 1.5;
        }
        if sql_lower.contains("order by") {
            base_time *= 1.5;
        }
        if sql_lower.contains("group by") {
            base_time *= 1.3;
        }

        base_time
    }

    fn check_missing_limit(sql: &str) -> Option<OptimizationSuggestion> {
        let sql_lower = sql.to_lowercase();
        if sql_lower.starts_with("select")
            && !sql_lower.contains("limit")
            && !sql_lower.contains("top ")
        {
            Some(OptimizationSuggestion {
                id: "add_limit".to_string(),
                category: SuggestionCategory::QueryRewrite,
                priority: Priority::High,
                title: "Add LIMIT clause".to_string(),
                description: "Query does not have a LIMIT clause which may return excessive rows"
                    .to_string(),
                estimated_savings: Some("50-90%".to_string()),
                sql_hint: Some("Add LIMIT n to restrict result set size".to_string()),
            })
        } else {
            None
        }
    }

    fn apply_add_limit(sql: &str) -> String {
        if sql.to_lowercase().contains("limit") {
            sql.to_string()
        } else {
            format!("{} LIMIT 100", sql.trim_end_matches(';'))
        }
    }

    fn check_select_star(sql: &str) -> Option<OptimizationSuggestion> {
        let sql_lower = sql.to_lowercase();
        if sql_lower.contains("select *") {
            Some(OptimizationSuggestion {
                id: "avoid_select_star".to_string(),
                category: SuggestionCategory::QueryRewrite,
                priority: Priority::Medium,
                title: "Avoid SELECT *".to_string(),
                description:
                    "SELECT * retrieves unnecessary columns and may miss future schema changes"
                        .to_string(),
                estimated_savings: Some("20-40%".to_string()),
                sql_hint: Some("Specify only needed columns explicitly".to_string()),
            })
        } else {
            None
        }
    }

    fn apply_select_star(sql: &str) -> String {
        sql.to_string()
    }

    fn check_missing_index(sql: &str) -> Option<OptimizationSuggestion> {
        let sql_lower = sql.to_lowercase();

        if sql_lower.contains("where") && !sql_lower.contains("index") && sql_lower.contains("=") {
            return Some(OptimizationSuggestion {
                id: "use_index".to_string(),
                category: SuggestionCategory::Index,
                priority: Priority::High,
                title: "Consider adding index".to_string(),
                description: "WHERE clause on non-indexed column may cause full table scan"
                    .to_string(),
                estimated_savings: Some("70-95%".to_string()),
                sql_hint: Some("CREATE INDEX idx ON table(column)".to_string()),
            });
        }
        None
    }

    fn apply_index_hint(sql: &str) -> String {
        sql.to_string()
    }

    fn check_join_order(sql: &str) -> Option<OptimizationSuggestion> {
        let sql_lower = sql.to_lowercase();
        if sql_lower.contains("from") {
            let join_count = sql_lower.matches("join").count();
            let has_implicit_join = sql_lower.contains(",")
                && sql_lower.contains("where")
                && !sql_lower.contains("join");
            if join_count > 1 || has_implicit_join {
                return Some(OptimizationSuggestion {
                    id: "optimize_join_order".to_string(),
                    category: SuggestionCategory::Join,
                    priority: Priority::Medium,
                    title: "Optimize JOIN order".to_string(),
                    description:
                        "Consider joining smaller tables first to reduce intermediate results"
                            .to_string(),
                    estimated_savings: Some("30-60%".to_string()),
                    sql_hint: Some("Reorder JOINs to process smaller tables first".to_string()),
                });
            }
        }
        None
    }

    fn apply_join_order(sql: &str) -> String {
        sql.to_string()
    }

    fn check_implicit_join(sql: &str) -> Option<OptimizationSuggestion> {
        let sql_lower = sql.to_lowercase();
        if sql_lower.contains("from")
            && sql_lower.contains("where")
            && sql_lower.contains(",")
            && !sql_lower.contains("join")
        {
            return Some(OptimizationSuggestion {
                id: "use_explicit_join".to_string(),
                category: SuggestionCategory::Join,
                priority: Priority::Low,
                title: "Use explicit JOIN syntax".to_string(),
                description:
                    "Implicit joins (comma-separated tables) are harder to maintain and debug"
                        .to_string(),
                estimated_savings: Some("5-10%".to_string()),
                sql_hint: Some("Use INNER JOIN / LEFT JOIN syntax instead".to_string()),
            });
        }
        None
    }

    fn apply_explicit_join(sql: &str) -> String {
        sql.to_string()
    }
}

impl Default for OptimizerService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_missing_limit() {
        let optimizer = OptimizerService::new();
        let result = optimizer.optimize("SELECT * FROM users");

        assert!(result.suggestions.iter().any(|s| s.id == "add_limit"));
        assert!(result.optimized_sql.to_lowercase().contains("limit"));
    }

    #[test]
    fn test_optimize_select_star() {
        let optimizer = OptimizerService::new();
        let result = optimizer.optimize("SELECT * FROM users WHERE id = 1");

        assert!(result
            .suggestions
            .iter()
            .any(|s| s.id == "avoid_select_star"));
    }

    #[test]
    fn test_analyze_queries() {
        let optimizer = OptimizerService::new();
        let suggestions = optimizer.analyze("SELECT * FROM users WHERE active = true");

        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_performance_estimate() {
        let optimizer = OptimizerService::new();
        let result = optimizer.optimize("SELECT * FROM users");

        assert!(result.estimated_improvement.before_ms > 0.0);
    }

    #[test]
    fn test_suggestion_priorities() {
        let optimizer = OptimizerService::new();
        let suggestions = optimizer.analyze("SELECT * FROM users");

        for suggestion in suggestions {
            assert!(matches!(
                suggestion.priority,
                Priority::High | Priority::Medium | Priority::Low
            ));
        }
    }
}
