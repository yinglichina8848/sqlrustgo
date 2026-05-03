use rusqlite::Connection;
use sqlrustgo_parser::parse;
use sqlrustgo_parser::Statement;
use sqlrustgo_parser::{AggregateFunction, Expression, JoinType, SelectStatement};
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

type Row = Vec<String>;

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Boolean(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => {
            if f.is_nan() {
                "NaN".to_string()
            } else if f.is_infinite() {
                if f.is_sign_positive() {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
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
    } else if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"'))
    {
        Value::Text(s[1..s.len() - 1].to_string())
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
                let storage = self
                    .storage
                    .read()
                    .map_err(|e| format!("Lock error: {:?}", e))?;

                // Handle Cross Join (comma-separated FROM clause with multiple tables)
                if select.join_clause.is_none() {
                    if let Some(ref from) = select.from {
                        if from.tables.len() > 1 {
                            let table_names: Vec<String> =
                                from.tables.iter().map(|t| t.name.clone()).collect();
                            let mut all_table_infos: Vec<TableInfo> = Vec::new();
                            let mut all_table_rows: Vec<Vec<Vec<Value>>> = Vec::new();
                            for name in &table_names {
                                let info = storage
                                    .get_table_info(name)
                                    .map_err(|e| format!("Table error: {:?}", e))?;
                                let rows = storage
                                    .scan(name)
                                    .map_err(|e| format!("Scan error: {:?}", e))?;
                                all_table_infos.push(info);
                                all_table_rows.push(rows);
                            }

                            let mut combined_info = all_table_infos[0].clone();
                            for info in all_table_infos.iter().skip(1) {
                                for col in &info.columns {
                                    combined_info.columns.push(col.clone());
                                }
                            }

                            let result_rows = compute_cartesian_product(&all_table_rows);

                            if let Some(ref where_expr) = select.where_clause {
                                let filtered: Vec<Vec<Value>> = result_rows
                                    .into_iter()
                                    .filter(|row| {
                                        eval_predicate(where_expr, row, &combined_info, &storage)
                                    })
                                    .collect();
                                return project_rows(&select, &filtered, &combined_info);
                            }

                            return project_rows(&select, &result_rows, &combined_info);
                        }
                    }
                }

                // Handle JOIN
                if let Some(ref join) = select.join_clause {
                    let left_table = select.first_table();
                    let right_table = &join.table;

                    let left_info = storage
                        .get_table_info(&left_table)
                        .map_err(|e| format!("Table error: {:?}", e))?;
                    let right_info = storage
                        .get_table_info(right_table)
                        .map_err(|e| format!("Table error: {:?}", e))?;

                    let left_rows = storage
                        .scan(&left_table)
                        .map_err(|e| format!("Scan error: {:?}", e))?;
                    let right_rows = storage
                        .scan(right_table)
                        .map_err(|e| format!("Scan error: {:?}", e))?;

                    // Build combined schema for eval_join_expr
                    let mut combined_info = left_info.clone();
                    for col in &right_info.columns {
                        combined_info.columns.push(col.clone());
                    }

                    let mut result_rows: Vec<Vec<Value>> = match join.join_type {
                        JoinType::Inner => {
                            let mut rows = Vec::new();
                            for left_row in &left_rows {
                                for right_row in &right_rows {
                                    let mut combined = left_row.clone();
                                    combined.extend(right_row.clone());
                                    if eval_join_expr(
                                        &join.on_clause,
                                        &combined,
                                        &left_info,
                                        &right_info,
                                    ) {
                                        rows.push(combined);
                                    }
                                }
                            }
                            rows
                        }
                        JoinType::Left => {
                            let mut rows = Vec::new();
                            for left_row in &left_rows {
                                let mut matched = false;
                                for right_row in &right_rows {
                                    let mut combined = left_row.clone();
                                    combined.extend(right_row.clone());
                                    if eval_join_expr(
                                        &join.on_clause,
                                        &combined,
                                        &left_info,
                                        &right_info,
                                    ) {
                                        rows.push(combined);
                                        matched = true;
                                    }
                                }
                                if !matched {
                                    let mut combined = left_row.clone();
                                    combined.extend(
                                        vec![Value::Null; right_info.columns.len()].into_iter(),
                                    );
                                    rows.push(combined);
                                }
                            }
                            rows
                        }
                        _ => return Err(format!("Unsupported join type: {:?}", join.join_type)),
                    };

                    // Apply WHERE if present (after join)
                    if let Some(ref where_expr) = select.where_clause {
                        result_rows.retain(|row| {
                            eval_predicate(where_expr, row, &combined_info, &storage)
                        });
                    }

                    // Handle GROUP BY after JOIN
                    if !select.group_by.is_empty() {
                        let left_col_count = left_info.columns.len();
                        let group_indices: Vec<(usize, Option<usize>)> = select
                            .group_by
                            .iter()
                            .filter_map(|expr| {
                                if let Expression::Identifier(name) = expr {
                                    let (table_name, col_name) =
                                        if let Some(dot_pos) = name.find('.') {
                                            (Some(&name[..dot_pos]), &name[dot_pos + 1..])
                                        } else {
                                            (None, name.as_str())
                                        };
                                    if let Some(table) = table_name {
                                        if table == left_info.name {
                                            if let Some(idx) = left_info
                                                .columns
                                                .iter()
                                                .position(|c| &c.name == col_name)
                                            {
                                                return Some((idx, None));
                                            }
                                        } else {
                                            if let Some(idx) = right_info
                                                .columns
                                                .iter()
                                                .position(|c| &c.name == col_name)
                                            {
                                                return Some((left_col_count + idx, None));
                                            }
                                        }
                                        None
                                    } else {
                                        if let Some(idx) = combined_info
                                            .columns
                                            .iter()
                                            .position(|c| &c.name == col_name)
                                        {
                                            Some((idx, None))
                                        } else {
                                            None
                                        }
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();

                        let mut groups: std::collections::HashMap<Vec<String>, Vec<&Vec<Value>>> =
                            std::collections::HashMap::new();
                        for row in &result_rows {
                            let key: Vec<String> = group_indices
                                .iter()
                                .map(|(i, _)| value_to_string(row.get(*i).unwrap_or(&Value::Null)))
                                .collect();
                            groups.entry(key).or_default().push(row);
                        }

                        let mut rows_after_group: Vec<Vec<Value>> = groups
                            .into_iter()
                            .map(|(key, group_rows)| {
                                let mut result_row: Vec<Value> =
                                    key.into_iter().map(|s| literal_to_value(&s)).collect();
                                if !select.aggregates.is_empty() {
                                    for agg in &select.aggregates {
                                        let val = match agg.func {
                                            AggregateFunction::Count => {
                                                if agg.args.is_empty() {
                                                    Value::Integer(group_rows.len() as i64)
                                                } else if let Some(expr) = agg.args.first() {
                                                    if let Expression::Identifier(ref col_name) =
                                                        expr
                                                    {
                                                        let (table_name, col_only) =
                                                            if let Some(dot_pos) =
                                                                col_name.find('.')
                                                            {
                                                                (
                                                                    Some(&col_name[..dot_pos]),
                                                                    &col_name[dot_pos + 1..],
                                                                )
                                                            } else {
                                                                (None, col_name.as_str())
                                                            };
                                                        let idx = if let Some(table) = table_name {
                                                            if table == left_info.name {
                                                                left_info.columns.iter().position(
                                                                    |c| &c.name == col_only,
                                                                )
                                                            } else {
                                                                right_info.columns.iter().position(
                                                                    |c| &c.name == col_only,
                                                                )
                                                            }
                                                            .map(|i| {
                                                                if table == left_info.name {
                                                                    i
                                                                } else {
                                                                    left_col_count + i
                                                                }
                                                            })
                                                        } else {
                                                            combined_info
                                                                .columns
                                                                .iter()
                                                                .position(|c| &c.name == col_only)
                                                        };
                                                        if let Some(i) = idx {
                                                            let count = group_rows
                                                                .iter()
                                                                .filter(|r| {
                                                                    !matches!(
                                                                        r.get(i),
                                                                        Some(Value::Null)
                                                                    )
                                                                })
                                                                .count();
                                                            Value::Integer(count as i64)
                                                        } else {
                                                            Value::Integer(0)
                                                        }
                                                    } else {
                                                        Value::Null
                                                    }
                                                } else {
                                                    Value::Null
                                                }
                                            }
                                            AggregateFunction::Sum => {
                                                if let Some(expr) = agg.args.first() {
                                                    if let Expression::Identifier(ref col_name) =
                                                        expr
                                                    {
                                                        let (table_name, col_only) =
                                                            if let Some(dot_pos) =
                                                                col_name.find('.')
                                                            {
                                                                (
                                                                    Some(&col_name[..dot_pos]),
                                                                    &col_name[dot_pos + 1..],
                                                                )
                                                            } else {
                                                                (None, col_name.as_str())
                                                            };
                                                        let idx = if let Some(table) = table_name {
                                                            if table == left_info.name {
                                                                left_info.columns.iter().position(
                                                                    |c| &c.name == col_only,
                                                                )
                                                            } else {
                                                                right_info.columns.iter().position(
                                                                    |c| &c.name == col_only,
                                                                )
                                                            }
                                                            .map(|i| {
                                                                if table == left_info.name {
                                                                    i
                                                                } else {
                                                                    left_col_count + i
                                                                }
                                                            })
                                                        } else {
                                                            combined_info
                                                                .columns
                                                                .iter()
                                                                .position(|c| &c.name == col_only)
                                                        };
                                                        if let Some(i) = idx {
                                                            let sum: i64 = group_rows
                                                                .iter()
                                                                .filter_map(|r| r.get(i))
                                                                .filter_map(|v| {
                                                                    if let Value::Integer(i) = v {
                                                                        Some(*i)
                                                                    } else {
                                                                        None
                                                                    }
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
                                        result_row.push(val);
                                    }
                                }
                                result_row
                            })
                            .collect();
                        result_rows = rows_after_group;
                        if !select.aggregates.is_empty() {
                            return Ok(result_rows
                                .into_iter()
                                .map(|r| r.into_iter().map(|v| value_to_string(&v)).collect())
                                .collect());
                        }
                    } else if !select.aggregates.is_empty() {
                        let mut result_row: Vec<Value> = Vec::new();
                        for agg in &select.aggregates {
                            let val = match agg.func {
                                AggregateFunction::Count => {
                                    if agg.args.is_empty() {
                                        Value::Integer(result_rows.len() as i64)
                                    } else if let Some(expr) = agg.args.first() {
                                        if let Expression::Identifier(ref col_name) = expr {
                                            if let Some(idx) = combined_info
                                                .columns
                                                .iter()
                                                .position(|c| &c.name == col_name)
                                            {
                                                let count = result_rows
                                                    .iter()
                                                    .filter(|r| {
                                                        !matches!(r.get(idx), Some(Value::Null))
                                                    })
                                                    .count();
                                                Value::Integer(count as i64)
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
                                AggregateFunction::Sum => {
                                    if let Some(expr) = agg.args.first() {
                                        if let Expression::Identifier(ref col_name) = expr {
                                            if let Some(idx) = combined_info
                                                .columns
                                                .iter()
                                                .position(|c| &c.name == col_name)
                                            {
                                                let sum: i64 = result_rows
                                                    .iter()
                                                    .filter_map(|r| r.get(idx))
                                                    .filter_map(|v| {
                                                        if let Value::Integer(i) = v {
                                                            Some(*i)
                                                        } else {
                                                            None
                                                        }
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
                            result_row.push(val);
                        }
                        return Ok(vec![result_row
                            .into_iter()
                            .map(|v| value_to_string(&v))
                            .collect()]);
                    }

                    return project_rows(&select, &result_rows, &combined_info);
                }

                // Simple case without JOIN
                let table_name = select.first_table();
                let table_info = storage
                    .get_table_info(&table_name)
                    .map_err(|e| format!("Table error: {:?}", e))?;

                let mut rows = storage
                    .scan(&table_name)
                    .map_err(|e| format!("Scan error: {:?}", e))?;

                if let Some(ref where_expr) = select.where_clause {
                    rows.retain(|row| eval_predicate(where_expr, row, &table_info, &storage));
                }

                if !select.group_by.is_empty() {
                    let group_indices: Vec<usize> = select
                        .group_by
                        .iter()
                        .filter_map(|expr| {
                            if let Expression::Identifier(name) = expr {
                                table_info.columns.iter().position(|c| &c.name == name)
                            } else {
                                None
                            }
                        })
                        .collect();

                    let mut groups: std::collections::HashMap<Vec<String>, Vec<&Vec<Value>>> =
                        std::collections::HashMap::new();
                    for row in &rows {
                        let key: Vec<String> = group_indices
                            .iter()
                            .map(|&i| value_to_string(row.get(i).unwrap_or(&Value::Null)))
                            .collect();
                        groups.entry(key).or_default().push(row);
                    }

                    let mut result_rows: Vec<Row> = groups
                        .into_iter()
                        .map(|(key, group_rows)| {
                            let mut result_row: Row = key;
                            if !select.aggregates.is_empty() {
                                for agg in &select.aggregates {
                                    let val = match agg.func {
                                        AggregateFunction::Count => {
                                            if agg.args.is_empty() {
                                                Value::Integer(group_rows.len() as i64)
                                            } else if let Some(expr) = agg.args.first() {
                                                if let Expression::Identifier(ref col_name) = expr {
                                                    if let Some(idx) = table_info
                                                        .columns
                                                        .iter()
                                                        .position(|c| &c.name == col_name)
                                                    {
                                                        let count = group_rows
                                                            .iter()
                                                            .filter(|r| {
                                                                !matches!(
                                                                    r.get(idx),
                                                                    Some(Value::Null)
                                                                )
                                                            })
                                                            .count();
                                                        Value::Integer(count as i64)
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
                                        AggregateFunction::Sum => {
                                            if let Some(expr) = agg.args.first() {
                                                if let Expression::Identifier(ref col_name) = expr {
                                                    if let Some(idx) = table_info
                                                        .columns
                                                        .iter()
                                                        .position(|c| &c.name == col_name)
                                                    {
                                                        let sum: i64 = group_rows
                                                            .iter()
                                                            .filter_map(|r| r.get(idx))
                                                            .filter_map(|v| {
                                                                if let Value::Integer(i) = v {
                                                                    Some(*i)
                                                                } else {
                                                                    None
                                                                }
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
                            }
                            result_row
                        })
                        .collect();

                    if let Some(ref having_expr) = select.having {
                        result_rows
                            .retain(|row| eval_having(having_expr, row, &select.group_by, &select));
                    }

                    result_rows.sort();
                    return Ok(result_rows);
                }

                if !select.aggregates.is_empty() {
                    let mut result_rows = Vec::new();
                    let mut result_row = Vec::new();

                    for agg in &select.aggregates {
                        let val = match agg.func {
                            AggregateFunction::Count => {
                                if agg.args.is_empty() {
                                    Value::Integer(rows.len() as i64)
                                } else if let Some(expr) = agg.args.first() {
                                    if let Expression::Identifier(ref col_name) = expr {
                                        if let Some(idx) = table_info
                                            .columns
                                            .iter()
                                            .position(|c| &c.name == col_name)
                                        {
                                            let count = rows
                                                .iter()
                                                .filter(|r| {
                                                    !matches!(r.get(idx), Some(Value::Null))
                                                })
                                                .count();
                                            Value::Integer(count as i64)
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
                            AggregateFunction::Sum => {
                                if let Some(expr) = agg.args.first() {
                                    if let Expression::Identifier(ref col_name) = expr {
                                        if let Some(idx) = table_info
                                            .columns
                                            .iter()
                                            .position(|c| &c.name == col_name)
                                        {
                                            let sum: i64 = rows
                                                .iter()
                                                .filter_map(|r| r.get(idx))
                                                .filter_map(|v| {
                                                    if let Value::Integer(i) = v {
                                                        Some(*i)
                                                    } else {
                                                        None
                                                    }
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

                project_rows(&select, &rows, &table_info)
            }
            _ => Ok(vec![]),
        }
    }
}

fn eval_join_expr(
    expr: &Expression,
    row: &[Value],
    left_info: &TableInfo,
    right_info: &TableInfo,
) -> bool {
    match expr {
        Expression::BinaryOp(left, op, right) => {
            let op_upper = op.to_uppercase();
            match op_upper.as_str() {
                "AND" => {
                    eval_join_expr(left, row, left_info, right_info)
                        && eval_join_expr(right, row, left_info, right_info)
                }
                "OR" => {
                    eval_join_expr(left, row, left_info, right_info)
                        || eval_join_expr(right, row, left_info, right_info)
                }
                "=" | "==" => {
                    let left_val = eval_join_expr_val(left, row, left_info, right_info);
                    let right_val = eval_join_expr_val(right, row, left_info, right_info);
                    sql_eq(&left_val, &right_val)
                }
                _ => {
                    let left_val = eval_join_expr_val(left, row, left_info, right_info);
                    let right_val = eval_join_expr_val(right, row, left_info, right_info);
                    sql_compare(&op_upper, &left_val, &right_val)
                }
            }
        }
        Expression::IsNull(inner) => matches!(
            eval_join_expr_val(inner, row, left_info, right_info),
            Value::Null
        ),
        Expression::IsNotNull(inner) => !matches!(
            eval_join_expr_val(inner, row, left_info, right_info),
            Value::Null
        ),
        _ => {
            let val = eval_join_expr_val(expr, row, left_info, right_info);
            matches!(val, Value::Boolean(true))
        }
    }
}

fn eval_join_expr_val(
    expr: &Expression,
    row: &[Value],
    left_info: &TableInfo,
    right_info: &TableInfo,
) -> Value {
    match expr {
        Expression::Identifier(name) => {
            if let Some(dot_pos) = name.find('.') {
                let (table, col) = name.split_at(dot_pos);
                let col = &col[1..];
                if table == left_info.name {
                    if let Some(idx) = left_info.columns.iter().position(|c| &c.name == col) {
                        return row.get(idx).cloned().unwrap_or(Value::Null);
                    }
                } else if table == right_info.name {
                    let right_offset = left_info.columns.len();
                    if let Some(idx) = right_info.columns.iter().position(|c| &c.name == col) {
                        return row.get(right_offset + idx).cloned().unwrap_or(Value::Null);
                    }
                }
                Value::Null
            } else {
                if let Some(idx) = left_info.columns.iter().position(|c| &c.name == name) {
                    return row.get(idx).cloned().unwrap_or(Value::Null);
                }
                let right_offset = left_info.columns.len();
                if let Some(idx) = right_info.columns.iter().position(|c| &c.name == name) {
                    return row.get(right_offset + idx).cloned().unwrap_or(Value::Null);
                }
                Value::Null
            }
        }
        _ => literal_to_value(&format!("{:?}", expr)),
    }
}

fn sql_eq(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Null, Value::Null) => false,
        (Value::Null, _) => false,
        (_, Value::Null) => false,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < 1e-9,
        (Value::Text(a), Value::Text(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        _ => false,
    }
}

fn compute_cartesian_product(table_rows: &[Vec<Vec<Value>>]) -> Vec<Vec<Value>> {
    match table_rows {
        [] => vec![vec![]],
        [first, rest @ ..] => {
            let rest_result = compute_cartesian_product(rest);
            first
                .iter()
                .flat_map(|row| {
                    rest_result
                        .iter()
                        .map(|rest_row| {
                            let mut combined = row.clone();
                            combined.extend(rest_row.clone());
                            combined
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        }
    }
}

fn project_rows(
    select: &sqlrustgo_parser::SelectStatement,
    rows: &[Vec<Value>],
    table_info: &TableInfo,
) -> Result<Vec<Row>, String> {
    if select.columns.is_empty() || select.columns.iter().all(|c| c.name == "*") {
        return Ok(rows
            .iter()
            .map(|r| r.iter().map(value_to_string).collect())
            .collect());
    }

    Ok(rows
        .iter()
        .map(|r| {
            select
                .columns
                .iter()
                .map(|col| {
                    if col.name == "*" {
                        return "*".to_string();
                    }
                    if let Some(ref expr) = col.expression {
                        value_to_string(&eval_expr(expr, r, table_info))
                    } else {
                        value_to_string(
                            table_info
                                .columns
                                .iter()
                                .position(|c| &c.name == &col.name)
                                .and_then(|i| r.get(i))
                                .unwrap_or(&Value::Null),
                        )
                    }
                })
                .collect()
        })
        .collect())
}

impl Default for RustEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn eval_predicate(
    expr: &Expression,
    row: &[Value],
    table_info: &TableInfo,
    storage: &MemoryStorage,
) -> bool {
    match expr {
        Expression::BinaryOp(left, op, right) => {
            let op_upper = op.to_uppercase();
            match op_upper.as_str() {
                "AND" => {
                    let left_val = eval_expr(left, row, table_info);
                    if matches!(left_val, Value::Boolean(false)) {
                        return false;
                    }
                    let right_result = eval_predicate(right, row, table_info, storage);
                    match sql_and(left_val, Value::Boolean(right_result)) {
                        Value::Boolean(true) => true,
                        _ => false,
                    }
                }
                "OR" => {
                    let left_val = eval_expr(left, row, table_info);
                    if matches!(left_val, Value::Boolean(true)) {
                        return true;
                    }
                    let right_result = eval_predicate(right, row, table_info, storage);
                    match sql_or(left_val, Value::Boolean(right_result)) {
                        Value::Boolean(true) => true,
                        _ => false,
                    }
                }
                _ => {
                    let left_val = eval_expr(left, row, table_info);
                    let right_val = eval_expr(right, row, table_info);
                    match cmp_op(&op_upper, &left_val, &right_val) {
                        Value::Boolean(true) => true,
                        _ => false,
                    }
                }
            }
        }
        Expression::IsNull(inner) => {
            matches!(eval_expr(inner, row, table_info), Value::Null)
        }
        Expression::IsNotNull(inner) => !matches!(eval_expr(inner, row, table_info), Value::Null),
        Expression::UnaryOp(op, inner) => {
            if op.to_uppercase() == "NOT" {
                let val = eval_expr(inner, row, table_info);
                match sql_not(val) {
                    Value::Boolean(true) => true,
                    _ => false,
                }
            } else {
                false
            }
        }
        Expression::In(left, subquery) => {
            let left_val = eval_expr(left, row, table_info);
            let subquery_result = eval_subquery(subquery, storage);
            let result = eval_in_subquery(&left_val, &subquery_result, false);
            matches!(result, Value::Boolean(true))
        }
        Expression::NotIn(left, subquery) => {
            let left_val = eval_expr(left, row, table_info);
            let subquery_result = eval_subquery(subquery, storage);
            let result = eval_in_subquery(&left_val, &subquery_result, true);
            matches!(result, Value::Boolean(true))
        }
        Expression::InList(left, values) => {
            let left_val = eval_expr(left, row, table_info);
            let result = eval_in_list(&left_val, values, false);
            matches!(result, Value::Boolean(true))
        }
        Expression::NotInList(left, values) => {
            let left_val = eval_expr(left, row, table_info);
            let result = eval_in_list(&left_val, values, true);
            matches!(result, Value::Boolean(true))
        }
        Expression::Exists(subquery) => {
            let subquery_result = eval_subquery(subquery, storage);
            matches!(eval_exists(&subquery_result, false), Value::Boolean(true))
        }
        Expression::NotExists(subquery) => {
            let subquery_result = eval_subquery(subquery, storage);
            matches!(eval_exists(&subquery_result, true), Value::Boolean(true))
        }
        _ => {
            let val = eval_expr(expr, row, table_info);
            matches!(val, Value::Boolean(true))
        }
    }
}

fn eval_subquery(subquery: &SelectStatement, storage: &MemoryStorage) -> Vec<Vec<Value>> {
    let mut results = Vec::new();
    let table_name = subquery.first_table();
    if let Ok(table_info) = storage.get_table_info(&table_name) {
        if let Ok(mut rows) = storage.scan(&table_name) {
            if let Some(ref where_clause) = subquery.where_clause {
                rows.retain(|row| eval_predicate(where_clause, row, &table_info, storage));
            }
            results = rows;
        }
    }
    results
}

fn eval_in_subquery(expr: &Value, subquery_result: &[Vec<Value>], negated: bool) -> Value {
    let mut has_null = false;
    let found = subquery_result.iter().any(|row| {
        if row.is_empty() {
            return false;
        }
        let v = &row[0];
        match v {
            Value::Null => {
                has_null = true;
                false
            }
            _ => {
                if matches!(expr, Value::Null) {
                    has_null = true;
                    return false;
                }
                let expr_str = value_to_string(expr);
                let v_str = value_to_string(v);
                expr_str == v_str
            }
        }
    });

    if found {
        Value::Boolean(!negated)
    } else if has_null {
        Value::Null
    } else {
        Value::Boolean(negated)
    }
}

fn eval_in_list(expr: &Value, values: &[Expression], negated: bool) -> Value {
    let mut has_null = false;
    let found = values.iter().any(|val_expr| {
        let v = eval_expr(val_expr, &[], &TableInfo::default());
        match &v {
            Value::Null => {
                has_null = true;
                false
            }
            _ => {
                if matches!(expr, Value::Null) {
                    has_null = true;
                    return false;
                }
                let expr_str = value_to_string(expr);
                let v_str = value_to_string(&v);
                expr_str == v_str
            }
        }
    });

    if found {
        Value::Boolean(!negated)
    } else if has_null {
        Value::Null
    } else {
        Value::Boolean(negated)
    }
}

fn eval_exists(subquery_result: &[Vec<Value>], negated: bool) -> Value {
    let exists = !subquery_result.is_empty();
    Value::Boolean(if negated { !exists } else { exists })
}

/// Compare two Values using operator, returning Value (for three-valued logic)
fn cmp_op(op: &str, left: &Value, right: &Value) -> Value {
    match op {
        "=" | "==" => cmp_eq(left, right),
        "!=" | "<>" => cmp_ne(left, right),
        ">" => cmp_gt(left, right),
        ">=" => cmp_ge(left, right),
        "<" => cmp_lt(left, right),
        "<=" => cmp_le(left, right),
        _ => Value::Null,
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
                "+" => arithmetic(left_val, right_val, |x, y| x + y),
                "-" => arithmetic(left_val, right_val, |x, y| x - y),
                "*" => arithmetic(left_val, right_val, |x, y| x * y),
                "/" => {
                    if let (Some(x), Some(y)) = (left_val.to_number(), right_val.to_number()) {
                        if y != 0.0 {
                            Value::Float(x / y)
                        } else {
                            Value::Null
                        }
                    } else {
                        Value::Null
                    }
                }
                "=" | "==" => cmp_eq(&left_val, &right_val),
                "!=" | "<>" => cmp_ne(&left_val, &right_val),
                ">" => cmp_gt(&left_val, &right_val),
                ">=" => cmp_ge(&left_val, &right_val),
                "<" => cmp_lt(&left_val, &right_val),
                "<=" => cmp_le(&left_val, &right_val),
                _ => Value::Null,
            }
        }
        Expression::IsNull(inner) => {
            let val = eval_expr(inner, row, table_info);
            Value::Boolean(matches!(val, Value::Null))
        }
        Expression::IsNotNull(inner) => {
            let val = eval_expr(inner, row, table_info);
            Value::Boolean(!matches!(val, Value::Null))
        }
        _ => Value::Null,
    }
}

fn sql_compare(op: &str, left: &Value, right: &Value) -> bool {
    match op {
        "=" | "==" => match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Null, _) => false,
            (_, Value::Null) => false,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < 1e-9,
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            _ => false,
        },
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

/// SQL AND with three-valued logic
/// FALSE AND xxx = FALSE (short-circuit, don't evaluate xxx)
/// NULL AND TRUE = NULL
/// TRUE AND TRUE = TRUE
pub fn sql_and(a: Value, b: Value) -> Value {
    match (&a, &b) {
        (Value::Boolean(false), _) | (_, Value::Boolean(false)) => Value::Boolean(false),
        (Value::Null, _) | (_, Value::Null) => Value::Null,
        (Value::Boolean(true), Value::Boolean(true)) => Value::Boolean(true),
        _ => Value::Null,
    }
}

/// SQL OR with three-valued logic
/// TRUE OR xxx = TRUE (short-circuit, don't evaluate xxx)
/// NULL OR FALSE = NULL
/// FALSE OR FALSE = FALSE
pub fn sql_or(a: Value, b: Value) -> Value {
    match (&a, &b) {
        (Value::Boolean(true), _) | (_, Value::Boolean(true)) => Value::Boolean(true),
        (Value::Null, _) | (_, Value::Null) => Value::Null,
        (Value::Boolean(false), Value::Boolean(false)) => Value::Boolean(false),
        _ => Value::Null,
    }
}

/// SQL NOT with three-valued logic
pub fn sql_not(v: Value) -> Value {
    match v {
        Value::Boolean(b) => Value::Boolean(!b),
        Value::Null => Value::Null,
        _ => Value::Null,
    }
}

/// Compare equality with three-valued logic (returns Value)
fn cmp_eq(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Null, _) | (_, Value::Null) => Value::Null,
        _ => Value::Boolean(a == b),
    }
}

/// Compare greater than with three-valued logic (returns Value)
fn cmp_gt(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Null, _) | (_, Value::Null) => Value::Null,
        _ => Value::Boolean(cmp_numbers(a, b) == Some(std::cmp::Ordering::Greater)),
    }
}

/// Compare less than with three-valued logic (returns Value)
fn cmp_lt(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Null, _) | (_, Value::Null) => Value::Null,
        _ => Value::Boolean(cmp_numbers(a, b) == Some(std::cmp::Ordering::Less)),
    }
}

/// Compare greater than or equal with three-valued logic (returns Value)
fn cmp_ge(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Null, _) | (_, Value::Null) => Value::Null,
        _ => {
            let ord = cmp_numbers(a, b);
            Value::Boolean(
                ord == Some(std::cmp::Ordering::Greater) || ord == Some(std::cmp::Ordering::Equal),
            )
        }
    }
}

/// Compare less than or equal with three-valued logic (returns Value)
fn cmp_le(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Null, _) | (_, Value::Null) => Value::Null,
        _ => {
            let ord = cmp_numbers(a, b);
            Value::Boolean(
                ord == Some(std::cmp::Ordering::Less) || ord == Some(std::cmp::Ordering::Equal),
            )
        }
    }
}

/// Compare not equal with three-valued logic (returns Value)
fn cmp_ne(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Null, _) | (_, Value::Null) => Value::Null,
        _ => Value::Boolean(cmp_numbers(a, b) != Some(std::cmp::Ordering::Equal)),
    }
}

/// Compare two Values numerically, returning Ordering
fn cmp_numbers(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Integer(a_i), Value::Integer(b_i)) => Some(a_i.cmp(b_i)),
        (Value::Float(a_f), Value::Float(b_f)) => a_f.partial_cmp(b_f),
        (Value::Integer(a_i), Value::Float(b_f)) => (*a_i as f64).partial_cmp(b_f),
        (Value::Float(a_f), Value::Integer(b_i)) => a_f.partial_cmp(&(*b_i as f64)),
        (Value::Text(a_s), Value::Text(b_s)) => Some(a_s.cmp(b_s)),
        (Value::Boolean(a_b), Value::Boolean(b_b)) => Some(a_b.cmp(b_b)),
        _ => None,
    }
}

/// Arithmetic operation helper using to_number
fn arithmetic<F>(a: Value, b: Value, f: F) -> Value
where
    F: Fn(f64, f64) -> f64,
{
    use sqlrustgo_types::Value;
    match (a.to_number(), b.to_number()) {
        (Some(x), Some(y)) => Value::Float(f(x, y)),
        _ => Value::Null,
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

/// Evaluate HAVING expression on aggregated result rows.
/// row_idxs maps column names to their indices in the result row.
fn eval_having(
    expr: &Expression,
    row: &[String],
    group_by: &[Expression],
    _select: &SelectStatement,
) -> bool {
    match expr {
        Expression::IsNull(inner) => {
            let val = eval_having_expr(inner, row, group_by, _select);
            val == "NULL" || val.is_empty()
        }
        Expression::IsNotNull(inner) => {
            let val = eval_having_expr(inner, row, group_by, _select);
            val != "NULL" && !val.is_empty()
        }
        Expression::BinaryOp(left, op, right) => {
            let op_upper = op.to_uppercase();
            match op_upper.as_str() {
                "AND" => {
                    eval_having(left, row, group_by, _select)
                        && eval_having(right, row, group_by, _select)
                }
                "OR" => {
                    eval_having(left, row, group_by, _select)
                        || eval_having(right, row, group_by, _select)
                }
                _ => {
                    let left_val = eval_having_expr(left, row, group_by, _select);
                    let right_val = eval_having_expr(right, row, group_by, _select);
                    let left_parsed = parse_for_compare(&left_val);
                    let right_parsed = parse_for_compare(&right_val);
                    sql_compare_having(&op_upper, &left_parsed, &right_parsed)
                }
            }
        }
        Expression::UnaryOp(op, inner) => {
            if op.to_uppercase() == "NOT" {
                !eval_having(inner, row, group_by, _select)
            } else {
                false
            }
        }
        _ => {
            let val = eval_having_expr(expr, row, group_by, _select);
            val != "NULL" && !val.is_empty()
        }
    }
}

fn eval_having_expr(
    expr: &Expression,
    row: &[String],
    group_by: &[Expression],
    select: &SelectStatement,
) -> String {
    match expr {
        Expression::Identifier(name) => {
            // First check if it's a GROUP BY column
            if let Some(idx) = group_by.iter().position(|g| {
                if let Expression::Identifier(g_name) = g {
                    g_name == name
                } else {
                    false
                }
            }) {
                return row.get(idx).cloned().unwrap_or_else(|| "NULL".to_string());
            }
            // Otherwise check if it's an aggregate (COUNT(*), SUM(col), etc.)
            // Aggregates appear after group by columns in the result row
            let group_by_count = group_by.len();
            for (i, agg) in select.aggregates.iter().enumerate() {
                let agg_name = format!(
                    "{}({})",
                    format!("{:?}", agg.func).to_uppercase(),
                    agg.args
                        .iter()
                        .map(|a| {
                            if let Expression::Identifier(id) = a {
                                id.clone()
                            } else {
                                "*".to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                );
                if agg_name.to_uppercase().contains(&name.to_uppercase())
                    || (name == "*" && matches!(agg.func, AggregateFunction::Count))
                {
                    return row
                        .get(group_by_count + i)
                        .cloned()
                        .unwrap_or_else(|| "NULL".to_string());
                }
            }
            // For COUNT(*) just return the aggregate result
            if name == "*" {
                return row
                    .get(group_by_count)
                    .cloned()
                    .unwrap_or_else(|| "0".to_string());
            }
            "NULL".to_string()
        }
        Expression::Literal(s) => s.clone(),
        _ => "NULL".to_string(),
    }
}

fn parse_for_compare(s: &str) -> Value {
    if s == "NULL" || s.is_empty() {
        return Value::Null;
    }
    if let Ok(i) = s.parse::<i64>() {
        return Value::Integer(i);
    }
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }
    Value::Text(s.to_string())
}

fn sql_compare_having(op: &str, left: &Value, right: &Value) -> bool {
    match op {
        "=" | "==" => match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Null, _) => false,
            (_, Value::Null) => false,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < 1e-9,
            (Value::Integer(a), Value::Float(b)) => (*a as f64 - b).abs() < 1e-9,
            (Value::Float(a), Value::Integer(b)) => (a - *b as f64).abs() < 1e-9,
            (Value::Text(a), Value::Text(b)) => a == b,
            _ => false,
        },
        "!=" | "<>" => !sql_compare_having("=", left, right),
        ">" => match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => *a > *b,
            (Value::Float(a), Value::Float(b)) => *a > *b,
            (Value::Integer(a), Value::Float(b)) => (*a as f64) > *b,
            (Value::Float(a), Value::Integer(b)) => *a > (*b as f64),
            _ => false,
        },
        ">=" => sql_compare_having(">", left, right) || sql_compare_having("=", left, right),
        "<" => match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => *a < *b,
            (Value::Float(a), Value::Float(b)) => *a < *b,
            (Value::Integer(a), Value::Float(b)) => (*a as f64) < *b,
            (Value::Float(a), Value::Integer(b)) => *a < (*b as f64),
            _ => false,
        },
        "<=" => sql_compare_having("<", left, right) || sql_compare_having("=", left, right),
        _ => false,
    }
}
