use super::context::ProcedureContext;
use super::context::StoredProcError;
use super::StoredProcExecutor;
use sqlrustgo_catalog::StoredProcStatement;
use sqlrustgo_storage::ColumnDefinition;
use sqlrustgo_types::Value;

impl StoredProcExecutor {
    /// Execute a list of procedure statements
    pub(crate) fn execute_body(
        &self,
        body: &[StoredProcStatement],
        ctx: &mut ProcedureContext,
    ) -> Result<(), String> {
        for stmt in body {
            if ctx.should_leave() {
                ctx.reset_leave();
                break;
            }
            if ctx.should_iterate() {
                ctx.reset_iterate();
                break;
            }
            if ctx.get_return().is_some() {
                break;
            }

            let result = self.execute_statement(stmt, ctx);

            if let Err(ref e) = result {
                if let Some(err_str) = e.strip_prefix("SQLSTATE ") {
                    let sqlstate = err_str
                        .split(':')
                        .next()
                        .unwrap_or("45000")
                        .trim()
                        .to_string();
                    let message = e
                        .strip_prefix(&format!("SQLSTATE {}: ", sqlstate))
                        .unwrap_or(e)
                        .trim()
                        .to_string();
                    let exc = StoredProcError { sqlstate, message };

                    if let Some(handler) = ctx.find_matching_handler(&exc) {
                        let handler_body = handler.body.clone();
                        ctx.set_exception_handling(true);
                        ctx.set_exception(exc.sqlstate.clone(), exc.message.clone());
                        let handler_result = self.execute_body(&handler_body, ctx);
                        ctx.clear_exception();
                        ctx.set_exception_handling(false);

                        handler_result.as_ref()?;
                        continue;
                    }
                }
                return result;
            }
        }
        Ok(())
    }

    /// Execute a single procedure statement
    pub(crate) fn execute_statement(
        &self,
        stmt: &StoredProcStatement,
        ctx: &mut ProcedureContext,
    ) -> Result<(), String> {
        match stmt {
            StoredProcStatement::Declare {
                name,
                default_value,
                ..
            } => {
                let value = default_value
                    .as_ref()
                    .map(|v| self.evaluate_constant(v))
                    .unwrap_or_else(|| Value::Null);
                ctx.set_var(name, value);
                Ok(())
            }
            StoredProcStatement::Set { variable, value } => {
                let evaluated = self.evaluate_expression(value, ctx)?;
                ctx.set_var(variable, evaluated);
                Ok(())
            }
            StoredProcStatement::RawSql(sql) => {
                if !sql.is_empty() {
                    self.execute_sql(sql, ctx)?;
                }
                Ok(())
            }
            StoredProcStatement::SelectInto {
                columns,
                into_vars,
                table,
                where_clause,
            } => {
                let where_str = where_clause
                    .as_ref()
                    .map(|w| format!(" WHERE {}", self.expand_variables_in_sql(w, ctx)))
                    .unwrap_or_default();

                let _query = if columns.is_empty() {
                    format!("SELECT * FROM {}{}", table, where_str)
                } else {
                    let cols = columns.join(", ");
                    format!("SELECT {} FROM {}{}", cols, table, where_str)
                };

                for (i, var) in into_vars.iter().enumerate() {
                    if i < columns.len() {
                        let col_expr = &columns[i];
                        let value = self.evaluate_expression(col_expr, ctx)?;
                        ctx.set_var(var, value);
                    } else {
                        ctx.set_var(var, Value::Null);
                    }
                }

                Ok(())
            }
            StoredProcStatement::If {
                condition,
                then_body,
                elseif_body,
                else_body,
            } => {
                if self.evaluate_condition(condition, ctx)? {
                    self.execute_body(then_body, ctx)?;
                } else {
                    let mut matched = false;
                    for (elsif_cond, elsif_body) in elseif_body {
                        if self.evaluate_condition(elsif_cond, ctx)? {
                            self.execute_body(elsif_body, ctx)?;
                            matched = true;
                            break;
                        }
                    }
                    if !matched && !else_body.is_empty() {
                        self.execute_body(else_body, ctx)?;
                    }
                }
                Ok(())
            }
            StoredProcStatement::While { condition, body } => {
                while self.evaluate_condition(condition, ctx)?
                    && !ctx.should_leave()
                    && ctx.get_return().is_none()
                {
                    ctx.reset_iterate();
                    self.execute_body(body, ctx)?;
                    if ctx.should_iterate() {
                        ctx.reset_iterate();
                    }
                }
                ctx.reset_leave();
                Ok(())
            }
            StoredProcStatement::Loop { body } => {
                loop {
                    if ctx.should_leave() {
                        ctx.reset_leave();
                        break;
                    }
                    if ctx.get_return().is_some() {
                        break;
                    }
                    self.execute_body(body, ctx)?;
                }
                Ok(())
            }
            StoredProcStatement::Case {
                case_value,
                when_clauses,
                else_result,
            } => {
                let case_val = if let Some(ref cv) = case_value {
                    self.evaluate_expression(cv, ctx)?
                } else {
                    Value::Null
                };

                for (when_val, result) in when_clauses {
                    let when_expr_val = self.evaluate_expression(when_val, ctx)?;
                    if case_val == when_expr_val {
                        return self.evaluate_expression(result, ctx).map(|v| {
                            ctx.set_return(v);
                        });
                    }
                }

                if let Some(else_val) = else_result {
                    return self.evaluate_expression(else_val, ctx).map(|v| {
                        ctx.set_return(v);
                    });
                }

                Ok(())
            }
            StoredProcStatement::CaseWhen {
                when_clauses,
                else_result,
            } => {
                for (condition, result) in when_clauses {
                    if self.evaluate_condition(condition, ctx)? {
                        return self.evaluate_expression(result, ctx).map(|v| {
                            ctx.set_return(v);
                        });
                    }
                }

                if let Some(else_val) = else_result {
                    return self.evaluate_expression(else_val, ctx).map(|v| {
                        ctx.set_return(v);
                    });
                }

                Ok(())
            }
            StoredProcStatement::Repeat { body, condition } => {
                loop {
                    if ctx.should_leave() {
                        ctx.reset_leave();
                        break;
                    }
                    if ctx.get_return().is_some() {
                        break;
                    }

                    ctx.reset_iterate();
                    self.execute_body(body, ctx)?;

                    if ctx.should_iterate() {
                        ctx.reset_iterate();
                        continue;
                    }

                    if self.evaluate_condition(condition, ctx)? {
                        break;
                    }
                }
                Ok(())
            }
            StoredProcStatement::Leave { .. } => {
                ctx.set_leave();
                Ok(())
            }
            StoredProcStatement::Iterate { .. } => {
                ctx.set_iterate();
                Ok(())
            }
            StoredProcStatement::Return { value } => {
                let ret_val = self.evaluate_expression(value, ctx)?;
                ctx.set_return(ret_val);
                Ok(())
            }
            StoredProcStatement::Call {
                procedure_name,
                args,
                into_var,
            } => {
                let call_args: Vec<Value> = args
                    .iter()
                    .map(|a| self.evaluate_expression(a, ctx).unwrap_or(Value::Null))
                    .collect();

                let result = self.execute_call(procedure_name, call_args);

                match result {
                    Ok(exec_result) => {
                        if let Some(var_name) = into_var {
                            if let Some(row) = exec_result.rows.first() {
                                if let Some(val) = row.first() {
                                    ctx.set_var(var_name, val.clone());
                                }
                            }
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            StoredProcStatement::Signal { sqlstate, message } => {
                let sqlstate = sqlstate.as_deref().unwrap_or("45000");
                let message = message.as_deref().unwrap_or("Unhandled exception");
                Err(format!("SQLSTATE {}: {}", sqlstate, message))
            }
            StoredProcStatement::Resignal { sqlstate, message } => {
                let sqlstate = sqlstate.as_deref().unwrap_or("45000");
                let message = message.as_deref().unwrap_or("Unhandled exception");
                Err(format!("SQLSTATE {}: {}", sqlstate, message))
            }
            StoredProcStatement::Block { label, body } => {
                if let Some(ref lbl) = label {
                    ctx.enter_label(lbl.clone());
                }

                ctx.enter_scope();

                let result = self.execute_body(body, ctx);

                ctx.exit_scope();

                if label.is_some() {
                    ctx.exit_label();
                }

                result
            }
            StoredProcStatement::DeclareHandler {
                condition_type,
                body,
            } => {
                ctx.push_handler(condition_type.clone(), body.clone());
                Ok(())
            }
            StoredProcStatement::DeclareCursor { name, query } => {
                ctx.declare_cursor(name.clone(), query.clone());
                Ok(())
            }
            StoredProcStatement::OpenCursor { name } => {
                let query = if let Some(cursor) = ctx.cursors.get(&name.clone()) {
                    cursor.query.clone()
                } else {
                    return Err(format!("Cursor '{}' not found", name));
                };

                let expanded = self.expand_variables_in_sql(&query, ctx);
                let statement = sqlrustgo_parser::parse(&expanded)
                    .map_err(|e| format!("Failed to parse cursor query: {}", e))?;

                if let sqlrustgo_parser::Statement::Select(select) = statement {
                    let storage = self.storage.read().unwrap();
                    let records = storage
                        .scan(&select.first_table())
                        .map_err(|e| format!("Failed to scan table: {}", e))?;
                    ctx.set_cursor_records(name, records);
                    ctx.open_cursor(name)?;
                }
                Ok(())
            }
            StoredProcStatement::Fetch { name, into_vars } => {
                let has_rows = ctx.fetch_cursor(name, into_vars)?;
                ctx.set_session_var("__found", Value::Boolean(has_rows));
                if !has_rows {
                    ctx.set_session_var("__found_rows", Value::Integer(0));
                }
                Ok(())
            }
            StoredProcStatement::CloseCursor { name } => {
                ctx.close_cursor(name)?;
                Ok(())
            }
        }
    }

    /// Execute a SQL statement using the storage engine
    pub(crate) fn execute_sql(&self, sql: &str, ctx: &mut ProcedureContext) -> Result<(), String> {
        let expanded_sql = self.expand_variables_in_sql(sql, ctx);
        let sql_upper = expanded_sql.trim().to_uppercase();

        if sql_upper.starts_with("SELECT")
            || sql_upper.starts_with("INSERT")
            || sql_upper.starts_with("UPDATE")
            || sql_upper.starts_with("DELETE")
        {
            let statement = sqlrustgo_parser::parse(&expanded_sql)
                .map_err(|e| format!("Failed to parse SQL: {}", e))?;

            self.execute_statement_storage_impl(&statement, ctx)?;

            if let Some(found_rows) = ctx.get_session_var("__found_rows") {
                ctx.set_session_var("ROW_COUNT", found_rows.clone());
            } else if let Some(last_insert) = ctx.get_session_var("__last_insert_count") {
                ctx.set_session_var("ROW_COUNT", last_insert.clone());
            } else if let Some(last_update) = ctx.get_session_var("__last_update_count") {
                ctx.set_session_var("ROW_COUNT", last_update.clone());
            } else if let Some(last_delete) = ctx.get_session_var("__last_delete_count") {
                ctx.set_session_var("ROW_COUNT", last_delete.clone());
            }
        } else if sql_upper.starts_with("CREATE")
            || sql_upper.starts_with("DROP")
            || sql_upper.starts_with("ALTER")
            || sql_upper.starts_with("SHOW")
            || sql_upper.starts_with("DESCRIBE")
            || sql_upper.starts_with("SET")
        {
            ctx.set_session_var("ROW_COUNT", Value::Integer(0));
        }

        Ok(())
    }

    /// Execute a parsed statement using storage engine
    pub(crate) fn execute_statement_storage_impl(
        &self,
        statement: &sqlrustgo_parser::Statement,
        ctx: &mut ProcedureContext,
    ) -> Result<(), String> {
        match statement {
            sqlrustgo_parser::Statement::WithSelect(with_select) => {
                if let Some(ref with_clause) = with_select.with_clause {
                    for cte in &with_clause.ctes {
                        let cte_records = if let sqlrustgo_parser::Statement::Union(union_stmt) =
                            &*cte.subquery
                        {
                            if union_stmt.union_all {
                                self.execute_recursive_cte(
                                    &union_stmt.left,
                                    &union_stmt.right,
                                    &cte.name,
                                    ctx,
                                )?
                            } else {
                                self.execute_cte_subquery(&cte.subquery, ctx)?
                            }
                        } else {
                            self.execute_cte_subquery(&cte.subquery, ctx)?
                        };
                        ctx.cte_tables.insert(cte.name.clone(), cte_records);
                    }
                }
                match with_select.select.as_ref() {
                    sqlrustgo_parser::Statement::Select(select) => {
                        let table_name = select.first_table();

                        let records = if ctx.cte_tables.contains_key(&table_name) {
                            ctx.cte_tables.get(&table_name).cloned().unwrap_or_default()
                        } else {
                            let storage = self.storage.read().unwrap();
                            storage
                                .scan(&table_name)
                                .map_err(|e| format!("Failed to scan table: {}", e))?
                        };

                        let filtered: Vec<Vec<Value>> =
                            if let Some(ref where_expr) = select.where_clause {
                                records
                                    .into_iter()
                                    .filter(|_row| {
                                        let where_val = self.expression_to_value(where_expr, ctx);
                                        if let Value::Boolean(b) = where_val {
                                            b
                                        } else {
                                            where_val != Value::Null
                                        }
                                    })
                                    .collect()
                            } else {
                                records
                            };

                        ctx.set_session_var(
                            "__last_select_result",
                            Value::Text(serde_json::to_string(&filtered).unwrap_or_default()),
                        );
                        ctx.set_session_var("__found_rows", Value::Integer(filtered.len() as i64));
                        Ok(())
                    }
                    sqlrustgo_parser::Statement::Union(union_stmt) => {
                        let left_table = if let sqlrustgo_parser::Statement::Select(left_select) =
                            union_stmt.left.as_ref()
                        {
                            left_select.first_table()
                        } else {
                            return Err("Union left side must be SELECT".to_string());
                        };
                        let right_table = if let sqlrustgo_parser::Statement::Select(right_select) =
                            union_stmt.right.as_ref()
                        {
                            right_select.first_table()
                        } else {
                            return Err("Union right side must be SELECT".to_string());
                        };

                        let left_records = if ctx.cte_tables.contains_key(&left_table) {
                            ctx.cte_tables.get(&left_table).cloned().unwrap_or_default()
                        } else {
                            let storage = self.storage.read().unwrap();
                            storage
                                .scan(&left_table)
                                .map_err(|e| format!("Failed to scan table: {}", e))?
                        };
                        let right_records = if ctx.cte_tables.contains_key(&right_table) {
                            ctx.cte_tables
                                .get(&right_table)
                                .cloned()
                                .unwrap_or_default()
                        } else {
                            let storage = self.storage.read().unwrap();
                            storage
                                .scan(&right_table)
                                .map_err(|e| format!("Failed to scan table: {}", e))?
                        };

                        let mut combined = left_records;
                        combined.extend(right_records);
                        if !union_stmt.union_all {
                            combined.sort();
                            combined.dedup();
                        }

                        ctx.set_session_var(
                            "__last_select_result",
                            Value::Text(serde_json::to_string(&combined).unwrap_or_default()),
                        );
                        ctx.set_session_var("__found_rows", Value::Integer(combined.len() as i64));
                        Ok(())
                    }
                    _ => Err("CTE main select must be SELECT or UNION".to_string()),
                }
            }
            sqlrustgo_parser::Statement::Select(select) => {
                let table_name = &select.first_table();

                let records = if table_name.is_empty() {
                    vec![]
                } else if ctx.cte_tables.contains_key(table_name) {
                    ctx.cte_tables.get(table_name).cloned().unwrap_or_default()
                } else {
                    let storage = self.storage.read().unwrap();
                    storage
                        .scan(table_name)
                        .map_err(|e| format!("Failed to scan table: {}", e))?
                };

                let filtered: Vec<Vec<Value>> = if let Some(ref where_expr) = select.where_clause {
                    records
                        .into_iter()
                        .filter(|_row| {
                            let where_val = self.expression_to_value(where_expr, ctx);
                            if let Value::Boolean(b) = where_val {
                                b
                            } else {
                                where_val != Value::Null
                            }
                        })
                        .collect()
                } else {
                    records
                };

                ctx.set_session_var(
                    "__last_select_result",
                    Value::Text(serde_json::to_string(&filtered).unwrap_or_default()),
                );
                ctx.set_session_var("__found_rows", Value::Integer(filtered.len() as i64));
                Ok(())
            }
            sqlrustgo_parser::Statement::Insert(insert) => {
                let table_name = &insert.table.clone();
                let insert_columns = insert.columns.clone();

                let table_info = {
                    let storage = self.storage.read().unwrap();
                    if !storage.has_table(table_name) {
                        return Err(format!("Table '{}' not found", table_name));
                    }
                    storage.get_table_info(table_name).ok()
                };

                let num_columns = table_info.as_ref().map(|i| i.columns.len()).unwrap_or(0);
                let has_hidden_rowid = table_info
                    .as_ref()
                    .map(|i| i.has_hidden_rowid)
                    .unwrap_or(false);
                let mut new_rows: Vec<Vec<Value>> = Vec::new();
                let mut insert_count = 0;

                if let Some(ref select_box) = insert.select {
                    if let sqlrustgo_parser::Statement::Select(ref select) = select_box.as_ref() {
                        let storage = self.storage.read().unwrap();
                        let records = storage
                            .scan(&select.first_table())
                            .map_err(|e| format!("Failed to scan table: {}", e))?;

                        let selected_rows: Vec<Vec<Value>> =
                            if let Some(ref where_expr) = select.where_clause {
                                records
                                    .into_iter()
                                    .filter(|_row| {
                                        let where_val = self.expression_to_value(where_expr, ctx);
                                        if let Value::Boolean(b) = where_val {
                                            b
                                        } else {
                                            where_val != Value::Null
                                        }
                                    })
                                    .collect()
                            } else {
                                records
                            };

                        for row in selected_rows {
                            let mut new_row: Vec<Value> = vec![Value::Null; num_columns];
                            if insert_columns.is_empty() {
                                for (col_idx, val) in row.iter().enumerate() {
                                    if col_idx < num_columns {
                                        new_row[col_idx] = val.clone();
                                    }
                                }
                            } else {
                                for (col_idx, col_name) in insert_columns.iter().enumerate() {
                                    if col_idx < row.len() {
                                        if let Some(ref info) = table_info {
                                            if let Some(target_idx) = info
                                                .columns
                                                .iter()
                                                .position(|c| c.name.eq_ignore_ascii_case(col_name))
                                            {
                                                new_row[target_idx] = row[col_idx].clone();
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(ref info) = table_info {
                                if !info.foreign_keys.is_empty() {
                                    self.validate_foreign_keys(
                                        table_name,
                                        &new_row,
                                        &insert_columns,
                                    )?;
                                }
                                if info.columns.iter().any(|c| c.primary_key) {
                                    self.validate_primary_key(
                                        table_name,
                                        &new_row,
                                        &insert_columns,
                                    )?;
                                }
                                if !info.unique_constraints.is_empty() {
                                    self.validate_unique_constraints(
                                        table_name,
                                        &new_row,
                                        &insert_columns,
                                    )?;
                                }
                            }
                            new_rows.push(new_row);
                        }
                        insert_count = new_rows.len();
                    }
                } else {
                    for row in &insert.values {
                        let mut new_row: Vec<Value> = vec![Value::Null; num_columns];

                        if insert_columns.is_empty() {
                            for (col_idx, expr) in row.iter().enumerate() {
                                if col_idx < num_columns {
                                    new_row[col_idx] = self.expression_to_value(expr, ctx);
                                }
                            }
                        } else {
                            for (value_idx, col_name) in insert_columns.iter().enumerate() {
                                if value_idx < row.len() {
                                    if let Some(ref info) = table_info {
                                        if let Some(target_idx) = info
                                            .columns
                                            .iter()
                                            .position(|c| c.name.eq_ignore_ascii_case(col_name))
                                        {
                                            new_row[target_idx] =
                                                self.expression_to_value(&row[value_idx], ctx);
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(ref info) = table_info {
                            if !info.foreign_keys.is_empty() {
                                let cols = insert_columns.clone();
                                self.validate_foreign_keys(table_name, &new_row, &cols)?;
                            }
                            if info.columns.iter().any(|c| c.primary_key) {
                                self.validate_primary_key(table_name, &new_row, &insert_columns)?;
                            }
                            if !info.unique_constraints.is_empty() {
                                self.validate_unique_constraints(
                                    table_name,
                                    &new_row,
                                    &insert_columns,
                                )?;
                            }
                        }

                        new_rows.push(new_row);
                    }
                    insert_count = insert.values.len();
                }

                {
                    let mut storage = self.storage.write().unwrap();
                    for mut new_row in new_rows {
                        if has_hidden_rowid {
                            if let Ok(Some(rowid)) =
                                storage.get_and_increment_next_rowid(table_name)
                            {
                                new_row.insert(0, Value::Integer(rowid as i64));
                            }
                        }
                        storage
                            .insert(table_name, vec![new_row])
                            .map_err(|e| format!("Failed to insert: {}", e))?;
                    }
                }

                ctx.set_session_var("__last_insert_count", Value::Integer(insert_count as i64));
                Ok(())
            }
            sqlrustgo_parser::Statement::Update(update) => {
                let table_name = &update.table;
                let mut storage = self.storage.write().unwrap();

                if !storage.has_table(table_name) {
                    return Err(format!("Table '{}' not found", table_name));
                }

                let table_info = storage.get_table_info(table_name).ok();

                let col_index_map: std::collections::HashMap<String, usize> = table_info
                    .as_ref()
                    .map(|info| {
                        info.columns
                            .iter()
                            .enumerate()
                            .map(|(i, c)| (c.name.to_lowercase(), i))
                            .collect()
                    })
                    .unwrap_or_default();

                let records = match storage.get_table_records_mut(table_name) {
                    Ok(r) => r,
                    Err(e) => return Err(format!("Failed to get table records: {}", e)),
                };

                let mut count = 0;
                let where_expr = update.where_clause.clone();

                for row in records.iter_mut() {
                    let matches = if let Some(ref where_clz) = where_expr {
                        let result =
                            self.evaluate_row_expression(where_clz, row, &col_index_map, ctx);
                        if let Value::Boolean(b) = result {
                            b
                        } else {
                            result != Value::Null
                        }
                    } else {
                        true
                    };

                    if matches {
                        for (col_name, expr) in &update.set_clauses {
                            if let Some(col_idx) = col_index_map.get(&col_name.to_lowercase()) {
                                if *col_idx < row.len() {
                                    let new_val = self.evaluate_row_expression(
                                        expr,
                                        row,
                                        &col_index_map,
                                        ctx,
                                    );
                                    row[*col_idx] = new_val;
                                }
                            }
                        }
                        count += 1;
                    }
                }

                ctx.set_session_var("__last_update_count", Value::Integer(count as i64));
                Ok(())
            }
            sqlrustgo_parser::Statement::Delete(delete) => {
                let table_name = &delete.table;
                let mut storage = self.storage.write().unwrap();

                if !storage.has_table(table_name) {
                    return Err(format!("Table '{}' not found", table_name));
                }

                let table_info = storage.get_table_info(table_name).ok();

                let col_index_map: std::collections::HashMap<String, usize> = table_info
                    .as_ref()
                    .map(|info| {
                        info.columns
                            .iter()
                            .enumerate()
                            .map(|(i, c)| (c.name.to_lowercase(), i))
                            .collect()
                    })
                    .unwrap_or_default();

                let records = match storage.get_table_records_mut(table_name) {
                    Ok(r) => r,
                    Err(e) => return Err(format!("Failed to get table records: {}", e)),
                };

                let where_expr = delete.where_clause.clone();

                let mut indices_to_delete: Vec<usize> = Vec::new();
                for (idx, row) in records.iter().enumerate() {
                    let matches = if let Some(ref where_clz) = where_expr {
                        let result =
                            self.evaluate_row_expression(where_clz, row, &col_index_map, ctx);
                        if let Value::Boolean(b) = result {
                            b
                        } else {
                            result != Value::Null
                        }
                    } else {
                        true
                    };
                    if matches {
                        indices_to_delete.push(idx);
                    }
                }

                let deleted_count = indices_to_delete.len();
                for idx in indices_to_delete.into_iter().rev() {
                    records.remove(idx);
                }

                ctx.set_session_var("__last_delete_count", Value::Integer(deleted_count as i64));
                Ok(())
            }
            sqlrustgo_parser::Statement::AlterTable(alter_table) => {
                let table_name = &alter_table.table_name;
                let mut storage = self.storage.write().unwrap();

                match &alter_table.operation {
                    sqlrustgo_parser::AlterTableOperation::AddColumn {
                        name,
                        data_type,
                        nullable,
                        default_value: _,
                    } => {
                        let column = ColumnDefinition {
                            name: name.clone(),
                            data_type: data_type.clone(),
                            nullable: *nullable,
                            primary_key: false,
                            auto_increment: false,
                        };
                        storage
                            .add_column(table_name, column)
                            .map_err(|e| format!("Failed to add column: {}", e))?;
                    }
                    sqlrustgo_parser::AlterTableOperation::RenameTo { new_name } => {
                        storage
                            .rename_table(table_name, new_name)
                            .map_err(|e| format!("Failed to rename table: {}", e))?;
                    }
                    sqlrustgo_parser::AlterTableOperation::DropColumn { name } => {
                        return Err(format!(
                            "DROP COLUMN '{}' not yet implemented in storage layer",
                            name
                        ));
                    }
                    sqlrustgo_parser::AlterTableOperation::ModifyColumn {
                        name,
                        data_type,
                        nullable: _,
                    } => {
                        return Err(format!(
                            "MODIFY COLUMN '{} {}' not yet implemented in storage layer",
                            name, data_type
                        ));
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Execute a SELECT statement and return rows
    pub(crate) fn execute_subquery(
        &self,
        select: &sqlrustgo_parser::SelectStatement,
    ) -> Vec<Vec<Value>> {
        let storage = match self.storage.read() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let records = match storage.scan(&select.first_table()) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        if let Some(ref where_expr) = select.where_clause {
            records
                .into_iter()
                .filter(|_row| {
                    let where_val = self.expression_to_value(where_expr, &ProcedureContext::new());
                    if let Value::Boolean(b) = where_val {
                        b
                    } else {
                        where_val != Value::Null
                    }
                })
                .collect()
        } else {
            records
        }
    }

    /// Extract column names and their positions from SELECT columns for binding
    pub(crate) fn extract_column_binding(
        &self,
        select: &sqlrustgo_parser::SelectStatement,
    ) -> Vec<(String, usize)> {
        let mut bindings = Vec::new();
        for (idx, col) in select.columns.iter().enumerate() {
            if let Some(ref expr) = col.expression {
                let identifiers = self.extract_identifiers_from_expr(expr);
                for name in identifiers {
                    if !name.starts_with('@') && name.parse::<i64>().is_err() {
                        bindings.push((name, idx));
                        break;
                    }
                }
            }
        }
        bindings
    }

    /// Bind row values to column names in a cloned context
    pub(crate) fn bind_row_to_context(
        &self,
        mut ctx: ProcedureContext,
        columns: &[String],
        row: &[Value],
    ) -> ProcedureContext {
        for (i, col_name) in columns.iter().enumerate() {
            if i < row.len() {
                ctx.set_var(col_name, row[i].clone());
            }
        }
        ctx
    }

    /// Validate foreign key constraints for a row being inserted
    pub(crate) fn validate_foreign_keys(
        &self,
        table_name: &str,
        row: &[Value],
        columns: &[String],
    ) -> Result<(), String> {
        let storage = self.storage.read().unwrap();
        let table_info = storage
            .get_table_info(table_name)
            .map_err(|e| format!("Failed to get table info: {}", e))?;

        for fk in &table_info.foreign_keys {
            let parent_values: Vec<Value> = fk
                .columns
                .iter()
                .filter_map(|col_name| {
                    columns
                        .iter()
                        .position(|c| c.eq_ignore_ascii_case(col_name))
                        .and_then(|idx| row.get(idx).cloned())
                })
                .collect();

            if parent_values.iter().any(|v| matches!(v, Value::Null)) {
                continue;
            }

            let parent_rows = storage
                .scan(&fk.referenced_table)
                .map_err(|e| format!("Failed to scan parent table: {}", e))?;

            let ref_col_indices: Vec<usize> = fk
                .referenced_columns
                .iter()
                .filter_map(|col_name| {
                    storage
                        .get_table_info(&fk.referenced_table)
                        .ok()?
                        .columns
                        .iter()
                        .position(|c| c.name.eq_ignore_ascii_case(col_name))
                })
                .collect();

            let parent_has_match = parent_rows.iter().any(|parent_row| {
                ref_col_indices
                    .iter()
                    .enumerate()
                    .all(|(i, &col_idx)| parent_row.get(col_idx) == parent_values.get(i))
            });

            if !parent_has_match {
                return Err(format!(
                    "Foreign key constraint failed: {} ({}) references {} ({}) which does not exist",
                    table_name,
                    fk.columns.join(", "),
                    fk.referenced_table,
                    fk.referenced_columns.join(", ")
                ));
            }
        }

        Ok(())
    }

    /// Validate unique constraints for a row being inserted
    pub(crate) fn validate_unique_constraints(
        &self,
        table_name: &str,
        row: &[Value],
        columns: &[String],
    ) -> Result<(), String> {
        let storage = self.storage.read().unwrap();
        let table_info = storage
            .get_table_info(table_name)
            .map_err(|e| format!("Failed to get table info: {}", e))?;

        for unique_constraint in &table_info.unique_constraints {
            let col_indices: Vec<usize> = unique_constraint
                .columns
                .iter()
                .filter_map(|col_name| {
                    columns
                        .iter()
                        .position(|c| c.eq_ignore_ascii_case(col_name))
                })
                .collect();

            if col_indices.is_empty() {
                continue;
            }

            let values: Vec<Value> = col_indices
                .iter()
                .filter_map(|&i| row.get(i).cloned())
                .collect();

            if values.iter().any(|v| matches!(v, Value::Null)) {
                continue;
            }

            let existing_rows = storage
                .scan(table_name)
                .map_err(|e| format!("Failed to scan table: {}", e))?;

            for existing_row in existing_rows {
                let existing_values: Vec<Value> = col_indices
                    .iter()
                    .filter_map(|&i| existing_row.get(i).cloned())
                    .collect();
                if existing_values == values {
                    return Err(format!(
                        "Duplicate unique key '{}': ({}) values ({}) already exist",
                        unique_constraint.name.as_deref().unwrap_or("unnamed"),
                        unique_constraint.columns.join(", "),
                        values
                            .iter()
                            .map(|v| format!("{:?}", v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate PRIMARY KEY uniqueness for a row being inserted
    pub(crate) fn validate_primary_key(
        &self,
        table_name: &str,
        row: &[Value],
        columns: &[String],
    ) -> Result<(), String> {
        let storage = self.storage.read().unwrap();
        let table_info = storage
            .get_table_info(table_name)
            .map_err(|e| format!("Failed to get table info: {}", e))?;

        let col_to_row_idx: std::collections::HashMap<String, usize> = if columns.is_empty() {
            std::collections::HashMap::new()
        } else {
            columns
                .iter()
                .enumerate()
                .map(|(i, name)| (name.to_uppercase(), i))
                .collect()
        };

        let pk_col_names: Vec<String> = table_info
            .columns
            .iter()
            .filter(|c| c.primary_key)
            .map(|c| c.name.to_uppercase())
            .collect();

        if pk_col_names.is_empty() {
            return Ok(());
        }

        let pk_values: Vec<Value> = pk_col_names
            .iter()
            .filter_map(|pk_name| {
                if let Some(&row_idx) = col_to_row_idx.get(pk_name) {
                    row.get(row_idx).cloned()
                } else if columns.is_empty() {
                    table_info
                        .columns
                        .iter()
                        .position(|c| c.name.to_uppercase() == *pk_name)
                        .and_then(|col_idx| row.get(col_idx).cloned())
                } else {
                    None
                }
            })
            .collect();

        if pk_values.iter().any(|v| matches!(v, Value::Null)) {
            return Ok(());
        }

        let existing_rows = storage
            .scan(table_name)
            .map_err(|e| format!("Failed to scan table: {}", e))?;

        let all_col_names: Vec<String> = table_info
            .columns
            .iter()
            .map(|c| c.name.to_uppercase())
            .collect();

        for existing_row in existing_rows {
            let existing_pk_values: Vec<Value> = pk_col_names
                .iter()
                .filter_map(|pk_name| {
                    all_col_names
                        .iter()
                        .position(|c| c == pk_name)
                        .and_then(|col_idx| existing_row.get(col_idx).cloned())
                })
                .collect();
            if existing_pk_values == pk_values {
                return Err(format!(
                    "Duplicate primary key: ({}) values ({}) already exist",
                    pk_col_names.join(", "),
                    pk_values
                        .iter()
                        .map(|v| format!("{:?}", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn executor() -> StoredProcExecutor {
        let catalog = Arc::new(sqlrustgo_catalog::Catalog::new("test"));
        StoredProcExecutor::new_for_test(catalog)
    }

    #[test]
    fn test_execute_body_leave_early() {
        let ex = executor();
        let mut ctx = ProcedureContext::new();
        ctx.set_var("x", Value::Integer(1));
        let body = vec![
            StoredProcStatement::Set {
                variable: "x".into(),
                value: "2".into(),
            },
            StoredProcStatement::Leave {
                label: "block".into(),
            },
            StoredProcStatement::Set {
                variable: "x".into(),
                value: "3".into(),
            },
        ];
        let result = ex.execute_body(&body, &mut ctx);
        assert!(result.is_ok());
        assert_eq!(ctx.get_var("x"), Some(&Value::Integer(2)));
    }

    #[test]
    fn test_execute_body_return_early() {
        let ex = executor();
        let mut ctx = ProcedureContext::new();
        let body = vec![
            StoredProcStatement::Return { value: "42".into() },
            StoredProcStatement::Set {
                variable: "x".into(),
                value: "99".into(),
            },
        ];
        let result = ex.execute_body(&body, &mut ctx);
        assert!(result.is_ok());
        assert_eq!(ctx.get_return(), Some(Value::Integer(42)));
    }

    #[test]
    fn test_execute_body_iterate_resets() {
        let ex = executor();
        let mut ctx = ProcedureContext::new();
        ctx.set_var("i", Value::Integer(0));
        let body = vec![
            StoredProcStatement::Iterate {
                label: "loop".into(),
            },
            StoredProcStatement::Set {
                variable: "i".into(),
                value: "99".into(),
            },
        ];
        let result = ex.execute_body(&body, &mut ctx);
        assert!(result.is_ok());
        assert!(!ctx.should_iterate());
    }

    #[test]
    fn test_extract_column_binding() {
        let ex = executor();
        use sqlrustgo_parser::{Expression, SelectColumn, SelectStatement};
        let select = SelectStatement {
            columns: vec![
                SelectColumn {
                    name: "a".into(),
                    expression: Some(Expression::Identifier("col1".into())),
                    alias: None,
                },
                SelectColumn {
                    name: "b".into(),
                    expression: Some(Expression::Identifier("col2".into())),
                    alias: None,
                },
            ],
            table: "t".into(),
            from: None,
            where_clause: None,
            join_clause: None,
            aggregates: vec![],
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
            offset: None,
            distinct: false,
            for_update: false,
        };
        let binding = ex.extract_column_binding(&select);
        assert_eq!(binding.len(), 2);
        assert_eq!(binding[0], ("col1".into(), 0));
        assert_eq!(binding[1], ("col2".into(), 1));
    }

    #[test]
    fn test_bind_row_to_context() {
        let ex = executor();
        let mut ctx = ProcedureContext::new();
        let columns = vec!["a".into(), "b".into()];
        let row = vec![Value::Integer(10), Value::Text("hello".into())];
        ctx = ex.bind_row_to_context(ctx, &columns, &row);
        assert_eq!(ctx.get_var("a"), Some(&Value::Integer(10)));
        assert_eq!(ctx.get_var("b"), Some(&Value::Text("hello".into())));
    }
}
