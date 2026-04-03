//! Explain Executor - generates EXPLAIN output from physical plans
//!
//! This module provides functionality to display query execution plans
//! with estimated costs and actual execution times (when ANALYZE is used).

use sqlrustgo_planner::PhysicalPlan;
use sqlrustgo_planner::{
    AggregateExec, Expr, FilterExec, HashJoinExec, IndexScanExec, JoinType, LimitExec,
    ProjectionExec, SeqScanExec, SetOperationExec, SetOperationType, SortExec, SortMergeJoinExec,
    WindowExec,
};
use sqlrustgo_types::Value;
use std::time::Instant;

/// Configuration for EXPLAIN output
#[derive(Debug, Clone)]
pub struct ExplainConfig {
    /// Whether to analyze and collect actual execution times
    pub analyze: bool,
    /// Whether to show estimated rows
    pub show_estimated_rows: bool,
    /// Whether to show costs
    pub show_costs: bool,
    /// Format style for output
    pub format: ExplainFormat,
}

impl Default for ExplainConfig {
    fn default() -> Self {
        Self {
            analyze: false,
            show_estimated_rows: true,
            show_costs: true,
            format: ExplainFormat::Tree,
        }
    }
}

impl ExplainConfig {
    /// Create a new config for EXPLAIN (no analyze)
    pub fn explain() -> Self {
        Self::default()
    }

    /// Create a new config for EXPLAIN ANALYZE
    pub fn explain_analyze() -> Self {
        Self {
            analyze: true,
            ..Default::default()
        }
    }
}

/// Format style for EXPLAIN output
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExplainFormat {
    /// Tree-style format with arrows (->)
    Tree,
    /// Traditional format with IDs
    Traditional,
}

/// Explain output line
#[derive(Debug, Clone)]
pub struct ExplainLine {
    /// Indentation level (for tree format)
    pub indent: usize,
    /// The operator name
    pub operator: String,
    /// Additional details (table, filter, etc.)
    pub details: Vec<String>,
    /// Estimated rows
    pub estimated_rows: Option<u64>,
    /// Actual rows (only for ANALYZE mode)
    pub actual_rows: Option<u64>,
    /// Execution time in microseconds (only for ANALYZE mode)
    pub execution_time_us: Option<u64>,
}

impl ExplainLine {
    /// Format this line as tree-style output
    pub fn format_tree(&self) -> String {
        let prefix = "  ".repeat(self.indent);
        let arrow = "-> ";

        let mut line = format!("{}{}{}", prefix, arrow, self.operator);

        if !self.details.is_empty() {
            line.push_str(": ");
            line.push_str(&self.details.join(", "));
        }

        if let Some(rows) = self.estimated_rows {
            line.push_str(&format!(", estimated_rows={}", rows));
        }

        if let Some(rows) = self.actual_rows {
            line.push_str(&format!(", actual_rows={}", rows));
        }

        if let Some(us) = self.execution_time_us {
            if us >= 1000 {
                line.push_str(&format!(", time={:.3}ms", us as f64 / 1000.0));
            } else {
                line.push_str(&format!(", time={}µs", us));
            }
        }

        line
    }
}

/// Explain output container
#[derive(Debug, Clone)]
pub struct ExplainOutput {
    /// The plan lines in execution order (bottom-up for tree)
    pub lines: Vec<ExplainLine>,
    /// Total execution time in microseconds (only for ANALYZE)
    pub total_time_us: Option<u64>,
}

impl ExplainOutput {
    /// Create empty explain output
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            total_time_us: None,
        }
    }

    /// Convert to string using tree format
    pub fn to_string_tree(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.format_tree())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for ExplainOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// ExplainExecutor - generates EXPLAIN output from a physical plan
pub struct ExplainExecutor {
    config: ExplainConfig,
}

impl ExplainExecutor {
    /// Create a new ExplainExecutor with the given config
    pub fn new(config: ExplainConfig) -> Self {
        Self { config }
    }

    /// Create ExplainExecutor for EXPLAIN (no ANALYZE)
    pub fn explain() -> Self {
        Self::new(ExplainConfig::explain())
    }

    /// Create ExplainExecutor for EXPLAIN ANALYZE
    pub fn explain_analyze() -> Self {
        Self::new(ExplainConfig::explain_analyze())
    }

    /// Execute EXPLAIN on a physical plan
    pub fn explain_plan(&self, plan: &dyn PhysicalPlan) -> ExplainOutput {
        let mut output = ExplainOutput::new();
        let start_time = if self.config.analyze {
            Some(Instant::now())
        } else {
            None
        };

        self.explain_node(plan, 0, &mut output);

        if let Some(start) = start_time {
            output.total_time_us = Some(start.elapsed().as_micros() as u64);
        }

        output
    }

    /// Recursively explain a plan node
    fn explain_node(&self, plan: &dyn PhysicalPlan, depth: usize, output: &mut ExplainOutput) {
        // First, explain children (depth-first, post-order)
        for child in plan.children() {
            self.explain_node(child, depth + 1, output);
        }

        // Then explain this node
        let (name, details, estimated_rows, actual_rows, exec_time) = self.extract_node_info(plan);

        let line = ExplainLine {
            indent: depth,
            operator: name,
            details,
            estimated_rows: Some(estimated_rows),
            actual_rows,
            execution_time_us: exec_time,
        };

        output.lines.push(line);
    }

    /// Extract information from a plan node
    fn extract_node_info(
        &self,
        plan: &dyn PhysicalPlan,
    ) -> (String, Vec<String>, u64, Option<u64>, Option<u64>) {
        let name = plan.name().to_string();
        let (_, _, estimated_rows, _) = plan.estimated_cost();
        let mut details = Vec::new();
        let mut actual_rows = None;
        let mut exec_time = None;

        // Extract operator-specific details by downcasting
        if let Some(scan) = plan.as_any().downcast_ref::<SeqScanExec>() {
            details.push(format!("table={}", scan.table_name()));
            if let Some(proj) = scan.projection() {
                details.push(format!("columns={:?}", proj));
            }
        } else if let Some(scan) = plan.as_any().downcast_ref::<IndexScanExec>() {
            details.push(format!("table={}", scan.table_name()));
            details.push(format!("index={}", scan.index_name()));
            let (min, max) = scan.key_range();
            if let (Some(min), Some(max)) = (min, max) {
                details.push(format!("key_range=[{}..{}]", min, max));
            }
        } else if let Some(proj) = plan.as_any().downcast_ref::<ProjectionExec>() {
            let exprs = self.format_exprs(proj.expr());
            details.push(format!("expr=[{}]", exprs));
        } else if let Some(filter) = plan.as_any().downcast_ref::<FilterExec>() {
            let pred = self.format_expr(filter.predicate());
            details.push(format!("filter=[{}]", pred));
        } else if let Some(hash_join) = plan.as_any().downcast_ref::<HashJoinExec>() {
            details.push(format!(
                "type={}",
                self.format_join_type(hash_join.join_type())
            ));
            if let Some(cond) = hash_join.condition() {
                details.push(format!("condition=[{}]", self.format_expr(cond)));
            }
        } else if let Some(smj) = plan.as_any().downcast_ref::<SortMergeJoinExec>() {
            details.push(format!("type={}", self.format_join_type(smj.join_type())));
            if let Some(cond) = smj.condition() {
                details.push(format!("condition=[{}]", self.format_expr(cond)));
            }
        } else if let Some(agg) = plan.as_any().downcast_ref::<AggregateExec>() {
            if !agg.group_expr().is_empty() {
                let group_str = self.format_exprs(agg.group_expr());
                details.push(format!("group=[{}]", group_str));
            }
            if !agg.aggregate_expr().is_empty() {
                let agg_str = self.format_aggregate_exprs(agg.aggregate_expr());
                details.push(format!("aggs=[{}]", agg_str));
            }
        } else if let Some(sort) = plan.as_any().downcast_ref::<SortExec>() {
            let sort_str = sort
                .sort_expr()
                .iter()
                .map(|s| {
                    format!(
                        "{} {}",
                        self.format_expr(&s.expr),
                        if s.asc { "ASC" } else { "DESC" }
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            details.push(format!("sort=[{}]", sort_str));
        } else if let Some(limit) = plan.as_any().downcast_ref::<LimitExec>() {
            details.push(format!("limit={}", limit.limit()));
            if let Some(offset) = limit.offset() {
                details.push(format!("offset={}", offset));
            }
        } else if let Some(set_op) = plan.as_any().downcast_ref::<SetOperationExec>() {
            details.push(format!(
                "type={}",
                self.format_set_op_type(set_op.op_type())
            ));
        } else if let Some(window) = plan.as_any().downcast_ref::<WindowExec>() {
            let window_str = self.format_window_exprs(window.window_exprs());
            details.push(format!("windows=[{}]", window_str));
        }

        // If ANALYZE mode, execute the node to get actual stats
        if self.config.analyze {
            let start = Instant::now();
            let result = plan.execute();
            exec_time = Some(start.elapsed().as_micros() as u64);

            if let Ok(rows) = result {
                actual_rows = Some(rows.len() as u64);
            }
        }

        (name, details, estimated_rows, actual_rows, exec_time)
    }

    /// Format a join type
    fn format_join_type(&self, join_type: JoinType) -> String {
        match join_type {
            JoinType::Inner => "Inner",
            JoinType::Left => "Left",
            JoinType::Right => "Right",
            JoinType::Full => "Full",
            JoinType::Cross => "Cross",
            JoinType::LeftSemi => "LeftSemi",
            JoinType::LeftAnti => "LeftAnti",
            JoinType::RightSemi => "RightSemi",
            JoinType::RightAnti => "RightAnti",
        }
        .to_string()
    }

    /// Format a set operation type
    fn format_set_op_type(&self, op: SetOperationType) -> String {
        match op {
            SetOperationType::Union | SetOperationType::UnionAll => "Union",
            SetOperationType::Intersect => "Intersect",
            SetOperationType::Except => "Except",
        }
        .to_string()
    }

    /// Format an expression
    fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Column(col) => col.name.clone(),
            Expr::Literal(val) => self.format_value(val),
            Expr::BinaryExpr { left, op, right } => {
                format!(
                    "{} {} {}",
                    self.format_expr(left),
                    op,
                    self.format_expr(right)
                )
            }
            Expr::AggregateFunction { func, args, .. } => {
                let func_name = match func {
                    sqlrustgo_planner::AggregateFunction::Count => "COUNT",
                    sqlrustgo_planner::AggregateFunction::Sum => "SUM",
                    sqlrustgo_planner::AggregateFunction::Avg => "AVG",
                    sqlrustgo_planner::AggregateFunction::Min => "MIN",
                    sqlrustgo_planner::AggregateFunction::Max => "MAX",
                };
                let args_str = args
                    .iter()
                    .map(|a| self.format_expr(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", func_name, args_str)
            }
            Expr::Wildcard => "*".to_string(),
            Expr::QualifiedWildcard { .. } => ".*".to_string(),
            Expr::ScalarSubquery(_) => "scalar_subquery".to_string(),
            Expr::InSubquery { .. } => "in_subquery".to_string(),
            Expr::Exists(_) => "exists".to_string(),
            Expr::AnyAll { .. } => "any_all".to_string(),
            Expr::UnaryExpr { op, expr } => format!("({} {})", op, self.format_expr(expr)),
            Expr::WindowFunction { func, .. } => format!("{:?}", func),
            Expr::Alias { expr, name } => format!("{} AS {}", self.format_expr(expr), name),
            Expr::Parameter { index } => format!("?{}", index),
            Expr::Between { expr, low, high } => {
                format!(
                    "{} BETWEEN {} AND {}",
                    self.format_expr(expr),
                    self.format_expr(low),
                    self.format_expr(high)
                )
            }
            Expr::InList { expr, values } => {
                format!(
                    "{} IN ({})",
                    self.format_expr(expr),
                    self.format_exprs(values)
                )
            }
        }
    }

    /// Format multiple expressions
    fn format_exprs(&self, exprs: &[Expr]) -> String {
        exprs
            .iter()
            .map(|e| self.format_expr(e))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Format aggregate expressions
    fn format_aggregate_exprs(&self, exprs: &[Expr]) -> String {
        exprs
            .iter()
            .map(|e| self.format_expr(e))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Format window expressions
    fn format_window_exprs(&self, exprs: &[Expr]) -> String {
        exprs
            .iter()
            .map(|e| self.format_expr(e))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Format a value
    fn format_value(&self, val: &Value) -> String {
        match val {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Decimal(d) => d.to_string(),
            Value::Text(s) => format!("'{}'", s),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "NULL".to_string(),
            Value::Blob(b) => format!("blob[{}]", b.len()),
            Value::Date(_) => "date".to_string(),
            Value::Timestamp(_) => "timestamp".to_string(),
            Value::Uuid(u) => format!("'{:036x}'", u),
            Value::Array(_) => "array".to_string(),
            Value::Enum(_, name) => format!("'{}'", name),
        }
    }
}

/// Execute EXPLAIN on a physical plan with default config
pub fn explain(plan: &dyn PhysicalPlan) -> ExplainOutput {
    ExplainExecutor::explain().explain_plan(plan)
}

/// Execute EXPLAIN ANALYZE on a physical plan
pub fn explain_analyze(plan: &dyn PhysicalPlan) -> ExplainOutput {
    ExplainExecutor::explain_analyze().explain_plan(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{Field, Schema};

    fn create_test_seq_scan() -> SeqScanExec {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        SeqScanExec::new("users".to_string(), schema)
    }

    fn create_test_projection() -> ProjectionExec {
        let schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let input = Box::new(create_test_seq_scan());
        ProjectionExec::new(input, vec![Expr::column("id")], schema)
    }

    fn create_test_filter() -> FilterExec {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let input = Box::new(create_test_seq_scan());
        let predicate = Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: sqlrustgo_planner::Operator::Gt,
            right: Box::new(Expr::Literal(Value::Integer(1))),
        };
        FilterExec::new(input, predicate)
    }

    #[test]
    fn test_explain_seq_scan() {
        let scan = create_test_seq_scan();
        let output = explain(&scan);

        let lines = output.lines;
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].operator, "SeqScan");
        assert!(lines[0].details.contains(&"table=users".to_string()));
        assert!(lines[0].estimated_rows.is_some());
    }

    #[test]
    fn test_explain_projection() {
        let proj = create_test_projection();
        let output = explain(&proj);

        // Should have 2 lines: SeqScan (depth 1) then Projection (depth 0)
        assert_eq!(output.lines.len(), 2);
        // First line is SeqScan (child)
        assert_eq!(output.lines[0].operator, "SeqScan");
        assert_eq!(output.lines[0].indent, 1);
        // Second line is Projection
        assert_eq!(output.lines[1].operator, "Projection");
        assert_eq!(output.lines[1].indent, 0);
    }

    #[test]
    fn test_explain_filter() {
        let filter = create_test_filter();
        let output = explain(&filter);

        // Should have 2 lines: SeqScan then Filter
        assert_eq!(output.lines.len(), 2);
        assert_eq!(output.lines[1].operator, "Filter");
        assert!(output.lines[1]
            .details
            .iter()
            .any(|d| d.contains("filter=")));
    }

    #[test]
    fn test_explain_tree_format() {
        let scan = create_test_seq_scan();
        let output = explain(&scan);

        let tree = output.to_string_tree();
        assert!(tree.contains("-> SeqScan"));
        assert!(tree.contains("table=users"));
    }

    #[test]
    fn test_explain_config_defaults() {
        let config = ExplainConfig::default();
        assert!(!config.analyze);
        assert!(config.show_estimated_rows);
        assert!(config.show_costs);
        assert_eq!(config.format, ExplainFormat::Tree);
    }

    #[test]
    fn test_explain_explain_config() {
        let config = ExplainConfig::explain();
        assert!(!config.analyze);
    }

    #[test]
    fn test_explain_analyze_config() {
        let config = ExplainConfig::explain_analyze();
        assert!(config.analyze);
    }

    #[test]
    fn test_explain_explain_analyze_mode() {
        let scan = create_test_seq_scan();
        let output = explain_analyze(&scan);

        // In analyze mode, we should get actual execution info
        assert!(
            output.lines[0].actual_rows.is_some() || output.lines[0].execution_time_us.is_some()
        );
    }

    #[test]
    fn test_explain_hash_join() {
        let left_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let right_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);

        let left = Box::new(SeqScanExec::new("orders".to_string(), left_schema));
        let right = Box::new(SeqScanExec::new("users".to_string(), right_schema));

        let condition = Expr::BinaryExpr {
            left: Box::new(Expr::column("orders.id")),
            op: sqlrustgo_planner::Operator::Eq,
            right: Box::new(Expr::column("users.id")),
        };

        let join = HashJoinExec::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            Some(condition),
            join_schema,
        );

        let output = explain(&join);

        // Should have 3 lines: SeqScan (orders), SeqScan (users), HashJoin
        assert_eq!(output.lines.len(), 3);
        // Last line is HashJoin
        let hash_join_line = &output.lines[2];
        assert_eq!(hash_join_line.operator, "HashJoin");
        assert!(hash_join_line
            .details
            .iter()
            .any(|d| d.contains("type=Inner")));
        assert!(hash_join_line
            .details
            .iter()
            .any(|d| d.contains("condition=")));
    }

    #[test]
    fn test_explain_aggregate() {
        let input_schema = Schema::new(vec![
            Field::new("dept".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("salary".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);
        let agg_schema = Schema::new(vec![
            Field::new("dept".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("sum".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);

        let input = Box::new(SeqScanExec::new("employees".to_string(), input_schema));

        let group_expr = vec![Expr::column("dept")];
        let aggregate_expr = vec![Expr::AggregateFunction {
            func: sqlrustgo_planner::AggregateFunction::Sum,
            args: vec![Expr::column("salary")],
            distinct: false,
        }];

        let agg = AggregateExec::new(input, group_expr, aggregate_expr, None, agg_schema);

        let output = explain(&agg);

        // Should have 2 lines: SeqScan, Aggregate
        assert_eq!(output.lines.len(), 2);
        let agg_line = &output.lines[1];
        assert_eq!(agg_line.operator, "Aggregate");
        assert!(agg_line.details.iter().any(|d| d.contains("group=")));
        assert!(agg_line.details.iter().any(|d| d.contains("aggs=")));
    }

    #[test]
    fn test_explain_simple_select_with_all_operators() {
        // Simulate: SELECT id, name FROM users WHERE id > 1
        let scan_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);

        let scan = Box::new(SeqScanExec::new("users".to_string(), scan_schema.clone()));

        let predicate = Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: sqlrustgo_planner::Operator::Gt,
            right: Box::new(Expr::Literal(Value::Integer(1))),
        };
        let filter = FilterExec::new(scan, predicate);

        let proj_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let proj = ProjectionExec::new(Box::new(filter), vec![Expr::column("id")], proj_schema);

        let output = explain(&proj);

        // Should have 3 lines: SeqScan (depth 2), Filter (depth 1), Projection (depth 0)
        assert_eq!(output.lines.len(), 3);
        assert_eq!(output.lines[0].operator, "SeqScan");
        assert_eq!(output.lines[0].indent, 2);
        assert_eq!(output.lines[1].operator, "Filter");
        assert_eq!(output.lines[1].indent, 1);
        assert_eq!(output.lines[2].operator, "Projection");
        assert_eq!(output.lines[2].indent, 0);
    }

    #[test]
    fn test_explain_analyze_collects_timing() {
        let scan = create_test_seq_scan();
        let output = explain_analyze(&scan);

        // Should have timing information
        assert!(output.total_time_us.is_some());
        assert!(output.lines[0].execution_time_us.is_some());
    }

    #[test]
    fn test_explain_output_format_tree() {
        let scan = create_test_seq_scan();
        let output = explain(&scan);

        let tree = output.to_string_tree();

        // Check that tree format contains expected parts
        assert!(tree.contains("-> SeqScan"));
        assert!(tree.contains("table=users"));
        assert!(tree.contains("estimated_rows="));
    }

    #[test]
    fn test_explain_with_limit() {
        let input_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let scan = Box::new(SeqScanExec::new("users".to_string(), input_schema));
        let limit = LimitExec::new(scan, 10, Some(5));

        let output = explain(&limit);

        // Should have 2 lines: SeqScan, Limit
        assert_eq!(output.lines.len(), 2);
        let limit_line = &output.lines[1];
        assert_eq!(limit_line.operator, "Limit");
        assert!(limit_line.details.iter().any(|d| d.contains("limit=10")));
        assert!(limit_line.details.iter().any(|d| d.contains("offset=5")));
    }
}
