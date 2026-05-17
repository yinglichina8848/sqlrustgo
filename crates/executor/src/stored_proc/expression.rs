use super::ProcedureContext;
use super::StoredProcExecutor;
use sqlrustgo_types::Value;

impl StoredProcExecutor {
    /// Convert parser Expression to runtime Value
    pub(crate) fn expression_to_value(
        &self,
        expr: &sqlrustgo_parser::Expression,
        ctx: &ProcedureContext,
    ) -> Value {
        match expr {
            sqlrustgo_parser::Expression::Literal(s) => {
                let s = s.trim();
                if s.eq_ignore_ascii_case("NULL") {
                    Value::Null
                } else if let Ok(n) = s.parse::<i64>() {
                    Value::Integer(n)
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Float(f)
                } else if s.starts_with('\'') && s.ends_with('\'') {
                    Value::Text(s[1..s.len() - 1].to_string())
                } else {
                    Value::Text(s.to_string())
                }
            }
            sqlrustgo_parser::Expression::Identifier(name) => {
                if let Some(stripped) = name.strip_prefix('@') {
                    ctx.get_var(stripped).cloned().unwrap_or(Value::Null)
                } else {
                    Value::Text(name.to_string())
                }
            }
            sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
                let left_val = self.expression_to_value(left, ctx);
                let right_val = self.expression_to_value(right, ctx);
                self.evaluate_binary_op(&left_val, &right_val, op)
            }
            sqlrustgo_parser::Expression::Subquery(select) => {
                let rows = self.execute_subquery(select);
                if let Some(first_row) = rows.first() {
                    first_row.first().cloned().unwrap_or(Value::Null)
                } else {
                    Value::Null
                }
            }
            sqlrustgo_parser::Expression::In(left, select) => {
                let left_val = self.expression_to_value(left, ctx);
                let rows = self.execute_subquery(select);
                let in_result = rows
                    .iter()
                    .any(|row| row.first().map(|v| v == &left_val).unwrap_or(false));
                Value::Boolean(in_result)
            }
            sqlrustgo_parser::Expression::NotIn(left, select) => {
                let left_val = self.expression_to_value(left, ctx);
                let rows = self.execute_subquery(select);
                let not_in_result = rows
                    .iter()
                    .all(|row| row.first().map(|v| v != &left_val).unwrap_or(true));
                Value::Boolean(not_in_result)
            }
            sqlrustgo_parser::Expression::Exists(select) => {
                let rows = self.execute_subquery(select);
                Value::Boolean(!rows.is_empty())
            }
            sqlrustgo_parser::Expression::NotExists(select) => {
                let rows = self.execute_subquery(select);
                Value::Boolean(rows.is_empty())
            }
            sqlrustgo_parser::Expression::QuantifiedOp(expr, quantifier, select) => {
                let rows = self.execute_subquery(select);
                let expr_val = self.expression_to_value(expr, ctx);
                match quantifier.as_str() {
                    "ALL" => {
                        let all_match = rows
                            .iter()
                            .all(|row| row.first().map(|v| v == &expr_val).unwrap_or(false));
                        Value::Boolean(all_match)
                    }
                    "ANY" | "SOME" => {
                        let any_match = rows
                            .iter()
                            .any(|row| row.first().map(|v| v == &expr_val).unwrap_or(false));
                        Value::Boolean(any_match)
                    }
                    _ => Value::Null,
                }
            }
            sqlrustgo_parser::Expression::IsNull(inner) => {
                let val = self.expression_to_value(inner, ctx);
                Value::Boolean(matches!(val, Value::Null))
            }
            sqlrustgo_parser::Expression::IsNotNull(inner) => {
                let val = self.expression_to_value(inner, ctx);
                Value::Boolean(!matches!(val, Value::Null))
            }
            sqlrustgo_parser::Expression::InList(left, values) => {
                let left_val = self.expression_to_value(left, ctx);
                let value_list: Vec<Value> = values
                    .iter()
                    .map(|v| self.expression_to_value(v, ctx))
                    .collect();
                Value::Boolean(value_list.contains(&left_val))
            }
            sqlrustgo_parser::Expression::NotInList(left, values) => {
                let left_val = self.expression_to_value(left, ctx);
                let value_list: Vec<Value> = values
                    .iter()
                    .map(|v| self.expression_to_value(v, ctx))
                    .collect();
                Value::Boolean(!value_list.contains(&left_val))
            }
            sqlrustgo_parser::Expression::NotLike(left, pattern, _) => {
                let left_val = self.expression_to_value(left, ctx);
                let pattern_val = self.expression_to_value(pattern, ctx);
                let like_result = self.like_match(&left_val, &pattern_val);
                Value::Boolean(!like_result)
            }
            sqlrustgo_parser::Expression::NotBetween(left, low, high) => {
                let left_val = self.expression_to_value(left, ctx);
                let low_val = self.expression_to_value(low, ctx);
                let high_val = self.expression_to_value(high, ctx);
                let between_result = self.between_match(&left_val, &low_val, &high_val);
                Value::Boolean(!between_result)
            }
            sqlrustgo_parser::Expression::NotRegexp(left, pattern) => {
                let left_val = self.expression_to_value(left, ctx);
                let pattern_val = self.expression_to_value(pattern, ctx);
                let regexp_result = self.regexp_match(&left_val, &pattern_val);
                Value::Boolean(!regexp_result)
            }
            sqlrustgo_parser::Expression::UnaryOp(op, expr) => {
                let val = self.expression_to_value(expr, ctx);
                match op.as_str() {
                    "NOT" => {
                        if let Value::Boolean(b) = val {
                            Value::Boolean(!b)
                        } else {
                            Value::Boolean(false)
                        }
                    }
                    _ => Value::Null,
                }
            }
            sqlrustgo_parser::Expression::Like(left, pattern, _) => {
                let left_val = self.expression_to_value(left, ctx);
                let pattern_val = self.expression_to_value(pattern, ctx);
                Value::Boolean(self.like_match(&left_val, &pattern_val))
            }
            sqlrustgo_parser::Expression::Between(left, low, high) => {
                let left_val = self.expression_to_value(left, ctx);
                let low_val = self.expression_to_value(low, ctx);
                let high_val = self.expression_to_value(high, ctx);
                Value::Boolean(self.between_match(&left_val, &low_val, &high_val))
            }
            sqlrustgo_parser::Expression::CaseWhen(when_clauses, else_expr) => {
                for clause in when_clauses {
                    let cond_val = self.expression_to_value(&clause.condition, ctx);
                    if let Value::Boolean(true) = cond_val {
                        return self.expression_to_value(&clause.result, ctx);
                    }
                }
                if let Some(else_box) = else_expr {
                    self.expression_to_value(else_box, ctx)
                } else {
                    Value::Null
                }
            }
            sqlrustgo_parser::Expression::Aggregate(_) => Value::Null,
            sqlrustgo_parser::Expression::FunctionCall(name, args) => {
                let name_upper = name.to_uppercase();
                if name_upper == "SUBSTRING" {
                    if args.is_empty() {
                        Value::Null
                    } else {
                        let str_val = self.expression_to_value(&args[0], ctx);
                        let start_val = if args.len() > 1 {
                            self.expression_to_value(&args[1], ctx)
                        } else {
                            Value::Null
                        };
                        let len_val = if args.len() > 2 {
                            Some(self.expression_to_value(&args[2], ctx))
                        } else {
                            None
                        };

                        match (str_val, start_val) {
                            (Value::Text(s), Value::Integer(start)) => {
                                let start_idx = ((start - 1).max(0)) as usize;
                                let result: String = if let Some(Value::Integer(len)) = len_val {
                                    s.chars().skip(start_idx).take(len as usize).collect()
                                } else {
                                    s.chars().skip(start_idx).collect()
                                };
                                Value::Text(result)
                            }
                            _ => Value::Null,
                        }
                    }
                } else {
                    Value::Null
                }
            }
            sqlrustgo_parser::Expression::WindowCall(_) => Value::Null,
            sqlrustgo_parser::Expression::Position(_, _) => Value::Null,
            sqlrustgo_parser::Expression::Insert(_, _, _, _) => Value::Null,
            sqlrustgo_parser::Expression::Extract(field, inner) => {
                let val = self.expression_to_value(inner, ctx);
                match (field.as_str(), &val) {
                    ("YEAR", Value::Integer(n)) => Value::Integer(*n / 10000),
                    ("MONTH", Value::Integer(n)) => Value::Integer((*n / 100) % 100),
                    ("DAY", Value::Integer(n)) => Value::Integer(*n % 100),
                    ("YEAR", Value::Text(s)) => {
                        if s.len() >= 4 {
                            s[..4].parse().map(Value::Integer).unwrap_or(Value::Null)
                        } else {
                            Value::Null
                        }
                    }
                    ("MONTH", Value::Text(s)) => {
                        if s.len() >= 7 {
                            s[5..7].parse().map(Value::Integer).unwrap_or(Value::Null)
                        } else {
                            Value::Null
                        }
                    }
                    ("DAY", Value::Text(s)) => {
                        if s.len() >= 10 {
                            s[8..10].parse().map(Value::Integer).unwrap_or(Value::Null)
                        } else {
                            Value::Null
                        }
                    }
                    _ => Value::Null,
                }
            }
            sqlrustgo_parser::Expression::MatchAgainst(_, _, _) => Value::Null,
        }
    }

    /// Evaluate an expression with row context (for UPDATE/DELETE WHERE clauses and SET expressions)
    pub(crate) fn evaluate_row_expression(
        &self,
        expr: &sqlrustgo_parser::Expression,
        row: &[Value],
        col_index_map: &std::collections::HashMap<String, usize>,
        ctx: &ProcedureContext,
    ) -> Value {
        match expr {
            sqlrustgo_parser::Expression::Literal(s) => {
                let s = s.trim();
                if s.eq_ignore_ascii_case("NULL") {
                    Value::Null
                } else if let Ok(n) = s.parse::<i64>() {
                    Value::Integer(n)
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Float(f)
                } else if s.starts_with('\'') && s.ends_with('\'') {
                    Value::Text(s[1..s.len() - 1].to_string())
                } else {
                    Value::Text(s.to_string())
                }
            }
            sqlrustgo_parser::Expression::Identifier(name) => {
                if let Some(stripped) = name.strip_prefix('@') {
                    ctx.get_var(stripped).cloned().unwrap_or(Value::Null)
                } else if let Some(&col_idx) = col_index_map.get(&name.to_lowercase()) {
                    if col_idx < row.len() {
                        row[col_idx].clone()
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Null
                }
            }
            sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
                let left_val = self.evaluate_row_expression(left, row, col_index_map, ctx);
                let right_val = self.evaluate_row_expression(right, row, col_index_map, ctx);
                self.evaluate_binary_op(&left_val, &right_val, op)
            }
            sqlrustgo_parser::Expression::UnaryOp(op, inner) => {
                let val = self.evaluate_row_expression(inner, row, col_index_map, ctx);
                match op.as_str() {
                    "NOT" => {
                        if let Value::Boolean(b) = val {
                            Value::Boolean(!b)
                        } else {
                            Value::Boolean(false)
                        }
                    }
                    _ => Value::Null,
                }
            }
            sqlrustgo_parser::Expression::IsNull(inner) => {
                let val = self.evaluate_row_expression(inner, row, col_index_map, ctx);
                Value::Boolean(matches!(val, Value::Null))
            }
            sqlrustgo_parser::Expression::IsNotNull(inner) => {
                let val = self.evaluate_row_expression(inner, row, col_index_map, ctx);
                Value::Boolean(!matches!(val, Value::Null))
            }
            sqlrustgo_parser::Expression::Like(left, pattern, _) => {
                let left_val = self.evaluate_row_expression(left, row, col_index_map, ctx);
                let pattern_val = self.evaluate_row_expression(pattern, row, col_index_map, ctx);
                Value::Boolean(self.like_match(&left_val, &pattern_val))
            }
            sqlrustgo_parser::Expression::NotLike(left, pattern, _) => {
                let left_val = self.evaluate_row_expression(left, row, col_index_map, ctx);
                let pattern_val = self.evaluate_row_expression(pattern, row, col_index_map, ctx);
                let like_result = self.like_match(&left_val, &pattern_val);
                Value::Boolean(!like_result)
            }
            sqlrustgo_parser::Expression::Between(left, low, high) => {
                let left_val = self.evaluate_row_expression(left, row, col_index_map, ctx);
                let low_val = self.evaluate_row_expression(low, row, col_index_map, ctx);
                let high_val = self.evaluate_row_expression(high, row, col_index_map, ctx);
                Value::Boolean(self.between_match(&left_val, &low_val, &high_val))
            }
            sqlrustgo_parser::Expression::NotBetween(left, low, high) => {
                let left_val = self.evaluate_row_expression(left, row, col_index_map, ctx);
                let low_val = self.evaluate_row_expression(low, row, col_index_map, ctx);
                let high_val = self.evaluate_row_expression(high, row, col_index_map, ctx);
                Value::Boolean(!self.between_match(&left_val, &low_val, &high_val))
            }
            sqlrustgo_parser::Expression::In(left, select) => {
                let left_val = self.evaluate_row_expression(left, row, col_index_map, ctx);
                let rows = self.execute_subquery(select);
                let in_result = rows
                    .iter()
                    .any(|r| r.first().map(|v| v == &left_val).unwrap_or(false));
                Value::Boolean(in_result)
            }
            sqlrustgo_parser::Expression::NotIn(left, select) => {
                let left_val = self.evaluate_row_expression(left, row, col_index_map, ctx);
                let rows = self.execute_subquery(select);
                let in_result = rows
                    .iter()
                    .any(|r| r.first().map(|v| v == &left_val).unwrap_or(false));
                Value::Boolean(!in_result)
            }
            sqlrustgo_parser::Expression::InList(left, values) => {
                let left_val = self.evaluate_row_expression(left, row, col_index_map, ctx);
                let value_list: Vec<Value> = values
                    .iter()
                    .map(|v| self.evaluate_row_expression(v, row, col_index_map, ctx))
                    .collect();
                Value::Boolean(value_list.contains(&left_val))
            }
            sqlrustgo_parser::Expression::NotInList(left, values) => {
                let left_val = self.evaluate_row_expression(left, row, col_index_map, ctx);
                let value_list: Vec<Value> = values
                    .iter()
                    .map(|v| self.evaluate_row_expression(v, row, col_index_map, ctx))
                    .collect();
                Value::Boolean(!value_list.contains(&left_val))
            }
            sqlrustgo_parser::Expression::CaseWhen(when_clauses, else_expr) => {
                for clause in when_clauses {
                    let cond_val =
                        self.evaluate_row_expression(&clause.condition, row, col_index_map, ctx);
                    if let Value::Boolean(true) = cond_val {
                        return self.evaluate_row_expression(
                            &clause.result,
                            row,
                            col_index_map,
                            ctx,
                        );
                    }
                }
                if let Some(else_box) = else_expr {
                    self.evaluate_row_expression(else_box, row, col_index_map, ctx)
                } else {
                    Value::Null
                }
            }
            sqlrustgo_parser::Expression::Exists(select) => {
                let rows = self.execute_subquery(select);
                Value::Boolean(!rows.is_empty())
            }
            sqlrustgo_parser::Expression::NotExists(select) => {
                let rows = self.execute_subquery(select);
                Value::Boolean(rows.is_empty())
            }
            sqlrustgo_parser::Expression::QuantifiedOp(expr, quantifier, select) => {
                let expr_val = self.evaluate_row_expression(expr, row, col_index_map, ctx);
                let rows = self.execute_subquery(select);
                let all_match = rows
                    .iter()
                    .all(|r| r.first().map(|v| v == &expr_val).unwrap_or(false));
                let any_match = rows
                    .iter()
                    .any(|r| r.first().map(|v| v == &expr_val).unwrap_or(false));
                match quantifier.as_str() {
                    "ALL" => Value::Boolean(all_match),
                    "ANY" | "SOME" => Value::Boolean(any_match),
                    _ => Value::Null,
                }
            }
            sqlrustgo_parser::Expression::Subquery(select) => {
                let rows = self.execute_subquery(select);
                if let Some(first_row) = rows.first() {
                    first_row.first().cloned().unwrap_or(Value::Null)
                } else {
                    Value::Null
                }
            }
            sqlrustgo_parser::Expression::FunctionCall(name, args) => {
                let name_upper = name.to_uppercase();
                if name_upper == "SUBSTRING" {
                    if args.is_empty() {
                        Value::Null
                    } else {
                        let str_val =
                            self.evaluate_row_expression(&args[0], row, col_index_map, ctx);
                        let start_val = if args.len() > 1 {
                            self.evaluate_row_expression(&args[1], row, col_index_map, ctx)
                        } else {
                            Value::Null
                        };
                        let len_val = if args.len() > 2 {
                            Some(self.evaluate_row_expression(&args[2], row, col_index_map, ctx))
                        } else {
                            None
                        };
                        match (str_val, start_val) {
                            (Value::Text(s), Value::Integer(start)) => {
                                let start_idx = ((start - 1).max(0)) as usize;
                                let result: String = if let Some(Value::Integer(len)) = len_val {
                                    s.chars().skip(start_idx).take(len as usize).collect()
                                } else {
                                    s.chars().skip(start_idx).collect()
                                };
                                Value::Text(result)
                            }
                            _ => Value::Null,
                        }
                    }
                } else {
                    Value::Null
                }
            }
            sqlrustgo_parser::Expression::Extract(field, inner) => {
                let val = self.evaluate_row_expression(inner, row, col_index_map, ctx);
                match (field.as_str(), &val) {
                    ("YEAR", Value::Integer(n)) => Value::Integer(*n / 10000),
                    ("MONTH", Value::Integer(n)) => Value::Integer((*n / 100) % 100),
                    ("DAY", Value::Integer(n)) => Value::Integer(*n % 100),
                    ("YEAR", Value::Text(s)) => {
                        s[..4].parse().map(Value::Integer).unwrap_or(Value::Null)
                    }
                    ("MONTH", Value::Text(s)) if s.len() >= 7 => {
                        s[5..7].parse().map(Value::Integer).unwrap_or(Value::Null)
                    }
                    ("DAY", Value::Text(s)) if s.len() >= 10 => {
                        s[8..10].parse().map(Value::Integer).unwrap_or(Value::Null)
                    }
                    _ => Value::Null,
                }
            }
            _ => Value::Null,
        }
    }

    /// Evaluate expression with row values directly
    pub(crate) fn evaluate_expression_with_row(
        &self,
        expr: &sqlrustgo_parser::Expression,
        row: &[Value],
    ) -> Value {
        match expr {
            sqlrustgo_parser::Expression::Identifier(name) => {
                if let Some(stripped) = name.strip_prefix('@') {
                    return Value::Text(stripped.to_string());
                }
                if let Ok(n) = name.parse::<i64>() {
                    if n as usize >= row.len() {
                        return Value::Null;
                    }
                    return row.get(n as usize).cloned().unwrap_or(Value::Null);
                }
                if let Ok(idx) = name.parse::<usize>() {
                    if idx >= row.len() {
                        return Value::Null;
                    }
                    return row.get(idx).cloned().unwrap_or(Value::Null);
                }
                Value::Text(name.to_string())
            }
            sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
                let left_val = self.evaluate_expression_with_row(left, row);
                let right_val = self.evaluate_expression_with_row(right, row);
                self.evaluate_binary_op(&left_val, &right_val, op)
            }
            sqlrustgo_parser::Expression::Literal(s) => {
                let s = s.trim();
                if s.eq_ignore_ascii_case("NULL") {
                    Value::Null
                } else if let Ok(n) = s.parse::<i64>() {
                    Value::Integer(n)
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Float(f)
                } else if s.starts_with('\'') && s.ends_with('\'') {
                    Value::Text(s[1..s.len() - 1].to_string())
                } else {
                    Value::Text(s.to_string())
                }
            }
            sqlrustgo_parser::Expression::UnaryOp(op, inner) => {
                let val = self.evaluate_expression_with_row(inner, row);
                match op.to_uppercase().as_str() {
                    "-" => {
                        if let Value::Integer(n) = val {
                            Value::Integer(-n)
                        } else if let Value::Float(f) = val {
                            Value::Float(-f)
                        } else {
                            Value::Null
                        }
                    }
                    "NOT" | "!" => {
                        if let Value::Boolean(b) = val {
                            Value::Boolean(!b)
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

    /// Evaluate expression with column binding (for recursive CTE)
    pub(crate) fn evaluate_expression_with_binding(
        &self,
        expr: &sqlrustgo_parser::Expression,
        row: &[Value],
        column_bindings: &[(String, usize)],
    ) -> Value {
        match expr {
            sqlrustgo_parser::Expression::Identifier(name) => {
                if let Some(stripped) = name.strip_prefix('@') {
                    return Value::Text(stripped.to_string());
                }
                for (col_name, col_idx) in column_bindings {
                    if col_name == name {
                        if *col_idx < row.len() {
                            return row[*col_idx].clone();
                        }
                        return Value::Null;
                    }
                }
                if let Ok(n) = name.parse::<i64>() {
                    if n as usize >= row.len() {
                        return Value::Null;
                    }
                    return row.get(n as usize).cloned().unwrap_or(Value::Null);
                }
                Value::Text(name.to_string())
            }
            sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
                let left_val = self.evaluate_expression_with_binding(left, row, column_bindings);
                let right_val = self.evaluate_expression_with_binding(right, row, column_bindings);
                self.evaluate_binary_op(&left_val, &right_val, op)
            }
            sqlrustgo_parser::Expression::Literal(s) => {
                let s = s.trim();
                if s.eq_ignore_ascii_case("NULL") {
                    Value::Null
                } else if let Ok(n) = s.parse::<i64>() {
                    Value::Integer(n)
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Float(f)
                } else if s.starts_with('\'') && s.ends_with('\'') {
                    Value::Text(s[1..s.len() - 1].to_string())
                } else {
                    Value::Text(s.to_string())
                }
            }
            sqlrustgo_parser::Expression::UnaryOp(op, inner) => {
                let val = self.evaluate_expression_with_binding(inner, row, column_bindings);
                match op.to_uppercase().as_str() {
                    "-" => {
                        if let Value::Integer(n) = val {
                            Value::Integer(-n)
                        } else if let Value::Float(f) = val {
                            Value::Float(-f)
                        } else {
                            Value::Null
                        }
                    }
                    "NOT" | "!" => {
                        if let Value::Boolean(b) = val {
                            Value::Boolean(!b)
                        } else {
                            Value::Null
                        }
                    }
                    _ => Value::Null,
                }
            }
            _ => self.evaluate_expression_with_row(expr, row),
        }
    }

    /// Evaluate expression with context (allows column bindings from CTE)
    pub(crate) fn expression_to_value_with_context(
        &self,
        expr: &sqlrustgo_parser::Expression,
        ctx: &ProcedureContext,
    ) -> Value {
        match expr {
            sqlrustgo_parser::Expression::Identifier(name) => {
                if let Some(stripped) = name.strip_prefix('@') {
                    ctx.get_var(stripped).cloned().unwrap_or(Value::Null)
                } else {
                    ctx.get_var(name).cloned().unwrap_or_else(|| {
                        if let Ok(n) = name.parse::<i64>() {
                            Value::Integer(n)
                        } else if let Ok(f) = name.parse::<f64>() {
                            Value::Float(f)
                        } else {
                            Value::Text(name.to_string())
                        }
                    })
                }
            }
            sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
                let left_val = self.expression_to_value_with_context(left, ctx);
                let right_val = self.expression_to_value_with_context(right, ctx);
                self.evaluate_binary_op(&left_val, &right_val, op)
            }
            sqlrustgo_parser::Expression::Literal(s) => {
                let s = s.trim();
                if s.eq_ignore_ascii_case("NULL") {
                    Value::Null
                } else if let Ok(n) = s.parse::<i64>() {
                    Value::Integer(n)
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Float(f)
                } else if s.starts_with('\'') && s.ends_with('\'') {
                    Value::Text(s[1..s.len() - 1].to_string())
                } else {
                    Value::Text(s.to_string())
                }
            }
            sqlrustgo_parser::Expression::UnaryOp(op, inner) => {
                let val = self.expression_to_value_with_context(inner, ctx);
                match op.to_uppercase().as_str() {
                    "-" => {
                        if let Value::Integer(n) = val {
                            Value::Integer(-n)
                        } else if let Value::Float(f) = val {
                            Value::Float(-f)
                        } else {
                            Value::Null
                        }
                    }
                    "NOT" | "!" => {
                        if let Value::Boolean(b) = val {
                            Value::Boolean(!b)
                        } else {
                            Value::Null
                        }
                    }
                    _ => Value::Null,
                }
            }
            _ => self.expression_to_value(expr, ctx),
        }
    }

    /// Check if a Value is true (for WHERE evaluation)
    pub(crate) fn value_is_true(&self, val: &Value) -> bool {
        match val {
            Value::Boolean(b) => *b,
            Value::Null => false,
            _ => true,
        }
    }

    /// Evaluate a condition expression (returns boolean)
    pub(crate) fn evaluate_condition(
        &self,
        condition: &str,
        ctx: &ProcedureContext,
    ) -> Result<bool, String> {
        let cond = condition.trim();

        for &op in &["<=", ">=", "!=", "<>", "=", "<", ">"] {
            if let Some(pos) = cond.find(op) {
                let left = cond[..pos].trim();
                let right = cond[pos + op.len()..].trim();

                let left_val = self.expand_variable(left, ctx);
                let right_val = self.evaluate_constant(right);

                return Ok(self.compare_values(&left_val, &right_val, op));
            }
        }

        Ok(cond != "0" && cond.to_lowercase() != "false" && !cond.is_empty())
    }

    /// Evaluate an expression and return a Value
    pub(crate) fn evaluate_expression(
        &self,
        expr: &str,
        ctx: &ProcedureContext,
    ) -> Result<Value, String> {
        let expr = expr.trim();

        if expr.starts_with('\'') && expr.ends_with('\'') {
            return Ok(Value::Text(expr[1..expr.len() - 1].to_string()));
        }

        if let Ok(num) = expr.parse::<i64>() {
            return Ok(Value::Integer(num));
        }
        if let Ok(float) = expr.parse::<f64>() {
            return Ok(Value::Float(float));
        }

        if ctx.has_var(expr) {
            return Ok(self.expand_variable(expr, ctx));
        }

        for &op in &["+", "-", "*", "/"] {
            if let Some(pos) = expr.find(op) {
                if pos > 0 && pos < expr.len() - 1 {
                    let left = self.evaluate_expression(&expr[..pos], ctx)?;
                    let right = self.evaluate_expression(&expr[pos + 1..], ctx)?;
                    return self.arithmetic_op(&left, &right, op);
                }
            }
        }

        Ok(Value::Text(expr.to_string()))
    }

    /// Expand a variable reference to its value
    pub(crate) fn expand_variable(&self, name: &str, ctx: &ProcedureContext) -> Value {
        let name = name.trim();

        if let Some(var_name) = name.strip_prefix('@') {
            ctx.get_var(var_name).cloned().unwrap_or(Value::Null)
        } else if ctx.has_var(name) {
            ctx.get_var(name).cloned().unwrap_or(Value::Null)
        } else {
            self.evaluate_constant(name)
        }
    }

    /// Expand variables in SQL string
    pub(crate) fn expand_variables_in_sql(&self, sql: &str, ctx: &ProcedureContext) -> String {
        let chars: Vec<char> = sql.chars().collect();
        let mut result = String::new();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '@' {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let var_name: String = chars[start + 1..i].iter().collect();
                let value = ctx
                    .get_var(&var_name)
                    .map(|v| self.escape_sql_value(v))
                    .unwrap_or_else(|| "NULL".to_string());
                result.push_str(&value);
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    /// Escape SQL value for interpolation
    pub(crate) fn escape_sql_value(&self, value: &Value) -> String {
        match value {
            Value::Text(s) => s.replace('\'', "''"),
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            Value::Null => "NULL".to_string(),
            Value::Blob(b) => b
                .iter()
                .map(|&x| x as char)
                .collect::<String>()
                .replace('\'', "''"),
            Value::Geometry(g) => g.to_string(),
        }
    }

    /// Evaluate a constant expression
    pub(crate) fn evaluate_constant(&self, value: &str) -> Value {
        let value = value.trim();

        if value.starts_with('\'') && value.ends_with('\'') {
            return Value::Text(value[1..value.len() - 1].to_string());
        }

        if let Ok(num) = value.parse::<i64>() {
            return Value::Integer(num);
        }
        if let Ok(float) = value.parse::<f64>() {
            return Value::Float(float);
        }

        if value.to_uppercase() == "NULL" {
            return Value::Null;
        }

        if value.to_uppercase() == "TRUE" {
            return Value::Boolean(true);
        }
        if value.to_uppercase() == "FALSE" {
            return Value::Boolean(false);
        }

        Value::Text(value.to_string())
    }

    /// Evaluate a binary operation and return a boolean Value
    pub(crate) fn evaluate_binary_op(&self, left: &Value, right: &Value, op: &str) -> Value {
        if matches!(left, Value::Null) || matches!(right, Value::Null) {
            return Value::Null;
        }

        match op {
            "=" | "==" | "IS" => Value::Boolean(left == right),
            "!=" | "<>" => Value::Boolean(left != right),
            ">" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Boolean(l > r)
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Boolean(l > r)
                } else if let (Value::Text(l), Value::Text(r)) = (left, right) {
                    Value::Boolean(l > r)
                } else {
                    Value::Boolean(false)
                }
            }
            ">=" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Boolean(l >= r)
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Boolean(l >= r)
                } else if let (Value::Text(l), Value::Text(r)) = (left, right) {
                    Value::Boolean(l >= r)
                } else {
                    Value::Boolean(false)
                }
            }
            "<" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Boolean(l < r)
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Boolean(l < r)
                } else if let (Value::Text(l), Value::Text(r)) = (left, right) {
                    Value::Boolean(l < r)
                } else {
                    Value::Boolean(false)
                }
            }
            "<=" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Boolean(l <= r)
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Boolean(l <= r)
                } else if let (Value::Text(l), Value::Text(r)) = (left, right) {
                    Value::Boolean(l <= r)
                } else {
                    Value::Boolean(false)
                }
            }
            "AND" | "&&" => {
                if let (Value::Boolean(l), Value::Boolean(r)) = (left, right) {
                    Value::Boolean(*l && *r)
                } else if matches!(left, Value::Null) || matches!(right, Value::Null) {
                    if let Value::Boolean(false) = left {
                        Value::Boolean(false)
                    } else if let Value::Boolean(false) = right {
                        Value::Boolean(false)
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Boolean(false)
                }
            }
            "OR" | "||" => {
                if let (Value::Boolean(l), Value::Boolean(r)) = (left, right) {
                    Value::Boolean(*l || *r)
                } else if matches!(left, Value::Null) || matches!(right, Value::Null) {
                    if let Value::Boolean(true) = left {
                        Value::Boolean(true)
                    } else if let Value::Boolean(true) = right {
                        Value::Boolean(true)
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Boolean(false)
                }
            }
            "+" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Integer(l.wrapping_add(*r))
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Float(l + r)
                } else if let (Value::Integer(l), Value::Float(r)) = (left, right) {
                    Value::Float(*l as f64 + r)
                } else if let (Value::Float(l), Value::Integer(r)) = (left, right) {
                    Value::Float(l + *r as f64)
                } else {
                    Value::Null
                }
            }
            "-" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Integer(l.wrapping_sub(*r))
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Float(l - r)
                } else if let (Value::Integer(l), Value::Float(r)) = (left, right) {
                    Value::Float(*l as f64 - r)
                } else if let (Value::Float(l), Value::Integer(r)) = (left, right) {
                    Value::Float(l - *r as f64)
                } else {
                    Value::Null
                }
            }
            "*" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Integer(l.wrapping_mul(*r))
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Float(l * r)
                } else if let (Value::Integer(l), Value::Float(r)) = (left, right) {
                    Value::Float(*l as f64 * r)
                } else if let (Value::Float(l), Value::Integer(r)) = (left, right) {
                    Value::Float(l * *r as f64)
                } else {
                    Value::Null
                }
            }
            "/" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    if *r == 0 {
                        Value::Null
                    } else {
                        Value::Integer(l / r)
                    }
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    if *r == 0.0 {
                        Value::Null
                    } else {
                        Value::Float(l / r)
                    }
                } else if let (Value::Integer(l), Value::Float(r)) = (left, right) {
                    if *r == 0.0 {
                        Value::Null
                    } else {
                        Value::Float(*l as f64 / r)
                    }
                } else if let (Value::Float(l), Value::Integer(r)) = (left, right) {
                    if *r == 0 {
                        Value::Null
                    } else {
                        Value::Float(l / *r as f64)
                    }
                } else {
                    Value::Null
                }
            }
            _ => Value::Null,
        }
    }

    /// LIKE pattern matching (supports % and _ wildcards)
    pub(crate) fn like_match(&self, left: &Value, pattern: &Value) -> bool {
        let (text, pat) = match (left, pattern) {
            (Value::Text(t), Value::Text(p)) => (t.as_str(), p.as_str()),
            _ => return false,
        };
        fn do_match(text: &str, pat: &str) -> bool {
            let t_bytes = text.as_bytes();
            let p_bytes = pat.as_bytes();
            let mut i = 0;
            let mut j = 0;
            let mut stack: Vec<(usize, usize)> = vec![];

            loop {
                if j >= p_bytes.len() {
                    if i >= t_bytes.len() {
                        return true;
                    }
                    if let Some((prev_i, prev_j)) = stack.pop() {
                        if prev_i < t_bytes.len() {
                            stack.push((prev_i + 1, prev_j));
                        }
                        i = prev_i + 1;
                        j = prev_j + 1;
                        if j < p_bytes.len() && p_bytes[j] == b'%' {
                            j += 1;
                            if j >= p_bytes.len() {
                                return true;
                            }
                        }
                        continue;
                    }
                    return false;
                }
                if i >= t_bytes.len() {
                    while j < p_bytes.len() && p_bytes[j] == b'%' {
                        j += 1;
                    }
                    return j >= p_bytes.len();
                }

                match p_bytes[j] {
                    b'%' => {
                        stack.push((i, j));
                        j += 1;
                        if j >= p_bytes.len() {
                            return true;
                        }
                    }
                    b'_' => {
                        i += 1;
                        j += 1;
                    }
                    c => {
                        let text_lower = t_bytes[i].to_ascii_lowercase();
                        let pat_lower = c.to_ascii_lowercase();
                        if text_lower == pat_lower {
                            i += 1;
                            j += 1;
                        } else {
                            if let Some((prev_i, prev_j)) = stack.pop() {
                                if prev_i < t_bytes.len() {
                                    stack.push((prev_i + 1, prev_j));
                                }
                                i = prev_i + 1;
                                j = prev_j + 1;
                                if j < p_bytes.len() && p_bytes[j] == b'%' {
                                    j += 1;
                                }
                                continue;
                            }
                            return false;
                        }
                    }
                }
            }
        }

        do_match(text, pat)
    }

    /// BETWEEN check: left >= low AND left <= high
    pub(crate) fn between_match(&self, left: &Value, low: &Value, high: &Value) -> bool {
        let ge_result = self.ge_values(left, low);
        let le_result = self.le_values(left, high);
        ge_result && le_result
    }

    /// REGEXP pattern matching
    pub(crate) fn regexp_match(&self, left: &Value, pattern: &Value) -> bool {
        let (text, pat) = match (left, pattern) {
            (Value::Text(t), Value::Text(p)) => (t.as_str(), p.as_str()),
            _ => return false,
        };
        let text_lower = text.to_lowercase();
        let pat_lower = pat.to_lowercase();
        text_lower.contains(&pat_lower)
    }

    /// Greater-than-or-equal comparison helper
    pub(crate) fn ge_values(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l >= r,
            (Value::Float(l), Value::Float(r)) => l >= r,
            (Value::Text(l), Value::Text(r)) => l >= r,
            _ => false,
        }
    }

    /// Less-than-or-equal comparison helper
    pub(crate) fn le_values(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l <= r,
            (Value::Float(l), Value::Float(r)) => l <= r,
            (Value::Text(l), Value::Text(r)) => l <= r,
            _ => false,
        }
    }

    /// Compare two values using the given operator
    pub(crate) fn compare_values(&self, left: &Value, right: &Value, op: &str) -> bool {
        match op {
            "=" | "==" => left == right,
            "!=" | "<>" => left != right,
            ">" => self.partial_cmp(left, right) == Some(std::cmp::Ordering::Greater),
            ">=" => matches!(
                self.partial_cmp(left, right),
                Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal)
            ),
            "<" => self.partial_cmp(left, right) == Some(std::cmp::Ordering::Less),
            "<=" => matches!(
                self.partial_cmp(left, right),
                Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal)
            ),
            _ => false,
        }
    }

    /// Compare two values (for ordering)
    pub(crate) fn partial_cmp(&self, left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;

        match (left, right) {
            (Value::Null, _) | (_, Value::Null) => None,
            (Value::Integer(l), Value::Integer(r)) => Some(l.cmp(r)),
            (Value::Float(l), Value::Float(r)) => Some(l.partial_cmp(r).unwrap_or(Ordering::Equal)),
            (Value::Text(l), Value::Text(r)) => Some(l.cmp(r)),
            (Value::Boolean(l), Value::Boolean(r)) => Some(l.cmp(r)),
            _ => None,
        }
    }

    /// Perform arithmetic operation on two values
    pub(crate) fn arithmetic_op(
        &self,
        left: &Value,
        right: &Value,
        op: &str,
    ) -> Result<Value, String> {
        match (left, right, op) {
            (Value::Integer(l), Value::Integer(r), "+") => Ok(Value::Integer(l + r)),
            (Value::Integer(l), Value::Integer(r), "-") => Ok(Value::Integer(l - r)),
            (Value::Integer(l), Value::Integer(r), "*") => Ok(Value::Integer(l * r)),
            (Value::Integer(l), Value::Integer(r), "/") => {
                if *r == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Integer(l / r))
                }
            }
            (Value::Float(l), Value::Float(r), "+") => Ok(Value::Float(l + r)),
            (Value::Float(l), Value::Float(r), "-") => Ok(Value::Float(l - r)),
            (Value::Float(l), Value::Float(r), "*") => Ok(Value::Float(l * r)),
            (Value::Float(l), Value::Float(r), "/") => {
                if *r == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Float(l / r))
                }
            }
            _ => Err(format!(
                "Unsupported arithmetic operation: {:?} {} {:?}",
                left, op, right
            )),
        }
    }

    /// Recursively extract identifier names from an expression
    pub(crate) fn extract_identifiers_from_expr(
        &self,
        expr: &sqlrustgo_parser::Expression,
    ) -> Vec<String> {
        let mut ids = Vec::new();
        match expr {
            sqlrustgo_parser::Expression::Identifier(name) => {
                ids.push(name.clone());
            }
            sqlrustgo_parser::Expression::BinaryOp(left, _, right) => {
                ids.extend(self.extract_identifiers_from_expr(left));
                ids.extend(self.extract_identifiers_from_expr(right));
            }
            sqlrustgo_parser::Expression::UnaryOp(_, inner) => {
                ids.extend(self.extract_identifiers_from_expr(inner));
            }
            sqlrustgo_parser::Expression::IsNull(inner) => {
                ids.extend(self.extract_identifiers_from_expr(inner));
            }
            sqlrustgo_parser::Expression::IsNotNull(inner) => {
                ids.extend(self.extract_identifiers_from_expr(inner));
            }
            sqlrustgo_parser::Expression::Like(left, pattern, _) => {
                ids.extend(self.extract_identifiers_from_expr(left));
                ids.extend(self.extract_identifiers_from_expr(pattern));
            }
            sqlrustgo_parser::Expression::NotLike(left, pattern, _) => {
                ids.extend(self.extract_identifiers_from_expr(left));
                ids.extend(self.extract_identifiers_from_expr(pattern));
            }
            sqlrustgo_parser::Expression::Between(expr, low, high) => {
                ids.extend(self.extract_identifiers_from_expr(expr));
                ids.extend(self.extract_identifiers_from_expr(low));
                ids.extend(self.extract_identifiers_from_expr(high));
            }
            sqlrustgo_parser::Expression::NotBetween(expr, low, high) => {
                ids.extend(self.extract_identifiers_from_expr(expr));
                ids.extend(self.extract_identifiers_from_expr(low));
                ids.extend(self.extract_identifiers_from_expr(high));
            }
            sqlrustgo_parser::Expression::InList(left, values) => {
                ids.extend(self.extract_identifiers_from_expr(left));
                for v in values {
                    ids.extend(self.extract_identifiers_from_expr(v));
                }
            }
            sqlrustgo_parser::Expression::CaseWhen(when_clauses, else_expr) => {
                for clause in when_clauses {
                    ids.extend(self.extract_identifiers_from_expr(&clause.condition));
                    ids.extend(self.extract_identifiers_from_expr(&clause.result));
                }
                if let Some(else_e) = else_expr {
                    ids.extend(self.extract_identifiers_from_expr(else_e));
                }
            }
            _ => {}
        }
        ids
    }
}
