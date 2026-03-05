//! Cost-Based Optimization (CBO) Cost Model
//!
//! Provides cost estimation for query optimization decisions.

use super::logical_plan::LogicalPlan;
use super::{DataType, JoinType};
use crate::types::SqlResult;

/// Cost estimation for a logical plan
#[derive(Debug, Clone, Default)]
pub struct Cost {
    /// Estimated CPU cost
    pub cpu_cost: f64,
    /// Estimated I/O cost (pages to read)
    pub io_cost: f64,
    /// Estimated memory usage (bytes)
    pub memory_cost: f64,
    /// Estimated output row count
    pub row_count: f64,
}

impl Cost {
    pub fn new(cpu_cost: f64, io_cost: f64, memory_cost: f64, row_count: f64) -> Self {
        Self {
            cpu_cost,
            io_cost,
            memory_cost,
            row_count,
        }
    }

    /// Combine costs using addition
    pub fn add(&self, other: &Cost) -> Cost {
        Cost {
            cpu_cost: self.cpu_cost + other.cpu_cost,
            io_cost: self.io_cost + other.io_cost,
            memory_cost: self.memory_cost + other.memory_cost,
            row_count: self.row_count + other.row_count,
        }
    }

    /// Multiply cost by a factor
    pub fn multiply(&self, factor: f64) -> Cost {
        Cost {
            cpu_cost: self.cpu_cost * factor,
            io_cost: self.io_cost * factor,
            memory_cost: self.memory_cost * factor,
            row_count: self.row_count * factor,
        }
    }

    /// Total cost (weighted sum)
    pub fn total(&self, cpu_weight: f64, io_weight: f64, memory_weight: f64) -> f64 {
        cpu_weight * self.cpu_cost + io_weight * self.io_cost + memory_weight * self.memory_cost
    }
}

impl std::ops::Add for Cost {
    type Output = Cost;

    fn add(self, other: Cost) -> Cost {
        Cost {
            cpu_cost: self.cpu_cost + other.cpu_cost,
            io_cost: self.io_cost + other.io_cost,
            memory_cost: self.memory_cost + other.memory_cost,
            row_count: self.row_count + other.row_count,
        }
    }
}

impl std::ops::Mul<f64> for Cost {
    type Output = Cost;

    fn mul(self, factor: f64) -> Cost {
        self.multiply(factor)
    }
}

/// Table statistics for cost estimation
#[derive(Debug, Clone, Default)]
pub struct TableStats {
    /// Estimated number of rows
    pub row_count: f64,
    /// Number of pages (approx)
    pub page_count: f64,
    /// Row size in bytes (average)
    pub row_size: f64,
    /// Column statistics
    pub column_stats: Vec<ColumnStats>,
}

/// Column-level statistics
#[derive(Debug, Clone, Default)]
pub struct ColumnStats {
    /// Column name
    pub name: String,
    /// Data type
    pub data_type: DataType,
    /// Number of distinct values (NDV)
    pub ndv: f64,
    /// Min value (for numeric types)
    pub min_value: Option<String>,
    /// Max value (for numeric types)
    pub max_value: Option<String>,
    /// Null count
    pub null_count: f64,
    /// Histogram (for selectivity estimation)
    pub histogram: Vec<HistogramBucket>,
}

/// Histogram bucket for value distribution
#[derive(Debug, Clone)]
pub struct HistogramBucket {
    pub min: String,
    pub max: String,
    pub count: f64,
}

impl Default for HistogramBucket {
    fn default() -> Self {
        Self {
            min: String::new(),
            max: String::new(),
            count: 0.0,
        }
    }
}

/// Statistics provider trait
pub trait StatisticsProvider: Send + Sync {
    /// Get table statistics
    fn get_table_stats(&self, table_name: &str) -> Option<TableStats>;

    /// Get column statistics
    fn get_column_stats(&self, table_name: &str, column_name: &str) -> Option<ColumnStats>;

    /// Estimate selectivity of a filter expression
    fn estimate_selectivity(&self, table_name: &str, column_name: &str) -> f64;
}

/// Default statistics provider (no statistics)
pub struct DefaultStatisticsProvider;

impl DefaultStatisticsProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultStatisticsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticsProvider for DefaultStatisticsProvider {
    fn get_table_stats(&self, _table_name: &str) -> Option<TableStats> {
        // Return default estimates when no statistics available
        Some(TableStats {
            row_count: 1000.0,
            page_count: 10.0,
            row_size: 100.0,
            column_stats: vec![],
        })
    }

    fn get_column_stats(&self, _table_name: &str, _column_name: &str) -> Option<ColumnStats> {
        Some(ColumnStats {
            name: String::new(),
            data_type: DataType::Integer,
            ndv: 100.0,
            min_value: None,
            max_value: None,
            null_count: 0.0,
            histogram: vec![],
        })
    }

    fn estimate_selectivity(&self, _table_name: &str, _column_name: &str) -> f64 {
        // Default: assume 10% selectivity when no statistics
        0.1
    }
}

/// Cost model trait for CBO
pub trait CostModel: Send + Sync {
    /// Estimate cost for a logical plan
    fn estimate_cost(&self, plan: &LogicalPlan) -> SqlResult<Cost>;
}

/// Default cost model implementation
#[allow(dead_code)]
pub struct DefaultCostModel {
    stats_provider: Box<dyn StatisticsProvider>,
    // Cost weights for total cost calculation
    cpu_weight: f64,
    io_weight: f64,
    memory_weight: f64,
}

impl DefaultCostModel {
    pub fn new(stats_provider: Box<dyn StatisticsProvider>) -> Self {
        Self {
            stats_provider,
            // Default weights: I/O is most expensive
            cpu_weight: 1.0,
            io_weight: 10.0,
            memory_weight: 1.0,
        }
    }

    pub fn with_weights(
        stats_provider: Box<dyn StatisticsProvider>,
        cpu_weight: f64,
        io_weight: f64,
        memory_weight: f64,
    ) -> Self {
        Self {
            stats_provider,
            cpu_weight,
            io_weight,
            memory_weight,
        }
    }

    /// Calculate cost for table scan
    fn cost_table_scan(&self, plan: &super::LogicalPlan) -> SqlResult<Cost> {
        if let super::LogicalPlan::TableScan {
            ref table_name,
            ref filters,
            ref limit,
            schema: _,
            projection: _,
        } = plan
        {
            let stats = self.stats_provider.get_table_stats(table_name);
            let default_stats = TableStats::default();
            let base_stats = stats.as_ref().unwrap_or(&default_stats);

            // Base I/O cost = pages to read
            let mut io_cost = base_stats.page_count;

            // Filter selectivity
            let selectivity = if filters.is_empty() {
                1.0
            } else {
                filters
                    .iter()
                    .map(|f| self.estimate_expr_selectivity(f, table_name))
                    .fold(1.0, |acc, s| acc * s)
            };

            // Apply selectivity to row count
            let row_count = base_stats.row_count * selectivity;

            // Apply limit if present
            let row_count = match limit {
                Some(n) => (*n as f64).min(row_count),
                None => row_count,
            };

            // Update I/O cost based on selectivity
            io_cost *= selectivity;

            // CPU cost: scan cost per row
            let cpu_cost = base_stats.row_count * 0.001; // 0.001 CPU units per row

            // Memory cost: row size * row count
            let memory_cost = base_stats.row_size * row_count;

            Ok(Cost {
                cpu_cost,
                io_cost,
                memory_cost,
                row_count,
            })
        } else {
            Ok(Cost::default())
        }
    }

    /// Calculate cost for filter
    fn cost_filter(&self, plan: &super::LogicalPlan) -> SqlResult<Cost> {
        if let super::LogicalPlan::Filter {
            ref input,
            ref predicate,
        } = plan
        {
            let input_cost = match self.estimate_cost(input) {
                Ok(c) => c,
                Err(_) => return Ok(Cost::default()),
            };

            // Filter selectivity
            let table_name = self.extract_table_name(input);
            let selectivity = table_name
                .map(|t| self.estimate_expr_selectivity(predicate, &t))
                .unwrap_or(0.1);

            let row_count = input_cost.row_count * selectivity;

            // CPU cost for predicate evaluation
            let cpu_cost = input_cost.row_count * 0.01;

            Ok(Cost {
                cpu_cost: input_cost.cpu_cost + cpu_cost,
                io_cost: input_cost.io_cost,
                memory_cost: input_cost.memory_cost * selectivity,
                row_count,
            })
        } else {
            Ok(Cost::default())
        }
    }

    /// Calculate cost for join
    fn cost_join(&self, plan: &super::LogicalPlan) -> SqlResult<Cost> {
        if let super::LogicalPlan::Join {
            ref left,
            ref right,
            ref join_type,
            ref on,
            ..
        } = plan
        {
            let left_cost = self.estimate_cost(left)?;
            let right_cost = self.estimate_cost(right)?;

            // Join selectivity estimation
            let selectivity = self.estimate_join_selectivity(join_type, on);

            let output_rows = left_cost.row_count * right_cost.row_count * selectivity;

            let cpu_cost = match join_type {
                JoinType::Inner => {
                    // Nested loop join cost
                    left_cost.row_count * right_cost.row_count * 0.001
                        + left_cost.cpu_cost
                        + right_cost.cpu_cost
                }
                JoinType::Left | JoinType::Right => {
                    left_cost.row_count * right_cost.row_count * 0.0012
                        + left_cost.cpu_cost
                        + right_cost.cpu_cost
                }
                _ => {
                    left_cost.row_count * right_cost.row_count * 0.0015
                        + left_cost.cpu_cost
                        + right_cost.cpu_cost
                }
            };

            // Memory cost: need to hold join output
            let memory_cost = (left_cost.memory_cost + right_cost.memory_cost) * selectivity;

            Ok(Cost {
                cpu_cost,
                io_cost: left_cost.io_cost + right_cost.io_cost,
                memory_cost,
                row_count: output_rows,
            })
        } else {
            Ok(Cost::default())
        }
    }

    /// Calculate cost for aggregate
    fn cost_aggregate(&self, plan: &super::LogicalPlan) -> SqlResult<Cost> {
        if let super::LogicalPlan::Aggregate {
            ref input,
            ref group_expr,
            ref aggr_expr,
            ..
        } = plan
        {
            let input_cost = self.estimate_cost(input)?;

            // Grouping reduces row count
            // Estimate distinct groups based on group-by columns
            let group_count = if group_expr.is_empty() {
                1.0 // No grouping = single aggregate
            } else {
                // Assume ~10% cardinality reduction per group-by column
                (input_cost.row_count * 0.9_f64.powi(group_expr.len() as i32)).max(1.0)
            };

            let row_count = group_count;
            let cpu_cost =
                input_cost.row_count * (group_expr.len() as f64 + aggr_expr.len() as f64) * 0.01;
            let memory_cost = input_cost.memory_cost * 0.5; // Aggregates reduce memory

            Ok(Cost {
                cpu_cost: input_cost.cpu_cost + cpu_cost,
                io_cost: input_cost.io_cost,
                memory_cost,
                row_count,
            })
        } else {
            Ok(Cost::default())
        }
    }

    /// Calculate cost for projection
    fn cost_projection(&self, plan: &super::LogicalPlan) -> SqlResult<Cost> {
        if let super::LogicalPlan::Projection { ref input, .. } = plan {
            let input_cost = self.estimate_cost(input)?;
            // Projection is cheap - just column selection
            let cpu_cost = input_cost.row_count * 0.001;
            Ok(Cost {
                cpu_cost: input_cost.cpu_cost + cpu_cost,
                io_cost: input_cost.io_cost,
                memory_cost: input_cost.memory_cost * 0.8, // May reduce width
                row_count: input_cost.row_count,
            })
        } else {
            Ok(Cost::default())
        }
    }

    /// Calculate cost for sort
    fn cost_sort(&self, plan: &super::LogicalPlan) -> SqlResult<Cost> {
        if let super::LogicalPlan::Sort { ref input, .. } = plan {
            let input_cost = self.estimate_cost(input)?;
            // Sort cost: O(n log n)
            let cpu_cost = input_cost.row_count * input_cost.row_count.log2() * 0.0001;
            // Memory for sorting (may need to hold in memory)
            let memory_cost = input_cost.memory_cost * 2.0;
            Ok(Cost {
                cpu_cost: input_cost.cpu_cost + cpu_cost,
                io_cost: input_cost.io_cost,
                memory_cost,
                row_count: input_cost.row_count,
            })
        } else {
            Ok(Cost::default())
        }
    }

    /// Calculate cost for limit
    fn cost_limit(&self, plan: &super::LogicalPlan) -> SqlResult<Cost> {
        if let super::LogicalPlan::Limit { ref input, n } = plan {
            let input_cost = self.estimate_cost(input)?;
            let row_count = (*n as f64).min(input_cost.row_count);
            let cost = input_cost.multiply(row_count / input_cost.row_count.max(1.0));
            Ok(Cost { row_count, ..cost })
        } else {
            Ok(Cost::default())
        }
    }

    /// Extract table name from a plan (for statistics lookup)
    fn extract_table_name(&self, plan: &LogicalPlan) -> Option<String> {
        match plan {
            LogicalPlan::TableScan { table_name, .. } => Some(table_name.clone()),
            LogicalPlan::Filter { input, .. } => self.extract_table_name(input),
            LogicalPlan::Projection { input, .. } => self.extract_table_name(input),
            _ => None,
        }
    }

    /// Estimate selectivity of an expression
    fn estimate_expr_selectivity(&self, expr: &super::Expr, table_name: &str) -> f64 {
        use super::Expr::*;
        match expr {
            BinaryExpr { left, op, right } => {
                let left_selectivity = match &**left {
                    Column(col) => self
                        .stats_provider
                        .estimate_selectivity(table_name, &col.name),
                    _ => 0.5,
                };
                let right_selectivity = match &**right {
                    Column(col) => self
                        .stats_provider
                        .estimate_selectivity(table_name, &col.name),
                    _ => 0.5,
                };
                match op {
                    super::Operator::Eq => (left_selectivity + right_selectivity) / 2.0 * 0.1,
                    super::Operator::NotEq => {
                        1.0 - (left_selectivity + right_selectivity) / 2.0 * 0.1
                    }
                    super::Operator::Lt
                    | super::Operator::LtEq
                    | super::Operator::Gt
                    | super::Operator::GtEq => 0.25,
                    super::Operator::And => left_selectivity * right_selectivity,
                    super::Operator::Or => {
                        left_selectivity + right_selectivity - left_selectivity * right_selectivity
                    }
                    _ => 0.1,
                }
            }
            UnaryExpr {
                op: super::Operator::Not,
                ..
            } => 0.5,
            _ => 0.1,
        }
    }

    /// Estimate join selectivity
    fn estimate_join_selectivity(
        &self,
        join_type: &JoinType,
        on: &[(super::Expr, super::Expr)],
    ) -> f64 {
        if on.is_empty() {
            return match join_type {
                JoinType::Cross => 1.0, // Cartesian product
                _ => 0.1,               // No join condition
            };
        }

        // Default selectivity for join condition
        let base_selectivity = 0.1;

        match join_type {
            JoinType::Inner => base_selectivity,
            JoinType::Left => base_selectivity * 1.1, // Slightly more due to NULLs
            JoinType::Right => base_selectivity * 1.1,
            JoinType::Full => base_selectivity * 1.2,
            JoinType::Cross => 1.0,
            JoinType::LeftSemi | JoinType::LeftAnti => base_selectivity,
            JoinType::RightSemi | JoinType::RightAnti => base_selectivity,
        }
    }
}

impl CostModel for DefaultCostModel {
    fn estimate_cost(&self, plan: &LogicalPlan) -> SqlResult<Cost> {
        match plan {
            LogicalPlan::TableScan { .. } => self.cost_table_scan(plan),
            LogicalPlan::Filter { .. } => self.cost_filter(plan),
            LogicalPlan::Join { .. } => self.cost_join(plan),
            LogicalPlan::Aggregate { .. } => self.cost_aggregate(plan),
            LogicalPlan::Projection { .. } => self.cost_projection(plan),
            LogicalPlan::Sort { .. } => self.cost_sort(plan),
            LogicalPlan::Limit { .. } => self.cost_limit(plan),
            LogicalPlan::Values { values, .. } => Ok(Cost {
                row_count: values.len() as f64,
                ..Default::default()
            }),
            LogicalPlan::EmptyRelation {
                produce_one_row, ..
            } => Ok(Cost {
                row_count: if *produce_one_row { 1.0 } else { 0.0 },
                ..Default::default()
            }),
            LogicalPlan::Subquery { subquery, .. } => Ok(self.estimate_cost(subquery)?),
            LogicalPlan::Union { inputs, .. } => {
                let mut total_cost = Cost::default();
                for input in inputs {
                    total_cost = total_cost.add(&self.estimate_cost(input)?);
                }
                Ok(total_cost)
            }
            LogicalPlan::Update { input, .. } | LogicalPlan::Delete { input, .. } => {
                Ok(self.estimate_cost(input)?)
            }
            LogicalPlan::CreateTable { .. } | LogicalPlan::DropTable { .. } => Ok(Cost::default()),
        }
    }
}

/// No-op cost model that returns zero cost
pub struct NoOpCostModel;

impl CostModel for NoOpCostModel {
    fn estimate_cost(&self, _plan: &LogicalPlan) -> SqlResult<Cost> {
        Ok(Cost::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::{Column, Field, Schema};

    fn test_schema() -> Schema {
        Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("age".to_string(), DataType::Integer),
        ])
    }

    fn test_table_scan() -> LogicalPlan {
        LogicalPlan::TableScan {
            table_name: "users".to_string(),
            projection: None,
            filters: vec![],
            limit: None,
            schema: test_schema(),
        }
    }

    #[test]
    fn test_cost_creation() {
        let cost = Cost::new(100.0, 50.0, 200.0, 1000.0);
        assert_eq!(cost.cpu_cost, 100.0);
        assert_eq!(cost.io_cost, 50.0);
        assert_eq!(cost.memory_cost, 200.0);
        assert_eq!(cost.row_count, 1000.0);
    }

    #[test]
    fn test_cost_add() {
        let c1 = Cost::new(10.0, 20.0, 30.0, 100.0);
        let c2 = Cost::new(5.0, 10.0, 15.0, 50.0);
        let c3 = c1.add(&c2);
        assert_eq!(c3.cpu_cost, 15.0);
        assert_eq!(c3.io_cost, 30.0);
        assert_eq!(c3.memory_cost, 45.0);
        assert_eq!(c3.row_count, 150.0);
    }

    #[test]
    fn test_cost_multiply() {
        let c1 = Cost::new(10.0, 20.0, 30.0, 100.0);
        let c2 = c1.multiply(2.0);
        assert_eq!(c2.cpu_cost, 20.0);
        assert_eq!(c2.io_cost, 40.0);
        assert_eq!(c2.memory_cost, 60.0);
        assert_eq!(c2.row_count, 200.0);
    }

    #[test]
    fn test_cost_total() {
        let cost = Cost::new(100.0, 50.0, 25.0, 1000.0);
        let total = cost.total(1.0, 10.0, 1.0);
        // 100*1 + 50*10 + 25*1 = 100 + 500 + 25 = 625
        assert_eq!(total, 625.0);
    }

    #[test]
    fn test_table_stats_default() {
        let stats = TableStats::default();
        assert_eq!(stats.row_count, 0.0);
    }

    #[test]
    fn test_default_statistics_provider() {
        let provider = DefaultStatisticsProvider::new();
        let stats = provider.get_table_stats("users");
        assert!(stats.is_some());
    }

    #[test]
    fn test_default_cost_model_table_scan() {
        let provider = Box::new(DefaultStatisticsProvider::new());
        let cost_model = DefaultCostModel::new(provider);
        let plan = test_table_scan();
        let cost = cost_model.estimate_cost(&plan).unwrap();
        assert!(cost.row_count > 0.0);
        assert!(cost.io_cost > 0.0);
    }

    #[test]
    fn test_noop_cost_model() {
        let cost_model = NoOpCostModel;
        let plan = test_table_scan();
        let cost = cost_model.estimate_cost(&plan).unwrap();
        assert_eq!(cost.cpu_cost, 0.0);
        assert_eq!(cost.io_cost, 0.0);
    }

    // ============ Additional CostModel Tests ============

    #[test]
    fn test_cost_default() {
        let cost = Cost::default();
        assert_eq!(cost.cpu_cost, 0.0);
        assert_eq!(cost.io_cost, 0.0);
        assert_eq!(cost.memory_cost, 0.0);
    }

    #[test]
    fn test_cost_clone() {
        let cost1 = Cost::new(100.0, 50.0, 25.0, 1000.0);
        let cost2 = cost1.clone();
        assert_eq!(cost1.cpu_cost, cost2.cpu_cost);
    }

    #[test]
    fn test_cost_compare() {
        let cheap = Cost::new(10.0, 10.0, 10.0, 100.0);
        let expensive = Cost::new(100.0, 100.0, 100.0, 1000.0);
        assert!(cheap.total(1.0, 1.0, 1.0) < expensive.total(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_default_cost_model_filter() {
        let provider = Box::new(DefaultStatisticsProvider::new());
        let cost_model = DefaultCostModel::new(provider);
        let plan = LogicalPlan::Filter {
            input: Box::new(test_table_scan()),
            predicate: Expr::Literal(Value::Boolean(true)),
        };
        let cost = cost_model.estimate_cost(&plan).unwrap();
        assert!(cost.row_count > 0.0);
    }

    #[test]
    fn test_default_cost_model_projection() {
        let provider = Box::new(DefaultStatisticsProvider::new());
        let cost_model = DefaultCostModel::new(provider);
        let plan = LogicalPlan::Projection {
            input: Box::new(test_table_scan()),
            expr: vec![],
            schema: Schema::new(vec![]),
        };
        let cost = cost_model.estimate_cost(&plan).unwrap();
        assert!(cost.row_count >= 0.0);
    }

    #[test]
    fn test_default_cost_model_join() {
        let provider = Box::new(DefaultStatisticsProvider::new());
        let cost_model = DefaultCostModel::new(provider);
        let plan = LogicalPlan::Join {
            left: Box::new(test_table_scan()),
            right: Box::new(test_table_scan()),
            join_type: JoinType::Inner,
            on: vec![],
            filter: None,
            schema: Schema::new(vec![]),
        };
        let cost = cost_model.estimate_cost(&plan).unwrap();
        assert!(cost.cpu_cost > 0.0);
    }

    #[test]
    fn test_cost_model_with_statistics() {
        let provider = Box::new(DefaultStatisticsProvider::new());
        let cost_model = DefaultCostModel::new(provider);

        // Get table stats from provider
        let stats = provider.get_table_stats("users");
        assert!(stats.is_some());

        // Estimate cost with stats
        let plan = test_table_scan();
        let cost = cost_model.estimate_cost(&plan).unwrap();
        assert!(cost.row_count > 0.0);
    }

    #[test]
    fn test_cost_estimate_with_limit() {
        let provider = Box::new(DefaultStatisticsProvider::new());
        let cost_model = DefaultCostModel::new(provider);
        let plan = LogicalPlan::Limit {
            input: Box::new(test_table_scan()),
            n: 100,
        };
        let cost = cost_model.estimate_cost(&plan).unwrap();
        assert!(cost.row_count <= 100.0);
    }
}
