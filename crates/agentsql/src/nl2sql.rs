use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Nl2SqlService {
    schema_service: Arc<crate::schema::SchemaService>,
    keyword_mappings: HashMap<String, String>,
    table_column_hints: HashMap<String, HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlQueryRequest {
    pub query: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlQueryResponse {
    pub sql: String,
    pub confidence: f32,
    pub table_hint: Option<String>,
    pub where_conditions: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlExplanation {
    pub sql: String,
    pub explanation: String,
    pub tables_used: Vec<String>,
    pub columns_used: Vec<String>,
    pub operations: Vec<String>,
}

impl Nl2SqlService {
    pub fn new(schema_service: Arc<crate::schema::SchemaService>) -> Self {
        let mut keyword_mappings = HashMap::new();
        keyword_mappings.insert("show".to_string(), "SELECT".to_string());
        keyword_mappings.insert("list".to_string(), "SELECT".to_string());
        keyword_mappings.insert("get".to_string(), "SELECT".to_string());
        keyword_mappings.insert("find".to_string(), "SELECT".to_string());
        keyword_mappings.insert("display".to_string(), "SELECT".to_string());
        keyword_mappings.insert("all".to_string(), "*".to_string());
        keyword_mappings.insert("every".to_string(), "*".to_string());
        keyword_mappings.insert("total".to_string(), "COUNT(*)".to_string());
        keyword_mappings.insert("count".to_string(), "COUNT(*)".to_string());
        keyword_mappings.insert("sum".to_string(), "SUM".to_string());
        keyword_mappings.insert("average".to_string(), "AVG".to_string());
        keyword_mappings.insert("maximum".to_string(), "MAX".to_string());
        keyword_mappings.insert("maximum".to_string(), "MIN".to_string());
        keyword_mappings.insert("minimum".to_string(), "MIN".to_string());
        keyword_mappings.insert("top".to_string(), "LIMIT".to_string());
        keyword_mappings.insert("first".to_string(), "LIMIT".to_string());
        keyword_mappings.insert("users".to_string(), "users".to_string());
        keyword_mappings.insert("user".to_string(), "users".to_string());
        keyword_mappings.insert("products".to_string(), "products".to_string());
        keyword_mappings.insert("product".to_string(), "products".to_string());
        keyword_mappings.insert("orders".to_string(), "orders".to_string());
        keyword_mappings.insert("order".to_string(), "orders".to_string());
        keyword_mappings.insert("id".to_string(), "id".to_string());
        keyword_mappings.insert("name".to_string(), "name".to_string());
        keyword_mappings.insert("email".to_string(), "email".to_string());
        keyword_mappings.insert("price".to_string(), "price".to_string());
        keyword_mappings.insert("total".to_string(), "total_price".to_string());
        keyword_mappings.insert("status".to_string(), "status".to_string());
        keyword_mappings.insert("created".to_string(), "created_at".to_string());
        keyword_mappings.insert("active".to_string(), "'active'".to_string());
        keyword_mappings.insert("pending".to_string(), "'pending'".to_string());
        keyword_mappings.insert("completed".to_string(), "'completed'".to_string());
        keyword_mappings.insert("where".to_string(), "WHERE".to_string());
        keyword_mappings.insert("and".to_string(), "AND".to_string());
        keyword_mappings.insert("or".to_string(), "OR".to_string());
        keyword_mappings.insert("order by".to_string(), "ORDER BY".to_string());
        keyword_mappings.insert("group by".to_string(), "GROUP BY".to_string());
        keyword_mappings.insert("having".to_string(), "HAVING".to_string());
        keyword_mappings.insert("join".to_string(), "JOIN".to_string());
        keyword_mappings.insert("left join".to_string(), "LEFT JOIN".to_string());
        keyword_mappings.insert("inner join".to_string(), "INNER JOIN".to_string());
        keyword_mappings.insert("on".to_string(), "ON".to_string());

        let mut table_column_hints = HashMap::new();
        table_column_hints.insert("users".to_string(), {
            let mut m = HashMap::new();
            m.insert(
                "user".to_string(),
                vec!["id".to_string(), "name".to_string(), "email".to_string()],
            );
            m.insert(
                "account".to_string(),
                vec!["id".to_string(), "name".to_string(), "email".to_string()],
            );
            m
        });
        table_column_hints.insert("products".to_string(), {
            let mut m = HashMap::new();
            m.insert(
                "product".to_string(),
                vec!["id".to_string(), "name".to_string(), "price".to_string()],
            );
            m.insert(
                "item".to_string(),
                vec!["id".to_string(), "name".to_string(), "price".to_string()],
            );
            m
        });
        table_column_hints.insert("orders".to_string(), {
            let mut m = HashMap::new();
            m.insert(
                "order".to_string(),
                vec![
                    "id".to_string(),
                    "user_id".to_string(),
                    "total_price".to_string(),
                    "status".to_string(),
                ],
            );
            m.insert(
                "purchase".to_string(),
                vec![
                    "id".to_string(),
                    "user_id".to_string(),
                    "total_price".to_string(),
                    "status".to_string(),
                ],
            );
            m
        });

        Self {
            schema_service,
            keyword_mappings,
            table_column_hints,
        }
    }

    pub fn natural_language_to_sql(&self, nl_query: &str) -> NlQueryResponse {
        let query_lower = nl_query.to_lowercase();
        let tokens: Vec<&str> = query_lower.split_whitespace().collect();

        let mut tables = Vec::new();
        let mut columns = Vec::new();
        let mut conditions = Vec::new();
        let mut select_columns = String::from("*");
        let mut has_aggregation = false;
        let mut order_column = String::new();
        let mut order_direction = String::new();
        let mut limit_value = String::new();

        for token in &tokens {
            if let Some(table) = self.find_table_match(token) {
                if !tables.contains(&table) {
                    tables.push(table);
                }
            }
            if let Some(cols) = self.find_column_hints(token) {
                for col in cols {
                    if !columns.contains(&col) {
                        columns.push(col);
                    }
                }
            }
        }

        if query_lower.contains("count") || query_lower.contains("total") {
            select_columns = "COUNT(*)".to_string();
            has_aggregation = true;
        } else if query_lower.contains("sum of") {
            if let Some(idx) = tokens.iter().position(|t| *t == "sum") {
                if tokens.get(idx + 1) == Some(&"of") {
                    if let Some(col) = tokens.get(idx + 2) {
                        select_columns = format!("SUM({})", col);
                        has_aggregation = true;
                        columns.push(col.to_string());
                    }
                }
            }
        } else if query_lower.contains("average") || query_lower.contains("avg") {
            if let Some(idx) = tokens.iter().position(|&t| t == "average" || t == "avg") {
                if let Some(col) = tokens.get(idx + 1) {
                    select_columns = format!("AVG({})", col);
                    has_aggregation = true;
                    columns.push(col.to_string());
                }
            }
        } else if query_lower.contains("max") || query_lower.contains("maximum") {
            if let Some(idx) = tokens.iter().position(|&t| t == "max" || t == "maximum") {
                if let Some(col) = tokens.get(idx + 1) {
                    select_columns = format!("MAX({})", col);
                    has_aggregation = true;
                    columns.push(col.to_string());
                }
            }
        } else if query_lower.contains("min") || query_lower.contains("minimum") {
            if let Some(idx) = tokens.iter().position(|&t| t == "min" || t == "minimum") {
                if let Some(col) = tokens.get(idx + 1) {
                    select_columns = format!("MIN({})", col);
                    has_aggregation = true;
                    columns.push(col.to_string());
                }
            }
        }

        if query_lower.contains("where")
            || query_lower.contains("equals")
            || query_lower.contains("is")
        {
            if let Some(pos) = query_lower.find("where") {
                let after_where = &query_lower[pos + 5..];
                if let Some(end) = after_where.find(" and ").or(after_where.find(" order ")) {
                    let condition_str = after_where[..end].trim();
                    if !condition_str.is_empty() {
                        conditions.push(condition_str.to_string());
                    }
                } else {
                    let condition_str = after_where.trim();
                    if !condition_str.is_empty() {
                        conditions.push(condition_str.to_string());
                    }
                }
            }
            if query_lower.contains("active") && !conditions.iter().any(|c| c.contains("active")) {
                conditions.push("status = 'active'".to_string());
            }
            if query_lower.contains("pending") && !conditions.iter().any(|c| c.contains("pending"))
            {
                conditions.push("status = 'pending'".to_string());
            }
            if query_lower.contains("completed")
                && !conditions.iter().any(|c| c.contains("completed"))
            {
                conditions.push("status = 'completed'".to_string());
            }
        }

        if query_lower.contains("order by") {
            if let Some(pos) = query_lower.find("order by") {
                let after_order = &query_lower[pos + 8..];
                let order_tokens: Vec<&str> = after_order.split_whitespace().collect();
                if let Some(col) = order_tokens.first() {
                    order_column = col.to_string();
                }
                if after_order.contains("desc") {
                    order_direction = "DESC".to_string();
                } else {
                    order_direction = "ASC".to_string();
                }
            }
        }

        if query_lower.contains("limit") {
            if let Some(pos) = query_lower.find("limit") {
                let after_limit = &query_lower[pos + 5..];
                let limit_tokens: Vec<&str> = after_limit.split_whitespace().collect();
                if let Some(val) = limit_tokens.first() {
                    if let Ok(num) = val.parse::<u32>() {
                        limit_value = num.to_string();
                    }
                }
            }
        }

        if let Some(top_idx) = tokens.iter().position(|&t| t == "top") {
            if let Some(val) = tokens.get(top_idx + 1) {
                if let Ok(num) = val.parse::<u32>() {
                    limit_value = num.to_string();
                }
            }
        }

        if tables.is_empty() {
            tables.push("users".to_string());
        }

        let primary_table = tables.first().unwrap();
        if columns.is_empty() {
            if let Some(cols) = self.table_column_hints.get(primary_table) {
                if let Some(default_cols) = cols.get("id").or(cols.get(primary_table)) {
                    columns = default_cols.clone();
                }
            }
        }

        let mut sql = format!("SELECT {}", select_columns);
        sql.push_str(&format!(" FROM {}", primary_table));

        if tables.len() > 1 {
            if primary_table == "users" && tables.contains(&"orders".to_string()) {
                sql.push_str(" INNER JOIN orders ON users.id = orders.user_id");
            } else if primary_table == "orders" && tables.contains(&"products".to_string()) {
                sql.push_str(" INNER JOIN products ON orders.product_id = products.id");
            }
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        if has_aggregation && !columns.is_empty() && !order_column.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", columns.join(", ")));
        }

        if !order_column.is_empty() {
            sql.push_str(&format!(" ORDER BY {}", order_column));
            if !order_direction.is_empty() {
                sql.push_str(&format!(" {}", order_direction));
            }
        }

        if !limit_value.is_empty() {
            sql.push_str(&format!(" LIMIT {}", limit_value));
        }

        sql.push(';');

        let confidence = self.calculate_confidence(&query_lower, &tables, &conditions);

        NlQueryResponse {
            sql,
            confidence,
            table_hint: Some(primary_table.clone()),
            where_conditions: conditions,
            error: None,
        }
    }

    fn find_table_match(&self, token: &str) -> Option<String> {
        let tables = self.schema_service.list_tables();
        for table in tables {
            let table_lower = table.to_lowercase();
            if table_lower == token || table_lower == token.trim_end_matches('s') {
                return Some(table);
            }
            if table_lower.contains(token) || token.contains(&table_lower) {
                return Some(table);
            }
        }
        None
    }

    fn find_column_hints(&self, token: &str) -> Option<Vec<String>> {
        for (_, columns) in &self.table_column_hints {
            for (_, hints) in columns {
                for hint in hints {
                    if hint.contains(token) || token.contains(hint) {
                        return Some(hints.clone());
                    }
                }
            }
        }
        None
    }

    fn calculate_confidence(&self, query: &str, tables: &[String], conditions: &[String]) -> f32 {
        let mut confidence: f32 = 0.5;

        if !tables.is_empty() {
            confidence += 0.2;
        }

        if query.contains("select")
            || query.contains("show")
            || query.contains("list")
            || query.contains("get")
        {
            confidence += 0.1;
        }

        if !conditions.is_empty() {
            confidence += 0.1;
        }

        if query.contains("join") || query.contains("with") {
            confidence += 0.05;
        }

        if query.contains("order") || query.contains("sort") {
            confidence += 0.05;
        }

        if query.contains("limit") || query.contains("top") {
            confidence += 0.05;
        }

        confidence.min(1.0)
    }

    pub fn explain_sql(&self, sql: &str) -> SqlExplanation {
        let sql_upper = sql.to_uppercase();
        let mut tables_used = Vec::new();
        let columns_used = Vec::new();
        let mut operations = Vec::new();

        let tables = self.schema_service.list_tables();
        for table in tables {
            if sql_upper.contains(&table.to_uppercase()) {
                tables_used.push(table);
            }
        }

        if sql_upper.contains("WHERE") {
            operations.push("Filtering (WHERE)".to_string());
        }
        if sql_upper.contains("JOIN") {
            operations.push("Join operation".to_string());
        }
        if sql_upper.contains("GROUP BY") {
            operations.push("Grouping".to_string());
        }
        if sql_upper.contains("ORDER BY") {
            operations.push("Sorting".to_string());
        }
        if sql_upper.contains("LIMIT") {
            operations.push("Result limiting".to_string());
        }
        if sql_upper.contains("COUNT") || sql_upper.contains("SUM") || sql_upper.contains("AVG") {
            operations.push("Aggregation".to_string());
        }
        if sql_upper.contains("INSERT") {
            operations.push("Insert".to_string());
        }
        if sql_upper.contains("UPDATE") {
            operations.push("Update".to_string());
        }
        if sql_upper.contains("DELETE") {
            operations.push("Delete".to_string());
        }

        let explanation = if operations.is_empty() {
            "Simple SELECT query".to_string()
        } else {
            format!("Query performs: {}", operations.join(", "))
        };

        SqlExplanation {
            sql: sql.to_string(),
            explanation,
            tables_used,
            columns_used,
            operations,
        }
    }
}

impl Default for Nl2SqlService {
    fn default() -> Self {
        Self::new(Arc::new(crate::schema::SchemaService::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nl2sql_simple_select() {
        let service = Nl2SqlService::default();
        let result = service.natural_language_to_sql("show all users");

        assert!(result.sql.contains("SELECT"));
        assert!(result.sql.contains("FROM users"));
        assert_eq!(result.confidence > 0.5, true);
    }

    #[test]
    fn test_nl2sql_with_limit() {
        let service = Nl2SqlService::default();
        let result = service.natural_language_to_sql("show top 10 users");

        assert!(result.sql.contains("LIMIT 10"));
    }

    #[test]
    fn test_nl2sql_with_where() {
        let service = Nl2SqlService::default();
        let result = service.natural_language_to_sql("show users where active");

        assert!(result.sql.contains("WHERE"));
        assert!(result.where_conditions.len() > 0);
    }

    #[test]
    fn test_nl2sql_with_status() {
        let service = Nl2SqlService::default();
        let result = service.natural_language_to_sql("show orders where status is pending");

        assert!(result.sql.contains("WHERE"));
        assert!(result.sql.contains("pending"));
    }

    #[test]
    fn test_nl2sql_count() {
        let service = Nl2SqlService::default();
        let result = service.natural_language_to_sql("count all users");

        assert!(result.sql.contains("COUNT(*)"));
    }

    #[test]
    fn test_nl2sql_products() {
        let service = Nl2SqlService::default();
        let result = service.natural_language_to_sql("show all products");

        assert!(result.sql.contains("FROM products"));
    }

    #[test]
    fn test_nl2sql_orders() {
        let service = Nl2SqlService::default();
        let result = service.natural_language_to_sql("show all orders");

        assert!(result.sql.contains("FROM orders"));
    }

    #[test]
    fn test_explain_sql() {
        let service = Nl2SqlService::default();
        let explanation = service.explain_sql("SELECT * FROM users WHERE id = 1");

        assert!(explanation.explanation.contains("Filtering"));
        assert!(explanation.tables_used.contains(&"users".to_string()));
    }

    #[test]
    fn test_explain_sql_with_join() {
        let service = Nl2SqlService::default();
        let explanation =
            service.explain_sql("SELECT * FROM users JOIN orders ON users.id = orders.user_id");

        assert!(explanation.explanation.contains("Join"));
    }

    #[test]
    fn test_confidence_calculation() {
        let service = Nl2SqlService::default();

        let low_conf = service.natural_language_to_sql("test");
        let high_conf =
            service.natural_language_to_sql("show top 10 users where active order by id");

        assert!(high_conf.confidence > low_conf.confidence);
    }
}
