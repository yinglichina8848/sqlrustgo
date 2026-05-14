use crate::{ClientGtid, SyncError, SyncResult, VectorClock};
use std::collections::HashSet;

pub struct OTEngine {
    dependency_tracker: HashSet<(String, u64)>,
}

impl OTEngine {
    pub fn new() -> Self {
        Self {
            dependency_tracker: HashSet::new(),
        }
    }

    pub fn check_dependencies(
        &self,
        cgtid: &ClientGtid,
        vector_clock: &VectorClock,
    ) -> SyncResult<()> {
        for (node_id, counter) in vector_clock.entries() {
            if *counter > 0 {
                let dep_key = (node_id.clone(), *counter);
                if !self.dependency_tracker.contains(&dep_key)
                    && dep_key != (cgtid.client_id.clone(), cgtid.txn_seq)
                {
                    return Err(SyncError::CausalityViolation(format!(
                        "{}:{} depends on uncommitted {}:{}",
                        cgtid.client_id, cgtid.txn_seq, node_id, counter
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn record_commit(&mut self, cgtid: &ClientGtid) {
        self.dependency_tracker
            .insert((cgtid.client_id.clone(), cgtid.txn_seq));
    }

    pub fn transform_operations(
        &self,
        local_ops: &[crate::Operation],
        remote_ops: &[crate::Operation],
        local_clock: &VectorClock,
        remote_clock: &VectorClock,
    ) -> SyncResult<Vec<crate::Operation>> {
        if local_clock.concurrent_with(remote_clock) {
            let transformed = self.resolve_conflicts(local_ops, remote_ops);
            Ok(transformed)
        } else if local_clock.happens_before(remote_clock) {
            Ok(local_ops.to_vec())
        } else {
            let transformed = self.resolve_conflicts(local_ops, remote_ops);
            Ok(transformed)
        }
    }

    fn resolve_conflicts(
        &self,
        local_ops: &[crate::Operation],
        remote_ops: &[crate::Operation],
    ) -> Vec<crate::Operation> {
        let mut result: Vec<crate::Operation> = Vec::new();

        for local_op in local_ops {
            let mut transformed_op: crate::Operation = local_op.clone();

            for remote_op in remote_ops {
                transformed_op = self.transform_pair(&transformed_op, remote_op);
            }

            result.push(transformed_op);
        }

        result
    }

    fn transform_pair(
        &self,
        local_op: &crate::Operation,
        remote_op: &crate::Operation,
    ) -> crate::Operation {
        use crate::OperationType::*;

        match (&local_op.op, &remote_op.op) {
            (Business(l), Business(r)) if l.entity_id == r.entity_id && l.op_type != r.op_type => {
                self.transform_business_conflict(l, r)
            }
            _ => local_op.clone(),
        }
    }

    fn transform_business_conflict(
        &self,
        local: &crate::BusinessOperation,
        remote: &crate::BusinessOperation,
    ) -> crate::Operation {
        use crate::OperationType::*;

        let priority_order = ["approve", "submit", "reject", "cancel"];

        let local_idx = priority_order.iter().position(|&x| x == local.op_type);
        let remote_idx = priority_order.iter().position(|&x| x == remote.op_type);

        let winner = match (local_idx, remote_idx) {
            (Some(li), Some(ri)) if li < ri => local.clone(),
            (Some(_li), None) => local.clone(),
            _ => remote.clone(),
        };

        crate::Operation {
            op: Business(winner),
        }
    }
}

impl Default for OTEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_clock(entries: Vec<(&str, u64)>) -> VectorClock {
        let mut clock = VectorClock::new();
        for (node, counter) in entries {
            clock = clock.with_entry(node, counter);
        }
        clock
    }

    #[test]
    fn test_concurrent_detection() {
        let clock1 = make_clock(vec![("node-a", 1), ("node-b", 2)]);
        let clock2 = make_clock(vec![("node-a", 2), ("node-b", 1)]);

        assert!(clock1.concurrent_with(&clock2));
    }

    #[test]
    fn test_happens_before() {
        let clock1 = make_clock(vec![("node-a", 1), ("node-b", 1)]);
        let clock2 = make_clock(vec![("node-a", 2), ("node-b", 1)]);

        assert!(clock1.happens_before(&clock2));
    }

    #[test]
    fn test_record_commit() {
        let mut engine = OTEngine::new();
        let cgtid = ClientGtid::new("iphone-23", 1);

        engine.record_commit(&cgtid);

        assert!(engine
            .dependency_tracker
            .contains(&("iphone-23".to_string(), 1)));
    }
}
