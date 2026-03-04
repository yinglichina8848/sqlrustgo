//! Cost Model Module

use super::*;

/// SimpleCostModel - estimates based on simple heuristics
pub struct SimpleCostModel;

impl<Plan> CostModel<Plan> for SimpleCostModel {
    fn estimate(&self, _plan: &Plan) -> f64 {
        // TODO: Implement proper cost estimation
        // For now, return a default cost
        1.0
    }
}
