use super::ProcedureContext;
use super::StoredProcExecutor;
use sqlrustgo_types::Value;

impl StoredProcExecutor {
    /// Execute CTE subquery and return rows
    pub(crate) fn execute_cte_subquery(
        &self,
        statement: &sqlrustgo_parser::Statement,
        ctx: &mut ProcedureContext,
    ) -> Result<Vec<Vec<Value>>, String> {
        match statement {
            sqlrustgo_parser::Statement::Select(select) => {
                let table_name = &select.first_table();

                if table_name.is_empty() {
                    let row: Vec<Value> = select
                        .columns
                        .iter()
                        .filter_map(|col| col.expression.as_ref())
                        .map(|expr| self.expression_to_value(expr, ctx))
                        .collect();
                    if row.is_empty() {
                        return Ok(vec![vec![]]);
                    }
                    if let Some(ref where_expr) = select.where_clause {
                        let where_val = self.expression_to_value(where_expr, ctx);
                        if let Value::Boolean(b) = where_val {
                            if b {
                                return Ok(vec![row]);
                            } else {
                                return Ok(vec![]);
                            }
                        }
                        if where_val != Value::Null {
                            return Ok(vec![row]);
                        }
                        return Ok(vec![]);
                    }
                    return Ok(vec![row]);
                }

                if let Some(cte_records) = ctx.cte_tables.get(table_name) {
                    let records = cte_records.clone();
                    if let Some(ref where_expr) = select.where_clause {
                        let filtered: Vec<Vec<Value>> = records
                            .into_iter()
                            .filter(|_row| {
                                let where_val = self.expression_to_value(where_expr, ctx);
                                if let Value::Boolean(b) = where_val {
                                    b
                                } else {
                                    where_val != Value::Null
                                }
                            })
                            .collect();
                        return Ok(filtered);
                    } else {
                        return Ok(records);
                    }
                }

                let storage = self.storage.read().unwrap();
                let records = storage
                    .scan(table_name)
                    .map_err(|e| format!("Failed to scan CTE table: {}", e))?;

                if let Some(ref where_expr) = select.where_clause {
                    let filtered: Vec<Vec<Value>> = records
                        .into_iter()
                        .filter(|_row| {
                            let where_val = self.expression_to_value(where_expr, ctx);
                            if let Value::Boolean(b) = where_val {
                                b
                            } else {
                                where_val != Value::Null
                            }
                        })
                        .collect();
                    Ok(filtered)
                } else {
                    Ok(records)
                }
            }
            sqlrustgo_parser::Statement::Union(union_stmt) => {
                let left_records = self.execute_cte_subquery(&union_stmt.left, ctx)?;
                let right_records = self.execute_cte_subquery(&union_stmt.right, ctx)?;
                if union_stmt.union_all {
                    Ok(left_records.into_iter().chain(right_records).collect())
                } else {
                    let mut combined = left_records;
                    combined.extend(right_records);
                    combined.sort();
                    combined.dedup();
                    Ok(combined)
                }
            }
            sqlrustgo_parser::Statement::Intersect(intersect_stmt) => {
                let left_records = self.execute_cte_subquery(&intersect_stmt.left, ctx)?;
                let right_records = self.execute_cte_subquery(&intersect_stmt.right, ctx)?;
                let right_set: std::collections::HashSet<_> = right_records.iter().collect();
                let result: Vec<_> = if intersect_stmt.intersect_all {
                    left_records
                        .into_iter()
                        .filter(|r| right_set.contains(r))
                        .collect()
                } else {
                    let mut left_unique: Vec<_> = left_records;
                    left_unique.sort();
                    left_unique.dedup();
                    left_unique
                        .into_iter()
                        .filter(|r| right_set.contains(r))
                        .collect()
                };
                Ok(result)
            }
            sqlrustgo_parser::Statement::Except(except_stmt) => {
                let left_records = self.execute_cte_subquery(&except_stmt.left, ctx)?;
                let right_records = self.execute_cte_subquery(&except_stmt.right, ctx)?;
                let right_set: std::collections::HashSet<_> = right_records.iter().collect();
                let result: Vec<_> = if except_stmt.except_all {
                    left_records
                        .into_iter()
                        .filter(|r| !right_set.contains(r))
                        .collect()
                } else {
                    let mut left_unique: Vec<_> = left_records;
                    left_unique.sort();
                    left_unique.dedup();
                    left_unique
                        .into_iter()
                        .filter(|r| !right_set.contains(r))
                        .collect()
                };
                Ok(result)
            }
            _ => Err(format!(
                "Unsupported statement type in CTE: {:?}",
                statement
            )),
        }
    }

    /// Execute a recursive CTE with iterative evaluation
    pub(crate) fn execute_recursive_cte(
        &self,
        anchor: &sqlrustgo_parser::Statement,
        recursive: &sqlrustgo_parser::Statement,
        cte_name: &str,
        ctx: &mut ProcedureContext,
    ) -> Result<Vec<Vec<Value>>, String> {
        let mut all_rows: Vec<Vec<Value>> = Vec::new();
        let mut cte_table: Vec<Vec<Value>> = Vec::new();
        let max_iterations = 1000;

        let anchor_rows = self.execute_cte_anchor(anchor, ctx)?;
        cte_table.extend(anchor_rows.clone());
        all_rows.extend(anchor_rows);

        for _iteration in 0..max_iterations {
            ctx.cte_tables
                .insert(cte_name.to_string(), cte_table.clone());

            let anchor_columns = self.extract_select_columns(recursive);
            let recursive_rows = self.execute_cte_recursive(recursive, ctx, &anchor_columns)?;

            if recursive_rows.is_empty() {
                break;
            }

            let new_rows: Vec<Vec<Value>> = recursive_rows
                .into_iter()
                .filter(|row| !cte_table.contains(row))
                .collect();

            if new_rows.is_empty() {
                break;
            }

            cte_table.extend(new_rows.clone());
            all_rows.extend(new_rows);
        }

        ctx.cte_tables.insert(cte_name.to_string(), cte_table);
        Ok(all_rows)
    }

    /// Extract column names from a SELECT statement
    pub(crate) fn extract_select_columns(
        &self,
        statement: &sqlrustgo_parser::Statement,
    ) -> Vec<String> {
        if let sqlrustgo_parser::Statement::Select(select) = statement {
            let cols: Vec<String> = select
                .columns
                .iter()
                .filter_map(|col| {
                    if col.expression.is_none() {
                        return Some(col.name.clone());
                    }
                    match col.expression.as_ref().unwrap() {
                        sqlrustgo_parser::Expression::Literal(_) => {
                            if col.alias.is_some() {
                                Some(col.alias.clone().unwrap())
                            } else {
                                Some(col.name.clone())
                            }
                        }
                        sqlrustgo_parser::Expression::Identifier(_) => {
                            if col.alias.is_some() {
                                Some(col.alias.clone().unwrap())
                            } else {
                                None
                            }
                        }
                        _ => Some(col.alias.clone().unwrap_or_else(|| col.name.clone())),
                    }
                })
                .collect();
            cols
        } else {
            vec![]
        }
    }

    /// Execute the anchor member of a recursive CTE (non-recursive SELECT)
    pub(crate) fn execute_cte_anchor(
        &self,
        statement: &sqlrustgo_parser::Statement,
        ctx: &mut ProcedureContext,
    ) -> Result<Vec<Vec<Value>>, String> {
        match statement {
            sqlrustgo_parser::Statement::Select(select) => {
                let table_name = select.first_table();

                if table_name.is_empty() || table_name == "empty" {
                    let row_ctx = ctx.clone();
                    let projected_row: Vec<Value> = select
                        .columns
                        .iter()
                        .filter_map(|col| {
                            col.expression
                                .as_ref()
                                .map(|expr| self.expression_to_value_with_context(expr, &row_ctx))
                        })
                        .collect();
                    return Ok(vec![projected_row]);
                }

                let raw_records: Vec<Vec<Value>> = if ctx.cte_tables.contains_key(&table_name) {
                    ctx.cte_tables.get(&table_name).cloned().unwrap_or_default()
                } else {
                    let storage = self.storage.read().unwrap();
                    storage
                        .scan(&table_name)
                        .map_err(|e| format!("Failed to scan anchor table: {}", e))?
                };

                let bound_ctx = ctx.clone();
                let columns: Vec<String> = select.columns.iter().map(|c| c.name.clone()).collect();
                let projected: Vec<Vec<Value>> = raw_records
                    .iter()
                    .filter_map(|row| {
                        let row_ctx = self.bind_row_to_context(bound_ctx.clone(), &columns, row);

                        if let Some(ref where_expr) = select.where_clause {
                            let where_val =
                                self.expression_to_value_with_context(where_expr, &row_ctx);
                            if !self.value_is_true(&where_val) {
                                return None;
                            }
                        }

                        let projected_row: Vec<Value> = select
                            .columns
                            .iter()
                            .filter_map(|col| {
                                col.expression.as_ref().map(|expr| {
                                    self.expression_to_value_with_context(expr, &row_ctx)
                                })
                            })
                            .collect();

                        if projected_row.is_empty() {
                            Some(row.clone())
                        } else {
                            Some(projected_row)
                        }
                    })
                    .collect();

                Ok(projected)
            }
            _ => Err(format!(
                "Unsupported anchor statement type: {:?}",
                statement
            )),
        }
    }

    /// Execute the recursive member of a recursive CTE
    pub(crate) fn execute_cte_recursive(
        &self,
        statement: &sqlrustgo_parser::Statement,
        ctx: &mut ProcedureContext,
        _cte_columns: &[String],
    ) -> Result<Vec<Vec<Value>>, String> {
        match statement {
            sqlrustgo_parser::Statement::Select(select) => {
                let table_name = &select.first_table();
                let records = if ctx.cte_tables.contains_key(table_name) {
                    ctx.cte_tables.get(table_name).cloned().unwrap_or_default()
                } else {
                    let storage = self.storage.read().unwrap();
                    storage
                        .scan(table_name)
                        .map_err(|e| format!("Failed to scan recursive table: {}", e))?
                };

                let column_bindings = self.extract_column_binding(select);

                let projected: Vec<Vec<Value>> = records
                    .iter()
                    .filter_map(|row| {
                        if let Some(ref where_expr) = select.where_clause {
                            let where_val = self.evaluate_expression_with_binding(
                                where_expr,
                                row,
                                &column_bindings,
                            );
                            if !self.value_is_true(&where_val) {
                                return None;
                            }
                        }

                        let projected_row: Vec<Value> = select
                            .columns
                            .iter()
                            .filter_map(|col| {
                                col.expression.as_ref().map(|expr| {
                                    self.evaluate_expression_with_binding(
                                        expr,
                                        row,
                                        &column_bindings,
                                    )
                                })
                            })
                            .collect();

                        if projected_row.is_empty() {
                            Some(row.clone())
                        } else {
                            Some(projected_row)
                        }
                    })
                    .collect();
                Ok(projected)
            }
            _ => Err(format!(
                "Unsupported recursive statement type: {:?}",
                statement
            )),
        }
    }
}
