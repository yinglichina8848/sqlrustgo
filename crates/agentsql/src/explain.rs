use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResult {
    pub plan: ExecutionPlan,
    pub warnings: Vec<String>,
    pub estimated_cost: f64,
    pub estimated_rows: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub node_type: String,
    pub table: Option<String>,
    pub alias: Option<String>,
    pub operation: String,
    pub cost: Option<f64>,
    pub rows: Option<u64>,
    pub children: Vec<ExecutionPlan>,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainOptions {
    pub format: ExplainFormat,
    pub verbose: bool,
    pub analyze: bool,
}

impl Default for ExplainOptions {
    fn default() -> Self {
        Self {
            format: ExplainFormat::Tree,
            verbose: false,
            analyze: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExplainFormat {
    Tree,
    Json,
    Text,
    Xml,
}

pub struct ExplainService {
    options: ExplainOptions,
}

impl ExplainService {
    pub fn new() -> Self {
        Self {
            options: ExplainOptions::default(),
        }
    }

    pub fn new_with_options(options: ExplainOptions) -> Self {
        Self { options }
    }

    pub fn explain(&self, sql: &str) -> ExplainResult {
        let plan = self.generate_plan(sql);
        let cost = self.estimate_cost(&plan);
        let rows = self.estimate_rows(&plan);
        let warnings = self.check_warnings(&plan);

        ExplainResult {
            plan,
            warnings,
            estimated_cost: cost,
            estimated_rows: rows,
        }
    }

    pub fn explain_json(&self, sql: &str) -> serde_json::Value {
        let result = self.explain(sql);
        serde_json::to_value(result).unwrap_or(serde_json::Value::Null)
    }

    pub fn explain_text(&self, sql: &str) -> String {
        let result = self.explain(sql);
        self.format_plan_text(&result.plan, 0)
    }

    fn generate_plan(&self, sql: &str) -> ExecutionPlan {
        let sql_lower = sql.to_lowercase();

        if sql_lower.starts_with("select") {
            self.explain_select(sql)
        } else if sql_lower.starts_with("insert") {
            self.explain_insert(sql)
        } else if sql_lower.starts_with("update") {
            self.explain_update(sql)
        } else if sql_lower.starts_with("delete") {
            self.explain_delete(sql)
        } else {
            ExecutionPlan {
                node_type: "Unknown".to_string(),
                table: None,
                alias: None,
                operation: sql.to_string(),
                cost: None,
                rows: None,
                children: vec![],
                details: HashMap::new(),
            }
        }
    }

    fn explain_select(&self, sql: &str) -> ExecutionPlan {
        let sql_lower = sql.to_lowercase();

        let mut details = HashMap::new();
        details.insert("sql".to_string(), serde_json::json!(sql));

        if sql_lower.contains("join") {
            details.insert("join_detected".to_string(), serde_json::json!(true));
        }

        if sql_lower.contains("where") {
            details.insert("filter".to_string(), serde_json::json!(true));
        }

        if sql_lower.contains("group by") {
            details.insert("group_by".to_string(), serde_json::json!(true));
        }

        if sql_lower.contains("order by") {
            details.insert("sort".to_string(), serde_json::json!(true));
        }

        if sql_lower.contains("limit") {
            details.insert("limit".to_string(), serde_json::json!(true));
        }

        let table = self.extract_table_name(sql);
        let mut children = vec![];

        if sql_lower.contains("join") {
            children.push(ExecutionPlan {
                node_type: "HashJoin".to_string(),
                table: None,
                alias: None,
                operation: "Hash Join".to_string(),
                cost: Some(1000.0),
                rows: Some(1000),
                children: vec![
                    ExecutionPlan {
                        node_type: "SeqScan".to_string(),
                        table: Some("table1".to_string()),
                        alias: Some("t1".to_string()),
                        operation: "Sequential Scan".to_string(),
                        cost: Some(500.0),
                        rows: Some(1000),
                        children: vec![],
                        details: HashMap::new(),
                    },
                    ExecutionPlan {
                        node_type: "SeqScan".to_string(),
                        table: Some("table2".to_string()),
                        alias: Some("t2".to_string()),
                        operation: "Sequential Scan".to_string(),
                        cost: Some(500.0),
                        rows: Some(500),
                        children: vec![],
                        details: HashMap::new(),
                    },
                ],
                details: HashMap::new(),
            });
        }

        if sql_lower.contains("where") {
            children.push(ExecutionPlan {
                node_type: "Filter".to_string(),
                table: None,
                alias: None,
                operation: "Filter".to_string(),
                cost: Some(100.0),
                rows: Some(100),
                children: vec![],
                details: HashMap::new(),
            });
        }

        ExecutionPlan {
            node_type: "Select".to_string(),
            table,
            alias: None,
            operation: "Select".to_string(),
            cost: Some(1100.0),
            rows: Some(100),
            children,
            details,
        }
    }

    fn explain_insert(&self, sql: &str) -> ExecutionPlan {
        let table = self.extract_table_name(sql);

        ExecutionPlan {
            node_type: "Insert".to_string(),
            table,
            alias: None,
            operation: "Insert".to_string(),
            cost: Some(50.0),
            rows: Some(1),
            children: vec![ExecutionPlan {
                node_type: "Result".to_string(),
                table: None,
                alias: None,
                operation: "Result".to_string(),
                cost: Some(10.0),
                rows: Some(1),
                children: vec![],
                details: HashMap::new(),
            }],
            details: {
                let mut d = HashMap::new();
                d.insert("sql".to_string(), serde_json::json!(sql));
                d
            },
        }
    }

    fn explain_update(&self, sql: &str) -> ExecutionPlan {
        let table = self.extract_table_name(sql);

        ExecutionPlan {
            node_type: "Update".to_string(),
            table: table.clone(),
            alias: None,
            operation: "Update".to_string(),
            cost: Some(200.0),
            rows: Some(1),
            children: vec![ExecutionPlan {
                node_type: "SeqScan".to_string(),
                table,
                alias: None,
                operation: "Sequential Scan".to_string(),
                cost: Some(100.0),
                rows: Some(10),
                children: vec![],
                details: HashMap::new(),
            }],
            details: {
                let mut d = HashMap::new();
                d.insert("sql".to_string(), serde_json::json!(sql));
                d
            },
        }
    }

    fn explain_delete(&self, sql: &str) -> ExecutionPlan {
        let table = self.extract_table_name(sql);

        ExecutionPlan {
            node_type: "Delete".to_string(),
            table: table.clone(),
            alias: None,
            operation: "Delete".to_string(),
            cost: Some(150.0),
            rows: Some(1),
            children: vec![ExecutionPlan {
                node_type: "SeqScan".to_string(),
                table,
                alias: None,
                operation: "Sequential Scan".to_string(),
                cost: Some(100.0),
                rows: Some(10),
                children: vec![],
                details: HashMap::new(),
            }],
            details: {
                let mut d = HashMap::new();
                d.insert("sql".to_string(), serde_json::json!(sql));
                d
            },
        }
    }

    fn extract_table_name(&self, sql: &str) -> Option<String> {
        let sql_lower = sql.to_lowercase();

        let keywords = ["from", "into", "update", "join"];
        for keyword in keywords {
            if let Some(pos) = sql_lower.find(keyword) {
                let rest = sql[pos + keyword.len()..].trim();
                let tokens: Vec<&str> = rest.split_whitespace().collect();
                if let Some(name) = tokens.first() {
                    let name = name.trim_matches(|c| c == '(' || c == ')' || c == ',');
                    if !name.is_empty() && !name.starts_with("select") {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }

    fn estimate_cost(&self, plan: &ExecutionPlan) -> f64 {
        let mut cost = plan.cost.unwrap_or(0.0);
        for child in &plan.children {
            cost += self.estimate_cost(child);
        }
        cost
    }

    fn estimate_rows(&self, plan: &ExecutionPlan) -> u64 {
        let rows = plan.rows.unwrap_or(0);
        if plan.children.is_empty() {
            rows
        } else {
            rows.max(
                plan.children
                    .iter()
                    .map(|c| self.estimate_rows(c))
                    .max()
                    .unwrap_or(0),
            )
        }
    }

    fn check_warnings(&self, plan: &ExecutionPlan) -> Vec<String> {
        let mut warnings = vec![];

        if let Some(rows) = plan.rows {
            if rows > 100000 {
                warnings
                    .push("Large result set detected. Consider adding LIMIT clause.".to_string());
            }
        }

        if let Some(table) = &plan.table {
            if table.contains("audit") || table.contains("log") {
                warnings.push(format!(
                    "Scanning large table '{}'. Consider using indexes.",
                    table
                ));
            }
        }

        for child in &plan.children {
            warnings.extend(self.check_warnings(child));
        }

        warnings
    }

    fn format_plan_text(&self, plan: &ExecutionPlan, indent: usize) -> String {
        let prefix = "  ".repeat(indent);
        let mut output = format!(
            "{}{} ({:.2}) [{} rows]\n",
            prefix,
            plan.operation,
            plan.cost.unwrap_or(0.0),
            plan.rows.unwrap_or(0)
        );

        for child in &plan.children {
            output.push_str(&self.format_plan_text(child, indent + 1));
        }

        output
    }
}

impl Default for ExplainService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_simple_select() {
        let service = ExplainService::new();
        let result = service.explain("SELECT * FROM users WHERE id = 1");

        assert_eq!(result.plan.node_type, "Select");
        assert!(result.estimated_cost > 0.0);
    }

    #[test]
    fn test_explain_select_with_join() {
        let service = ExplainService::new();
        let result =
            service.explain("SELECT * FROM users JOIN orders ON users.id = orders.user_id");

        assert_eq!(result.plan.node_type, "Select");
        assert!(!result.plan.children.is_empty());
    }

    #[test]
    fn test_explain_insert() {
        let service = ExplainService::new();
        let result =
            service.explain("INSERT INTO users (name, email) VALUES ('test', 'test@example.com')");

        assert_eq!(result.plan.node_type, "Insert");
    }

    #[test]
    fn test_explain_text_format() {
        let service = ExplainService::new();
        let text = service.explain_text("SELECT * FROM users");

        assert!(text.contains("Select"));
    }

    #[test]
    fn test_warnings_for_large_table() {
        let service = ExplainService::new();
        let result = service.explain("SELECT * FROM audit_log");

        assert!(!result.warnings.is_empty());
    }
}
