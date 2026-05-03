use rusqlite::Connection;
use sqlrustgo_parser::parse;
use sqlrustgo_parser::Statement;
use sqlrustgo_parser::{Expression, SelectColumn, AggregateFunction};
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

type Row = Vec<String>;

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Boolean(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => {
            if f.is_nan() {
                "NaN".to_string()
            } else if f.is_infinite() {
                if f.is_sign_positive() { "Infinity".to_string() } else { "-Infinity".to_string() }
            } else {
                f.to_string()
            }
        }
        Value::Text(s) => s.clone(),
        Value::Blob(b) => format!("{:?}", b),
    }
}

fn literal_to_value(s: &str) -> Value {
    let s = s.trim();
    if s.eq_ignore_ascii_case("NULL") {
        Value::Null
    } else if let Ok(i) = s.parse::<i64>() {
        Value::Integer(i)
    } else if let Ok(f) = s.parse::<f64>() {
        Value::Float(f)
    } else if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        Value::Text(s[1..s.len()-1].to_string())
    } else if s.eq_ignore_ascii_case("TRUE") {
        Value::Boolean(true)
    } else if s.eq_ignore_ascii_case("FALSE") {
        Value::Boolean(false)
    } else {
        Value::Text(s.to_string())
    }
}

pub struct SqliteEngine {
    conn: Connection,
}

impl SqliteEngine {
    pub fn new() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        Self { conn }
    }

    pub fn execute(&self, sql: &str) -> Result<(), String> {
        self.conn
            .execute(sql, [])
            .map_err(|e| format!("SQLite error: {}", e))?;
        Ok(())
    }

    pub fn query(&self, sql: &str) -> Result<Vec<Row>, String> {
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| format!("Prepare error: {}", e))?;

        let column_count = stmt.column_count();
        let rows = stmt
            .query_map([], move |row| {
                let mut r = Vec::new();
                for i in 0..column_count {
                    let val: rusqlite::Result<rusqlite::types::Value> = row.get(i);
                    let s = match val {
                        Ok(rusqlite::types::Value::Null) => "NULL".to_string(),
                        Ok(rusqlite::types::Value::Integer(i)) => i.to_string(),
                        Ok(rusqlite::types::Value::Real(f)) => f.to_string(),
                        Ok(rusqlite::types::Value::Text(s)) => s,
                        Ok(rusqlite::types::Value::Blob(b)) => format!("{:?}", b),
                        Err(_) => "NULL".to_string(),
                    };
                    r.push(s);
                }
                Ok(r)
            })
            .map_err(|e| format!("Query error: {}", e))?;

        rows.map(|r| r.map_err(|e| format!("Row error: {}", e)))
            .collect()
    }
}

impl Default for SqliteEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RustEngine {
    storage: Arc<RwLock<MemoryStorage>>,
}

impl RustEngine {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
        }
    }

    pub fn execute(&mut self, sql: &str) -> Result<(), String> {
        let statement = parse(sql).map_err(|e| format!("Parse error: {:?}", e))?;

        match statement {
            Statement::CreateTable(create) => {
                let info = TableInfo {
                    name: create.name.clone(),
                    columns: create
                        .columns
                        .into_iter()
                        .map(|c| ColumnDefinition {
                            name: c.name,
                            data_type: c.data_type,
                            nullable: c.nullable,
                            primary_key: c.primary_key,
                        })
                        .collect(),
                    foreign_keys: vec![],
                    unique_constraints: vec![],
                    check_constraints: vec![],
                    partition_info: None,
                };
                self.storage
                    .write()
                    .map_err(|e| format!("Lock error: {:?}", e))?
                    .create_table(&info)
                    .map_err(|e| format!("Create table error: {:?}", e))?;
                Ok(())
            }
            Statement::DropTable(drop) => {
                self.storage
                    .write()
                    .map_err(|e| format!("Lock error: {:?}", e))?
                    .drop_table(&drop.name)
                    .map_err(|e| format!("Drop table error: {:?}", e))?;
                Ok(())
            }
            Statement::Insert(insert) => {
                let mut records: Vec<Vec<Value>> = Vec::new();
                for row_exprs in &insert.values {
                    let mut record = Vec::new();
                    for expr in row_exprs {
                        if let Expression::Literal(s) = expr {
                            record.push(literal_to_value(s));
                        } else {
                            return Err(format!("Cannot convert expression to value: {:?}", expr));
                        }
                    }
                    records.push(record);
                }
                self.storage
                    .write()
                    .map_err(|e| format!("Lock error: {:?}", e))?
                    .insert(&insert.table, records)
                    .map_err(|e| format!("Insert error: {:?}", e))?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn query(&self, sql: &str) -> Result<Vec<Row>, String> {
        let statement = parse(sql).map_err(|e| format!("Parse error: {:?}", e))?;

        match statement {
            Statement::Select(select) => {
                let storage = self.storage.read().map_err(|e| format!("Lock error: {:?}", e))?;
                let table_name = select.first_table();

                let table_info = storage
                    .get_table_info(&table_name)
                    .map_err(|e| format!("Table error: {:?}", e))?;

                let mut rows = storage
                    .scan(&table_name)
                    .map_err(|e| format!("Scan error: {:?}", e))?;

                if let Some(ref where_expr) = select.where_clause {
                    rows.retain(|row| eval_predicate(where_expr, row, &table_info));
                }

                if !select.aggregates.is_empty() {
                    let mut result_rows = Vec::new();
                    let mut result_row = Vec::new();

                    for agg in &select.aggregates {
                        let val = match agg.func {
                            AggregateFunction::Count => Value::Integer(rows.len() as i64),
                            AggregateFunction::Sum => {
                                if let Some(expr) = agg.args.first() {
                                    if let Expression::Identifier(ref col_name) = expr {
                                        if let Some(idx) = table_info.columns.iter().position(|c| &c.name == col_name) {
                                            let sum: i64 = rows.iter()
                                                .filter_map(|r| r.get(idx))
                                                .filter_map(|v| {
                                                    if let Value::Integer(i) = v { Some(*i) } else { None }
                                                })
                                                .sum();
                                            Value::Integer(sum)
                                        } else {
                                            Value::Null
                                        }
                                    } else {
                                        Value::Null
                                    }
                                } else {
                                    Value::Null
                                }
                            }
                            _ => Value::Null,
                        };
                        result_row.push(value_to_string(&val));
                    }

                    result_rows.push(result_row);
                    return Ok(result_rows);
                }

                if !select.group_by.is_empty() {
                    let group_indices: Vec<usize> = select.group_by.iter().filter_map(|expr| {
                        if let Expression::Identifier(name) = expr {
                            table_info.columns.iter().position(|c| &c.name == name)
                        } else {
                            None
                        }
                    }).collect();

                    let mut groups: std::collections::HashMap<Vec<String>, Vec<&Vec<Value>>> = std::collections::HashMap::new();
                    for row in &rows {
                        let key: Vec<String> = group_indices.iter()
                            .map(|&i| value_to_string(row.get(i).unwrap_or(&Value::Null)))
                            .collect();
                        groups.entry(key).or_default().push(row);
                    }

                    let mut result_rows: Vec<Row> = groups.into_iter().map(|(key, group_rows)| {
                        let mut result_row: Row = key;
                        for agg in &select.aggregates {
                            let val = match agg.func {
                                AggregateFunction::Count => Value::Integer(group_rows.len() as i64),
                                AggregateFunction::Sum => {
                                    if let Some(expr) = agg.args.first() {
                                        if let Expression::Identifier(ref col_name) = expr {
                                            if let Some(idx) = table_info.columns.iter().position(|c| &c.name == col_name) {
                                                let sum: i64 = group_rows.iter()
                                                    .filter_map(|r| r.get(idx))
                                                    .filter_map(|v| {
                                                        if let Value::Integer(i) = v { Some(*i) } else { None }
                                                    })
                                                    .sum();
                                                Value::Integer(sum)
                                            } else {
                                                Value::Null
                                            }
                                        } else {
                                            Value::Null
                                        }
                                    } else {
                                        Value::Null
                                    }
                                }
                                _ => Value::Null,
                            };
                            result_row.push(value_to_string(&val));
                        }
                        result_row
                    }).collect();

                    result_rows.sort();
                    return Ok(result_rows);
                }

                if select.columns.is_empty() || select.columns.iter().all(|c| c.name == "*") {
                    return Ok(rows.iter().map(|r| r.iter().map(value_to_string).collect()).collect());
                }

                let col_indices: Vec<usize> = select.columns.iter().filter_map(|c| {
                    if c.name == "*" {
                        None
                    } else {
                        table_info.columns.iter().position(|col| &col.name == &c.name)
                    }
                }).collect();

                Ok(rows.iter().map(|r| {
                    col_indices.iter().map(|&i| value_to_string(r.get(i).unwrap_or(&Value::Null))).collect()
                }).collect())
            }
            _ => Ok(vec![]),
        }
    }
}

impl Default for RustEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn eval_predicate(expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
    match expr {
        Expression::BinaryOp(left, op, right) => {
            let op_upper = op.to_uppercase();
            match op_upper.as_str() {
                "AND" => eval_predicate(left, row, table_info) && eval_predicate(right, row, table_info),
                "OR" => eval_predicate(left, row, table_info) || eval_predicate(right, row, table_info),
                _ => {
                    let left_val = eval_expr(left, row, table_info);
                    let right_val = eval_expr(right, row, table_info);
                    sql_compare(&op_upper, &left_val, &right_val)
                }
            }
        }
        Expression::IsNull(inner) => {
            matches!(eval_expr(inner, row, table_info), Value::Null)
        }
        Expression::IsNotNull(inner) => {
            !matches!(eval_expr(inner, row, table_info), Value::Null)
        }
        Expression::UnaryOp(op, inner) => {
            if op.to_uppercase() == "NOT" {
                !eval_predicate(inner, row, table_info)
            } else {
                false
            }
        }
        _ => {
            let val = eval_expr(expr, row, table_info);
            matches!(val, Value::Boolean(true))
        }
    }
}

fn eval_expr(expr: &Expression, row: &[Value], table_info: &TableInfo) -> Value {
    match expr {
        Expression::Literal(s) => literal_to_value(s),
        Expression::Identifier(name) => {
            if let Some(idx) = table_info.columns.iter().position(|c| &c.name == name) {
                row.get(idx).cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        Expression::BinaryOp(left, op, right) => {
            let left_val = eval_expr(left, row, table_info);
            let right_val = eval_expr(right, row, table_info);
            let op = op.to_uppercase();
            match op.as_str() {
                "+" => {
                    if let (Value::Integer(a), Value::Integer(b)) = (&left_val, &right_val) {
                        Value::Integer(a + b)
                    } else if let (Value::Float(a), Value::Float(b)) = (&left_val, &right_val) {
                        Value::Float(a + b)
                    } else if let (Value::Integer(a), Value::Float(b)) = (&left_val, &right_val) {
                        Value::Float(*a as f64 + b)
                    } else if let (Value::Float(a), Value::Integer(b)) = (&left_val, &right_val) {
                        Value::Float(a + *b as f64)
                    } else {
                        Value::Null
                    }
                }
                "-" => {
                    if let (Value::Integer(a), Value::Integer(b)) = (&left_val, &right_val) {
                        Value::Integer(a - b)
                    } else if let (Value::Float(a), Value::Float(b)) = (&left_val, &right_val) {
                        Value::Float(a - b)
                    } else if let (Value::Integer(a), Value::Float(b)) = (&left_val, &right_val) {
                        Value::Float(*a as f64 - b)
                    } else if let (Value::Float(a), Value::Integer(b)) = (&left_val, &right_val) {
                        Value::Float(a - *b as f64)
                    } else {
                        Value::Null
                    }
                }
                "*" => {
                    if let (Value::Integer(a), Value::Integer(b)) = (&left_val, &right_val) {
                        Value::Integer(a * b)
                    } else if let (Value::Float(a), Value::Float(b)) = (&left_val, &right_val) {
                        Value::Float(a * b)
                    } else {
                        Value::Null
                    }
                }
                "/" => {
                    if let (Value::Integer(a), Value::Integer(b)) = (&left_val, &right_val) {
                        if *b != 0 {
                            Value::Integer(a / b)
                        } else {
                            Value::Null
                        }
                    } else if let (Value::Float(a), Value::Float(b)) = (&left_val, &right_val) {
                        if *b != 0.0 {
                            Value::Float(a / b)
                        } else {
                            Value::Null
                        }
                    } else {
                        Value::Null
                    }
                }
                _ => Value::Null,
            }
        }
        _ => Value::Null,
    }
}

fn sql_compare(op: &str, left: &Value, right: &Value) -> bool {
    match op {
        "=" | "==" => {
            match (left, right) {
                (Value::Null, Value::Null) => true,
                (Value::Null, _) => false,
                (_, Value::Null) => false,
                (Value::Integer(a), Value::Integer(b)) => a == b,
                (Value::Float(a), Value::Float(b)) => (a - b).abs() < 1e-9,
                (Value::Text(a), Value::Text(b)) => a == b,
                (Value::Boolean(a), Value::Boolean(b)) => a == b,
                _ => false,
            }
        }
        "!=" | "<>" => !sql_compare("=", left, right),
        ">" => {
            let result = match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => *a > *b,
                (Value::Float(a), Value::Float(b)) => *a > *b,
                (Value::Integer(a), Value::Float(b)) => (*a as f64) > *b,
                (Value::Float(a), Value::Integer(b)) => *a > (*b as f64),
                (Value::Text(a), Value::Text(b)) => a > b,
                _ => false,
            };
            result
        }
        ">=" => {
            let result = match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => *a >= *b,
                (Value::Float(a), Value::Float(b)) => *a >= *b,
                (Value::Integer(a), Value::Float(b)) => (*a as f64) >= *b,
                (Value::Float(a), Value::Integer(b)) => *a >= (*b as f64),
                (Value::Text(a), Value::Text(b)) => a >= b,
                _ => false,
            };
            result
        }
        "<" => {
            let result = match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => *a < *b,
                (Value::Float(a), Value::Float(b)) => *a < *b,
                (Value::Integer(a), Value::Float(b)) => (*a as f64) < *b,
                (Value::Float(a), Value::Integer(b)) => *a < (*b as f64),
                (Value::Text(a), Value::Text(b)) => a < b,
                _ => false,
            };
            result
        }
        "<=" => {
            let result = match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => *a <= *b,
                (Value::Float(a), Value::Float(b)) => *a <= *b,
                (Value::Integer(a), Value::Float(b)) => (*a as f64) <= *b,
                (Value::Float(a), Value::Integer(b)) => *a <= (*b as f64),
                (Value::Text(a), Value::Text(b)) => a <= b,
                _ => false,
            };
            result
        }
        _ => false,
    }
}

fn normalize(rows: &mut Vec<Row>) {
    for row in rows.iter_mut() {
        for val in row.iter_mut() {
            if val.is_empty() {
                *val = "NULL".to_string();
            }
        }
    }
    rows.sort();
}

pub fn assert_sql_eq(sql: &str, setup: &[&str]) -> Result<(), String> {
    let mut sqlite = SqliteEngine::new();
    let mut rust = RustEngine::new();

    for s in setup {
        sqlite.execute(s).map_err(|e| e.to_string())?;
        rust.execute(s).map_err(|e| e.to_string())?;
    }

    let mut left = sqlite.query(sql).map_err(|e| e.to_string())?;
    let mut right = rust.query(sql).map_err(|e| e.to_string())?;

    normalize(&mut left);
    normalize(&mut right);

    if left != right {
        return Err(format!(
            "SQL mismatch: {}\nSQLite: {:?}\nRust: {:?}",
            sql, left, right
        ));
    }
    Ok(())
}

pub fn assert_query_eq(left: Vec<Row>, right: Vec<Row>) -> Result<(), String> {
    let mut left_norm = left;
    let mut right_norm = right;

    normalize(&mut left_norm);
    normalize(&mut right_norm);

    if left_norm != right_norm {
        return Err(format!(
            "Query mismatch:\nExpected: {:?}\nGot: {:?}",
            left_norm, right_norm
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_basic_query() {
        let sqlite = SqliteEngine::new();
        sqlite.execute("CREATE TABLE t(a INT)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (1)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (2)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (3)").unwrap();

        let result = sqlite.query("SELECT * FROM t").unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_sqlite_count() {
        let sqlite = SqliteEngine::new();
        sqlite.execute("CREATE TABLE t(a INT)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (1)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (2)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (3)").unwrap();

        let result = sqlite.query("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(result[0][0], "3");
    }

    #[test]
    fn test_sqlite_where() {
        let sqlite = SqliteEngine::new();
        sqlite.execute("CREATE TABLE t(a INT)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (1)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (2)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (3)").unwrap();

        let result = sqlite.query("SELECT a FROM t WHERE a > 1").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_sqlite_aggregate() {
        let sqlite = SqliteEngine::new();
        sqlite.execute("CREATE TABLE orders(amount INT)").unwrap();
        sqlite.execute("INSERT INTO orders VALUES (100)").unwrap();
        sqlite.execute("INSERT INTO orders VALUES (200)").unwrap();
        sqlite.execute("INSERT INTO orders VALUES (150)").unwrap();

        let result = sqlite.query("SELECT SUM(amount) FROM orders").unwrap();
        assert_eq!(result[0][0], "450");
    }
}
