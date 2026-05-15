use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalAction {
    Approve,
    Reject,
}

impl ApprovalAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalAction::Approve => "APPROVE",
            ApprovalAction::Reject => "REJECT",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub approval_id: String,
    pub instance_id: String,
    pub stage: String,
    pub approver_id: String,
    pub signature: String,
    pub action: ApprovalAction,
    pub comment: Option<String>,
    pub timestamp: u64,
}

impl ApprovalRecord {
    pub fn new(
        instance_id: String,
        stage: String,
        approver_id: String,
        signature: String,
        action: ApprovalAction,
        comment: Option<String>,
    ) -> Self {
        ApprovalRecord {
            approval_id: uuid_simple(),
            instance_id,
            stage,
            approver_id,
            signature,
            action,
            comment,
            timestamp: current_timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalChain {
    pub instance_id: String,
    pub required_signatures: u32,
    pub approvals: Vec<ApprovalRecord>,
}

impl ApprovalChain {
    pub fn new(instance_id: String, required_signatures: u32) -> Self {
        ApprovalChain {
            instance_id,
            required_signatures,
            approvals: Vec::new(),
        }
    }

    pub fn add_approval(&mut self, record: ApprovalRecord) -> Result<(), String> {
        if record.action == ApprovalAction::Reject {
            self.approvals.push(record);
            return Ok(());
        }

        let existing_approvals = self
            .approvals
            .iter()
            .filter(|a| a.stage == record.stage && a.action == ApprovalAction::Approve)
            .count();

        if existing_approvals >= self.required_signatures as usize {
            return Err("Required signatures already collected".to_string());
        }

        self.approvals.push(record);
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        let approvals_count = self
            .approvals
            .iter()
            .filter(|a| a.action == ApprovalAction::Approve)
            .count();

        approvals_count >= self.required_signatures as usize
    }

    pub fn get_approval_count(&self, stage: &str) -> usize {
        self.approvals
            .iter()
            .filter(|a| a.stage == stage && a.action == ApprovalAction::Approve)
            .count()
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn uuid_simple() -> String {
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("apr-{:x}-{:x}", dur.as_secs(), dur.subsec_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_chain_creation() {
        let chain = ApprovalChain::new("instance-1".to_string(), 2);
        assert_eq!(chain.required_signatures, 2);
        assert!(!chain.is_complete());
    }

    #[test]
    fn test_add_approval() {
        let mut chain = ApprovalChain::new("instance-1".to_string(), 2);

        let record = ApprovalRecord::new(
            "instance-1".to_string(),
            "review".to_string(),
            "user-1".to_string(),
            "signature-1".to_string(),
            ApprovalAction::Approve,
            None,
        );

        chain.add_approval(record).unwrap();
        assert!(!chain.is_complete());

        let record2 = ApprovalRecord::new(
            "instance-1".to_string(),
            "review".to_string(),
            "user-2".to_string(),
            "signature-2".to_string(),
            ApprovalAction::Approve,
            None,
        );

        chain.add_approval(record2).unwrap();
        assert!(chain.is_complete());
    }

    #[test]
    fn test_reject_terminates() {
        let mut chain = ApprovalChain::new("instance-1".to_string(), 2);

        let record = ApprovalRecord::new(
            "instance-1".to_string(),
            "review".to_string(),
            "user-1".to_string(),
            "signature-1".to_string(),
            ApprovalAction::Reject,
            Some("Not approved".to_string()),
        );

        chain.add_approval(record).unwrap();
        assert_eq!(chain.approvals.len(), 1);
    }
}
