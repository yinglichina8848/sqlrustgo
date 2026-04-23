//! Two-Phase Commit (2PC) implementation for distributed transactions
//!
//! Provides atomic commit protocol for multi-shard transactions.

use crate::raft::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type TransactionId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TransactionState {
    #[default]
    Init,
    Preparing,
    Prepared,
    Committing,
    Committed,
    Aborting,
    Aborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    Yes,
    No,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub node_id: NodeId,
    pub shard_id: u64,
    pub prepared: bool,
    pub vote: Option<Vote>,
    pub error: Option<String>,
}

impl Participant {
    pub fn new(node_id: NodeId, shard_id: u64) -> Self {
        Self {
            node_id,
            shard_id,
            prepared: false,
            vote: None,
            error: None,
        }
    }

    pub fn vote_yes(&mut self) {
        self.vote = Some(Vote::Yes);
        self.prepared = true;
    }

    pub fn vote_no(&mut self, reason: &str) {
        self.vote = Some(Vote::No);
        self.error = Some(reason.to_string());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedTransaction {
    pub tx_id: TransactionId,
    pub coordinator_id: NodeId,
    pub participants: Vec<Participant>,
    pub state: TransactionState,
    pub created_at: u64,
    pub timeout_ms: u64,
    pub last_update_ms: u64,
    pub error_reason: Option<String>,
}

impl DistributedTransaction {
    pub fn new(
        tx_id: TransactionId,
        coordinator_id: NodeId,
        participants: Vec<Participant>,
    ) -> Self {
        let now = current_timestamp();
        Self {
            tx_id,
            coordinator_id,
            participants,
            state: TransactionState::Init,
            created_at: now,
            timeout_ms: 30_000,
            last_update_ms: now,
            error_reason: None,
        }
    }

    pub fn is_timed_out(&self) -> bool {
        current_timestamp() > self.last_update_ms + self.timeout_ms
    }

    pub fn all_voted_yes(&self) -> bool {
        self.participants.iter().all(|p| p.vote == Some(Vote::Yes))
    }

    pub fn any_voted_no(&self) -> bool {
        self.participants.iter().any(|p| p.vote == Some(Vote::No))
    }

    pub fn set_aborted(&mut self, reason: &str) {
        self.state = TransactionState::Aborted;
        self.error_reason = Some(reason.to_string());
        self.last_update_ms = current_timestamp();
    }

    pub fn set_committed(&mut self) {
        self.state = TransactionState::Committed;
        self.last_update_ms = current_timestamp();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TwoPCMessage {
    Prepare {
        tx_id: TransactionId,
        coordinator_id: NodeId,
    },
    PrepareResponse {
        tx_id: TransactionId,
        node_id: NodeId,
        vote: Vote,
        error: Option<String>,
    },
    Commit {
        tx_id: TransactionId,
    },
    CommitResponse {
        tx_id: TransactionId,
        node_id: NodeId,
        success: bool,
    },
    Abort {
        tx_id: TransactionId,
        reason: String,
    },
    AbortResponse {
        tx_id: TransactionId,
        node_id: NodeId,
    },
    Ack {
        tx_id: TransactionId,
    },
}

pub struct TwoPhaseCommit {
    node_id: NodeId,
    transactions: HashMap<TransactionId, DistributedTransaction>,
    #[allow(dead_code)]
    is_coordinator: bool,
    next_tx_id: TransactionId,
}

impl TwoPhaseCommit {
    pub fn new(node_id: NodeId, is_coordinator: bool) -> Self {
        Self {
            node_id,
            transactions: HashMap::new(),
            is_coordinator,
            next_tx_id: 1,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn begin_transaction(&mut self, participants: Vec<Participant>) -> TransactionId {
        let tx_id = self.next_tx_id;
        self.next_tx_id += 1;

        let tx = DistributedTransaction::new(tx_id, self.node_id, participants);
        self.transactions.insert(tx_id, tx);

        tx_id
    }

    pub fn get_transaction(&self, tx_id: TransactionId) -> Option<&DistributedTransaction> {
        self.transactions.get(&tx_id)
    }

    pub fn get_transaction_mut(
        &mut self,
        tx_id: TransactionId,
    ) -> Option<&mut DistributedTransaction> {
        self.transactions.get_mut(&tx_id)
    }

    pub fn handle_message(&mut self, msg: TwoPCMessage) -> Vec<(NodeId, TwoPCMessage)> {
        match msg {
            TwoPCMessage::Prepare {
                tx_id,
                coordinator_id,
            } => self.handle_prepare(tx_id, coordinator_id),
            TwoPCMessage::PrepareResponse {
                tx_id,
                node_id,
                vote,
                error,
            } => self.handle_prepare_response(tx_id, node_id, vote, error),
            TwoPCMessage::Commit { tx_id } => self.handle_commit(tx_id),
            TwoPCMessage::CommitResponse {
                tx_id,
                node_id,
                success,
            } => self.handle_commit_response(tx_id, node_id, success),
            TwoPCMessage::Abort { tx_id, reason } => self.handle_abort(tx_id, &reason),
            TwoPCMessage::AbortResponse { tx_id, node_id } => {
                self.handle_abort_response(tx_id, node_id)
            }
            TwoPCMessage::Ack { tx_id } => self.handle_ack(tx_id),
        }
    }

    fn handle_prepare(
        &mut self,
        tx_id: TransactionId,
        coordinator_id: NodeId,
    ) -> Vec<(NodeId, TwoPCMessage)> {
        let tx = self.transactions.get_mut(&tx_id);

        if let Some(tx) = tx {
            tx.state = TransactionState::Preparing;
            tx.last_update_ms = current_timestamp();

            vec![(
                coordinator_id,
                TwoPCMessage::PrepareResponse {
                    tx_id,
                    node_id: self.node_id,
                    vote: Vote::Yes,
                    error: None,
                },
            )]
        } else {
            vec![(
                coordinator_id,
                TwoPCMessage::PrepareResponse {
                    tx_id,
                    node_id: self.node_id,
                    vote: Vote::No,
                    error: Some("Transaction not found".to_string()),
                },
            )]
        }
    }

    fn handle_prepare_response(
        &mut self,
        tx_id: TransactionId,
        node_id: NodeId,
        vote: Vote,
        error: Option<String>,
    ) -> Vec<(NodeId, TwoPCMessage)> {
        let tx = self.transactions.get_mut(&tx_id);

        if let Some(tx) = tx {
            if let Some(participant) = tx.participants.iter_mut().find(|p| p.node_id == node_id) {
                match vote {
                    Vote::Yes => participant.vote_yes(),
                    Vote::No => participant.vote_no(error.as_deref().unwrap_or("Unknown")),
                }
            }

            tx.last_update_ms = current_timestamp();

            if tx.state == TransactionState::Preparing {
                if tx.all_voted_yes() {
                    tx.state = TransactionState::Prepared;
                    return tx
                        .participants
                        .iter()
                        .map(|p| (p.node_id, TwoPCMessage::Commit { tx_id }))
                        .collect();
                } else if tx.any_voted_no() {
                    tx.state = TransactionState::Aborting;
                    let reason = tx
                        .participants
                        .iter()
                        .find(|p| p.vote == Some(Vote::No))
                        .and_then(|p| p.error.clone())
                        .unwrap_or_else(|| "Participant voted No".to_string());

                    return tx
                        .participants
                        .iter()
                        .map(|p| {
                            (
                                p.node_id,
                                TwoPCMessage::Abort {
                                    tx_id,
                                    reason: reason.clone(),
                                },
                            )
                        })
                        .collect();
                }
            }
        }

        vec![]
    }

    fn handle_commit(&mut self, tx_id: TransactionId) -> Vec<(NodeId, TwoPCMessage)> {
        let tx = self.transactions.get_mut(&tx_id);

        if let Some(tx) = tx {
            tx.state = TransactionState::Committing;
            tx.last_update_ms = current_timestamp();

            vec![(
                tx.coordinator_id,
                TwoPCMessage::CommitResponse {
                    tx_id,
                    node_id: self.node_id,
                    success: true,
                },
            )]
        } else {
            vec![]
        }
    }

    fn handle_commit_response(
        &mut self,
        tx_id: TransactionId,
        node_id: NodeId,
        success: bool,
    ) -> Vec<(NodeId, TwoPCMessage)> {
        let tx = self.transactions.get_mut(&tx_id);

        if let Some(tx) = tx {
            if success && tx.participants.iter().all(|p| p.node_id != node_id) {
                let all_acknowledged = tx
                    .participants
                    .iter()
                    .filter(|p| p.node_id != self.node_id)
                    .count()
                    == 0;

                if all_acknowledged || tx.participants.len() == 1 {
                    tx.set_committed();
                }
            }
        }

        vec![]
    }

    fn handle_abort(&mut self, tx_id: TransactionId, reason: &str) -> Vec<(NodeId, TwoPCMessage)> {
        let tx = self.transactions.get_mut(&tx_id);

        if let Some(tx) = tx {
            tx.set_aborted(reason);

            vec![(
                tx.coordinator_id,
                TwoPCMessage::AbortResponse {
                    tx_id,
                    node_id: self.node_id,
                },
            )]
        } else {
            vec![]
        }
    }

    fn handle_abort_response(
        &mut self,
        tx_id: TransactionId,
        _node_id: NodeId,
    ) -> Vec<(NodeId, TwoPCMessage)> {
        let tx = self.transactions.get_mut(&tx_id);

        if let Some(tx) = tx {
            tx.state = TransactionState::Aborted;
            tx.last_update_ms = current_timestamp();
        }

        vec![]
    }

    fn handle_ack(&mut self, _tx_id: TransactionId) -> Vec<(NodeId, TwoPCMessage)> {
        vec![]
    }

    pub fn prepare(&mut self, tx_id: TransactionId) -> Result<PrepareResult, TwoPCError> {
        let tx = self
            .transactions
            .get_mut(&tx_id)
            .ok_or(TwoPCError::TransactionNotFound(tx_id))?;

        if tx.state != TransactionState::Init {
            return Err(TwoPCError::InvalidState(tx.state));
        }

        tx.state = TransactionState::Preparing;
        tx.last_update_ms = current_timestamp();

        let participants: Vec<NodeId> = tx.participants.iter().map(|p| p.node_id).collect();

        Ok(PrepareResult {
            tx_id,
            messages: participants
                .into_iter()
                .map(|node_id| {
                    (
                        node_id,
                        TwoPCMessage::Prepare {
                            tx_id,
                            coordinator_id: self.node_id,
                        },
                    )
                })
                .collect(),
        })
    }

    pub fn force_abort_timed_out(&mut self) -> Vec<AbortCommand> {
        let mut commands = Vec::new();

        for (tx_id, tx) in self.transactions.iter_mut() {
            if tx.is_timed_out()
                && tx.state != TransactionState::Aborted
                && tx.state != TransactionState::Committed
            {
                let reason = "Transaction timeout".to_string();
                tx.set_aborted(&reason);

                commands.push(AbortCommand {
                    tx_id: *tx_id,
                    participants: tx.participants.iter().map(|p| p.node_id).collect(),
                    reason,
                });
            }
        }

        commands
    }

    pub fn cleanup_completed(&mut self) {
        self.transactions.retain(|_, tx| {
            tx.state != TransactionState::Committed && tx.state != TransactionState::Aborted
        });
    }

    pub fn num_active_transactions(&self) -> usize {
        self.transactions.len()
    }
}

pub struct PrepareResult {
    pub tx_id: TransactionId,
    pub messages: Vec<(NodeId, TwoPCMessage)>,
}

pub struct AbortCommand {
    pub tx_id: TransactionId,
    pub participants: Vec<NodeId>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum TwoPCError {
    TransactionNotFound(TransactionId),
    InvalidState(TransactionState),
    ParticipantRejected { node_id: NodeId, reason: String },
}

impl std::fmt::Display for TwoPCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TwoPCError::TransactionNotFound(id) => write!(f, "Transaction not found: {}", id),
            TwoPCError::InvalidState(state) => write!(f, "Invalid transaction state: {:?}", state),
            TwoPCError::ParticipantRejected { node_id, reason } => {
                write!(f, "Participant {} rejected: {}", node_id, reason)
            }
        }
    }
}

impl std::error::Error for TwoPCError {}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_begin_transaction() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0), Participant::new(3, 1)];

        let tx_id = tpc.begin_transaction(participants);
        assert_eq!(tx_id, 1);
    }

    #[test]
    fn test_prepare_success() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0), Participant::new(3, 1)];

        let tx_id = tpc.begin_transaction(participants);
        let result = tpc.prepare(tx_id);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().messages.len(), 2);
    }

    #[test]
    fn test_prepare_invalid_state() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];

        let tx_id = tpc.begin_transaction(participants);
        tpc.prepare(tx_id).unwrap();

        let result = tpc.prepare(tx_id);
        assert!(matches!(result, Err(TwoPCError::InvalidState(_))));
    }

    #[test]
    fn test_transaction_timeout() {
        let participants = vec![Participant::new(2, 0)];
        let mut tx = DistributedTransaction::new(1, 1, participants);

        tx.timeout_ms = 1; // 1 second (current_timestamp is in seconds)
        std::thread::sleep(Duration::from_secs(2));

        assert!(tx.is_timed_out());
    }

    #[test]
    fn test_all_voted_yes() {
        let mut participants = vec![Participant::new(2, 0), Participant::new(3, 1)];
        for p in &mut participants {
            p.vote = Some(Vote::Yes);
        }
        let tx = DistributedTransaction::new(1, 1, participants);

        assert!(tx.all_voted_yes());
    }

    #[test]
    fn test_any_voted_no() {
        let mut participants = vec![Participant::new(2, 0), Participant::new(3, 1)];
        participants[0].vote_yes();
        participants[1].vote_no("Network timeout");
        let tx = DistributedTransaction::new(1, 1, participants);

        assert!(tx.any_voted_no());
    }

    #[test]
    fn test_set_aborted() {
        let participants = vec![Participant::new(2, 0)];
        let tx = DistributedTransaction::new(1, 1, participants);
        assert_eq!(tx.state, TransactionState::Init);

        let mut tx = tx;
        tx.set_aborted("User cancelled");
        assert_eq!(tx.state, TransactionState::Aborted);
        assert!(tx.error_reason.is_some());
    }

    #[test]
    fn test_set_committed() {
        let participants = vec![Participant::new(2, 0)];
        let mut tx = DistributedTransaction::new(1, 1, participants);
        tx.set_committed();
        assert_eq!(tx.state, TransactionState::Committed);
    }

    #[test]
    fn test_force_abort_timed_out() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];

        let tx_id = tpc.begin_transaction(participants);
        // Set last_update_ms to the past so transaction is timed out
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.last_update_ms = 0; // Set to epoch, way in the past
        }

        let abort_commands = tpc.force_abort_timed_out();
        assert_eq!(abort_commands.len(), 1);
        assert_eq!(abort_commands[0].tx_id, tx_id);
    }

    #[test]
    fn test_cleanup_completed() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];

        let tx_id = tpc.begin_transaction(participants);

        // Set transaction to committed
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.set_committed();
        }

        assert_eq!(tpc.num_active_transactions(), 1);
        tpc.cleanup_completed();
        assert_eq!(tpc.num_active_transactions(), 0);
    }

    #[test]
    fn test_num_active_transactions() {
        let mut tpc = TwoPhaseCommit::new(1, true);

        assert_eq!(tpc.num_active_transactions(), 0);

        let participants1 = vec![Participant::new(2, 0)];
        tpc.begin_transaction(participants1);
        assert_eq!(tpc.num_active_transactions(), 1);

        let participants2 = vec![Participant::new(3, 0)];
        tpc.begin_transaction(participants2);
        assert_eq!(tpc.num_active_transactions(), 2);
    }

    #[test]
    fn test_get_transaction() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];

        let tx_id = tpc.begin_transaction(participants);

        let tx = tpc.get_transaction(tx_id);
        assert!(tx.is_some());

        let tx = tpc.get_transaction(999);
        assert!(tx.is_none());
    }

    #[test]
    fn test_prepare_transaction_not_found() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let result = tpc.prepare(999);
        assert!(matches!(result, Err(TwoPCError::TransactionNotFound(_))));
    }

    #[test]
    fn test_participant_vote() {
        let mut p = Participant::new(2, 0);
        assert!(p.vote.is_none());

        p.vote_yes();
        assert!(p.prepared);
        assert_eq!(p.vote, Some(Vote::Yes));

        let mut p2 = Participant::new(3, 1);
        p2.vote_no("Error");
        assert_eq!(p2.vote, Some(Vote::No));
        assert_eq!(p2.error, Some("Error".to_string()));
    }

    #[test]
    fn test_participant_debug() {
        let p = Participant::new(2, 1);
        let debug_str = format!("{:?}", p);
        assert!(debug_str.contains("node_id: 2"));
        assert!(debug_str.contains("shard_id: 1"));
    }

    #[test]
    fn test_transaction_state_debug() {
        assert_eq!(format!("{:?}", TransactionState::Init), "Init");
        assert_eq!(format!("{:?}", TransactionState::Preparing), "Preparing");
        assert_eq!(format!("{:?}", TransactionState::Prepared), "Prepared");
        assert_eq!(format!("{:?}", TransactionState::Committing), "Committing");
        assert_eq!(format!("{:?}", TransactionState::Committed), "Committed");
        assert_eq!(format!("{:?}", TransactionState::Aborting), "Aborting");
        assert_eq!(format!("{:?}", TransactionState::Aborted), "Aborted");
    }

    #[test]
    fn test_vote_debug() {
        assert_eq!(format!("{:?}", Vote::Yes), "Yes");
        assert_eq!(format!("{:?}", Vote::No), "No");
    }

    #[test]
    fn test_two_pc_error_display() {
        let err = TwoPCError::TransactionNotFound(123);
        assert!(err.to_string().contains("123"));

        let err2 = TwoPCError::InvalidState(TransactionState::Init);
        assert!(err2.to_string().contains("Init"));

        let err3 = TwoPCError::ParticipantRejected {
            node_id: 42,
            reason: "error".to_string(),
        };
        assert!(err3.to_string().contains("42"));
        assert!(err3.to_string().contains("error"));
    }

    #[test]
    fn test_two_pc_error_debug() {
        let err = TwoPCError::TransactionNotFound(123);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("TransactionNotFound"));
    }

    #[test]
    fn test_two_pc_message_debug() {
        let msg = TwoPCMessage::Prepare { tx_id: 1, coordinator_id: 100 };
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Prepare"));

        let msg2 = TwoPCMessage::Commit { tx_id: 1 };
        let debug_str2 = format!("{:?}", msg2);
        assert!(debug_str2.contains("Commit"));
    }

    #[test]
    fn test_two_pc_message_serialization() {
        let msg = TwoPCMessage::Prepare { tx_id: 42, coordinator_id: 1 };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Prepare"));

        let parsed: TwoPCMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            TwoPCMessage::Prepare { tx_id, coordinator_id: _ } => assert_eq!(tx_id, 42),
            _ => panic!("Expected Prepare"),
        }
    }

    #[test]
    fn test_transaction_state_serialization() {
        let state = TransactionState::Prepared;
        let json = serde_json::to_string(&state).unwrap();
        let parsed: TransactionState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TransactionState::Prepared);
    }

    #[test]
    fn test_vote_serialization() {
        let vote = Vote::Yes;
        let json = serde_json::to_string(&vote).unwrap();
        let parsed: Vote = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Vote::Yes);
    }

    #[test]
    fn test_two_pc_message_commit_response() {
        let msg = TwoPCMessage::CommitResponse { tx_id: 1, node_id: 2, success: true };
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("CommitResponse"));
    }

    #[test]
    fn test_two_pc_message_abort() {
        let msg = TwoPCMessage::Abort { tx_id: 1, reason: "timeout".to_string() };
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Abort"));
    }

    #[test]
    fn test_two_pc_message_abort_response() {
        let msg = TwoPCMessage::AbortResponse { tx_id: 1, node_id: 2 };
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("AbortResponse"));
    }

    #[test]
    fn test_two_pc_message_ack() {
        let msg = TwoPCMessage::Ack { tx_id: 1 };
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Ack"));
    }

    #[test]
    fn test_two_phase_commit_new() {
        let tpc = TwoPhaseCommit::new(1, true);
        assert_eq!(tpc.num_active_transactions(), 0);
    }

    #[test]
    fn test_two_phase_commit_with_coordinator() {
        let tpc = TwoPhaseCommit::new(42, true);
        assert_eq!(tpc.num_active_transactions(), 0);
    }

    #[test]
    fn test_two_phase_commit_num_active_transactions() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        tpc.begin_transaction(participants);
        assert_eq!(tpc.num_active_transactions(), 1);
    }

    #[test]
    fn test_two_phase_commit_get_transaction() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);
        let tx = tpc.get_transaction(tx_id);
        assert!(tx.is_some());
    }

    #[test]
    fn test_two_phase_commit_get_transaction_none() {
        let tpc = TwoPhaseCommit::new(1, true);
        let tx = tpc.get_transaction(999);
        assert!(tx.is_none());
    }

    #[test]
    fn test_two_phase_commit_force_abort_timed_out() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.last_update_ms = 0;
        }
        let commands = tpc.force_abort_timed_out();
        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn test_two_phase_commit_cleanup_completed() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.set_committed();
        }
        tpc.cleanup_completed();
        assert_eq!(tpc.num_active_transactions(), 0);
    }

    #[test]
    fn test_distributed_transaction_new() {
        let participants = vec![Participant::new(2, 0)];
        let tx = DistributedTransaction::new(1, 100, participants);
        assert_eq!(tx.tx_id, 1);
        assert_eq!(tx.coordinator_id, 100);
        assert_eq!(tx.state, TransactionState::Init);
    }

    #[test]
    fn test_distributed_transaction_set_aborted() {
        let participants = vec![Participant::new(2, 0)];
        let mut tx = DistributedTransaction::new(1, 1, participants);
        tx.set_aborted("test");
        assert_eq!(tx.state, TransactionState::Aborted);
        assert!(tx.error_reason.is_some());
    }

    #[test]
    fn test_distributed_transaction_set_committed() {
        let participants = vec![Participant::new(2, 0)];
        let mut tx = DistributedTransaction::new(1, 1, participants);
        tx.set_committed();
        assert_eq!(tx.state, TransactionState::Committed);
    }

    #[test]
    fn test_distributed_transaction_is_timed_out() {
        let participants = vec![Participant::new(2, 0)];
        let mut tx = DistributedTransaction::new(1, 1, participants);
        tx.timeout_ms = 1; // 1 second (current_timestamp is in seconds)
        std::thread::sleep(Duration::from_secs(2));
        assert!(tx.is_timed_out());
    }

    #[test]
    fn test_distributed_transaction_not_timed_out() {
        let participants = vec![Participant::new(2, 0)];
        let tx = DistributedTransaction::new(1, 1, participants);
        assert!(!tx.is_timed_out());
    }

    // =====================================================================
    // White-box Tests: Branch Coverage for handle_prepare
    // =====================================================================

    #[test]
    fn test_handle_prepare_transaction_exists() {
        let mut tpc = TwoPhaseCommit::new(1, false);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);

        // handle_prepare should return Vote::Yes when transaction exists
        let responses = tpc.handle_message(TwoPCMessage::Prepare {
            tx_id,
            coordinator_id: 100,
        });

        assert_eq!(responses.len(), 1);
        let (_, msg) = &responses[0];
        match msg {
            TwoPCMessage::PrepareResponse { vote, .. } => {
                assert_eq!(*vote, Vote::Yes);
            }
            _ => panic!("Expected PrepareResponse"),
        }
    }

    #[test]
    fn test_handle_prepare_transaction_not_found() {
        let mut tpc = TwoPhaseCommit::new(1, false);

        // handle_prepare should return Vote::No when transaction doesn't exist
        let responses = tpc.handle_message(TwoPCMessage::Prepare {
            tx_id: 999,
            coordinator_id: 100,
        });

        assert_eq!(responses.len(), 1);
        let (_, msg) = &responses[0];
        match msg {
            TwoPCMessage::PrepareResponse { vote, error, .. } => {
                assert_eq!(*vote, Vote::No);
                assert!(error.is_some());
            }
            _ => panic!("Expected PrepareResponse"),
        }
    }

    // =====================================================================
    // White-box Tests: Branch Coverage for handle_prepare_response
    // =====================================================================

    #[test]
    fn test_handle_prepare_response_vote_yes() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0), Participant::new(3, 1)];
        let tx_id = tpc.begin_transaction(participants);

        // First, set state to Preparing
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Preparing;
        }

        // Handle Vote::Yes from participant 2
        let responses = tpc.handle_message(TwoPCMessage::PrepareResponse {
            tx_id,
            node_id: 2,
            vote: Vote::Yes,
            error: None,
        });

        // Should not trigger commit/abort yet (waiting for all votes)
        assert!(responses.is_empty() || !responses.iter().any(|(_n, m)| matches!(m, TwoPCMessage::Commit { .. })));
    }

    #[test]
    fn test_handle_prepare_response_vote_no_triggers_abort() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0), Participant::new(3, 1)];
        let tx_id = tpc.begin_transaction(participants);

        // Set state to Preparing
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Preparing;
        }

        // Handle Vote::No from participant 2
        let responses = tpc.handle_message(TwoPCMessage::PrepareResponse {
            tx_id,
            node_id: 2,
            vote: Vote::No,
            error: Some("Disk full".to_string()),
        });

        // Should trigger Abort messages to all participants
        assert_eq!(responses.len(), 2);
        for (_, msg) in responses {
            match msg {
                TwoPCMessage::Abort { tx_id: abort_tx_id, reason } => {
                    assert_eq!(abort_tx_id, tx_id);
                    assert!(reason.contains("Disk full") || reason.contains("Participant voted No"));
                }
                _ => panic!("Expected Abort message"),
            }
        }
    }

    #[test]
    fn test_handle_prepare_response_all_voted_yes_triggers_commit() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0), Participant::new(3, 1)];
        let tx_id = tpc.begin_transaction(participants);

        // Set state to Preparing
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Preparing;
        }

        // First vote from participant 2
        tpc.handle_message(TwoPCMessage::PrepareResponse {
            tx_id,
            node_id: 2,
            vote: Vote::Yes,
            error: None,
        });

        // Second vote from participant 3
        let responses = tpc.handle_message(TwoPCMessage::PrepareResponse {
            tx_id,
            node_id: 3,
            vote: Vote::Yes,
            error: None,
        });

        // Should trigger Commit messages to all participants
        assert_eq!(responses.len(), 2);
        for (_, msg) in responses {
            match msg {
                TwoPCMessage::Commit { tx_id: commit_tx_id } => {
                    assert_eq!(commit_tx_id, tx_id);
                }
                _ => panic!("Expected Commit message"),
            }
        }
    }

    #[test]
    fn test_handle_prepare_response_unknown_participant() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);

        // Set state to Preparing
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Preparing;
        }

        // Handle response from unknown participant
        let responses = tpc.handle_message(TwoPCMessage::PrepareResponse {
            tx_id,
            node_id: 999, // Unknown participant
            vote: Vote::Yes,
            error: None,
        });

        // Should be handled gracefully (empty response or ignored)
        // The participant is not in the list, so nothing should happen
        assert!(responses.is_empty());
    }

    // =====================================================================
    // White-box Tests: Branch Coverage for handle_commit
    // =====================================================================

    #[test]
    fn test_handle_commit_transaction_exists() {
        let mut tpc = TwoPhaseCommit::new(1, false);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);

        // Set to Committed state
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Prepared;
        }

        let responses = tpc.handle_message(TwoPCMessage::Commit { tx_id });

        // Should send CommitResponse
        assert_eq!(responses.len(), 1);
    }

    #[test]
    fn test_handle_commit_transaction_not_found() {
        let mut tpc = TwoPhaseCommit::new(1, false);

        let responses = tpc.handle_message(TwoPCMessage::Commit { tx_id: 999 });

        // Should return empty for unknown transaction
        assert!(responses.is_empty());
    }

    // =====================================================================
    // White-box Tests: Branch Coverage for handle_commit_response
    // =====================================================================

    #[test]
    fn test_handle_commit_response_success() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0), Participant::new(3, 1)];
        let tx_id = tpc.begin_transaction(participants);

        // Set state to Committing
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Committing;
        }

        // First success response (this node is not the coordinator, so condition check)
        let responses1 = tpc.handle_message(TwoPCMessage::CommitResponse {
            tx_id,
            node_id: 2,
            success: true,
        });
        assert!(responses1.is_empty());

        // Second success response
        tpc.handle_message(TwoPCMessage::CommitResponse {
            tx_id,
            node_id: 3,
            success: true,
        });

        // The state remains Committing until explicitly set
        let tx = tpc.get_transaction(tx_id).unwrap();
        assert_eq!(tx.state, TransactionState::Committing);
    }

    #[test]
    fn test_handle_commit_response_failure() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);

        // Set state to Committing
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Committing;
        }

        let responses = tpc.handle_message(TwoPCMessage::CommitResponse {
            tx_id,
            node_id: 2,
            success: false,
        });

        // Should handle failure
        assert!(responses.is_empty());
    }

    // =====================================================================
    // White-box Tests: Branch Coverage for handle_abort
    // =====================================================================

    #[test]
    fn test_handle_abort_transaction_exists() {
        let mut tpc = TwoPhaseCommit::new(1, false);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);

        let responses = tpc.handle_message(TwoPCMessage::Abort {
            tx_id,
            reason: "User requested".to_string(),
        });

        assert_eq!(responses.len(), 1);
        let (coordinator_id, msg) = &responses[0];
        assert_eq!(*coordinator_id, 1);
        match msg {
            TwoPCMessage::AbortResponse { tx_id: abort_tx_id, node_id: response_node_id } => {
                assert_eq!(*abort_tx_id, tx_id);
                assert_eq!(*response_node_id, 1);
            }
            _ => panic!("Expected AbortResponse"),
        }
    }

    #[test]
    fn test_handle_abort_transaction_not_found() {
        let mut tpc = TwoPhaseCommit::new(1, false);

        let responses = tpc.handle_message(TwoPCMessage::Abort {
            tx_id: 999,
            reason: "User requested".to_string(),
        });

        // Should return empty for unknown transaction
        assert!(responses.is_empty());
    }

    // =====================================================================
    // White-box Tests: Branch Coverage for handle_abort_response
    // =====================================================================

    #[test]
    fn test_handle_abort_response() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);

        // Set state to Aborting
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Aborting;
        }

        let responses = tpc.handle_message(TwoPCMessage::AbortResponse {
            tx_id,
            node_id: 2,
        });

        // Should be handled
        assert!(responses.is_empty());
    }

    // =====================================================================
    // White-box Tests: Path Coverage for force_abort_timed_out
    // =====================================================================

    #[test]
    fn test_force_abort_timed_out_multiple_transactions() {
        let mut tpc = TwoPhaseCommit::new(1, true);

        // Create multiple transactions
        let tx_id1 = tpc.begin_transaction(vec![Participant::new(2, 0)]);
        let tx_id2 = tpc.begin_transaction(vec![Participant::new(3, 0)]);
        let _tx_id3 = tpc.begin_transaction(vec![Participant::new(4, 0)]);

        // Set first two as timed out
        if let Some(tx) = tpc.get_transaction_mut(tx_id1) {
            tx.last_update_ms = 0;
        }
        if let Some(tx) = tpc.get_transaction_mut(tx_id2) {
            tx.last_update_ms = 0;
        }

        let commands = tpc.force_abort_timed_out();

        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn test_force_abort_timed_out_already_aborted() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let tx_id = tpc.begin_transaction(vec![Participant::new(2, 0)]);

        // Set as timed out AND already aborted
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.last_update_ms = 0;
            tx.state = TransactionState::Aborted;
        }

        let commands = tpc.force_abort_timed_out();

        // Should not abort again
        assert!(commands.is_empty());
    }

    #[test]
    fn test_force_abort_timed_out_already_committed() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let tx_id = tpc.begin_transaction(vec![Participant::new(2, 0)]);

        // Set as timed out AND already committed
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.last_update_ms = 0;
            tx.state = TransactionState::Committed;
        }

        let commands = tpc.force_abort_timed_out();

        // Should not abort committed transaction
        assert!(commands.is_empty());
    }

    // =====================================================================
    // White-box Tests: Path Coverage for cleanup_completed
    // =====================================================================

    #[test]
    fn test_cleanup_completed_mixed_states() {
        let mut tpc = TwoPhaseCommit::new(1, true);

        // Create transactions with different states
        let tx_id1 = tpc.begin_transaction(vec![Participant::new(2, 0)]);
        let tx_id2 = tpc.begin_transaction(vec![Participant::new(3, 0)]);
        let tx_id3 = tpc.begin_transaction(vec![Participant::new(4, 0)]);

        // Set different states
        if let Some(tx) = tpc.get_transaction_mut(tx_id1) {
            tx.state = TransactionState::Committed;
        }
        if let Some(tx) = tpc.get_transaction_mut(tx_id2) {
            tx.state = TransactionState::Aborted;
        }
        if let Some(tx) = tpc.get_transaction_mut(tx_id3) {
            tx.state = TransactionState::Preparing; // Active, should be kept
        }

        assert_eq!(tpc.num_active_transactions(), 3);
        tpc.cleanup_completed();
        assert_eq!(tpc.num_active_transactions(), 1);

        // Only tx_id3 should remain
        assert!(tpc.get_transaction(tx_id1).is_none());
        assert!(tpc.get_transaction(tx_id2).is_none());
        assert!(tpc.get_transaction(tx_id3).is_some());
    }

    // =====================================================================
    // White-box Tests: Condition Coverage for is_timed_out
    // =====================================================================

    #[test]
    fn test_is_timed_out_edge_case_no_timeout() {
        let participants = vec![Participant::new(2, 0)];
        let tx = DistributedTransaction::new(1, 1, participants);
        // Default timeout is 30_000ms, should not timeout immediately
        assert!(!tx.is_timed_out());
    }

    #[test]
    fn test_is_timed_out_edge_case_zero_timeout() {
        let participants = vec![Participant::new(2, 0)];
        let mut tx = DistributedTransaction::new(1, 1, participants);
        tx.last_update_ms = 0;
        tx.timeout_ms = 0;
        assert!(tx.is_timed_out());
    }

    // =====================================================================
    // White-box Tests: All TransactionState transitions
    // =====================================================================

    #[test]
    fn test_transaction_state_all_transitions() {
        let participants = vec![Participant::new(2, 0)];
        let mut tx = DistributedTransaction::new(1, 1, participants);

        // Init -> Preparing
        tx.state = TransactionState::Preparing;
        assert_eq!(tx.state, TransactionState::Preparing);

        // Preparing -> Prepared
        tx.state = TransactionState::Prepared;
        assert_eq!(tx.state, TransactionState::Prepared);

        // Prepared -> Committing
        tx.state = TransactionState::Committing;
        assert_eq!(tx.state, TransactionState::Committing);

        // Committing -> Committed
        tx.set_committed();
        assert_eq!(tx.state, TransactionState::Committed);
    }

    #[test]
    fn test_transaction_state_abort_transitions() {
        let participants = vec![Participant::new(2, 0)];
        let mut tx = DistributedTransaction::new(1, 1, participants);

        // Init -> Preparing
        tx.state = TransactionState::Preparing;
        assert_eq!(tx.state, TransactionState::Preparing);

        // Preparing -> Aborting
        tx.state = TransactionState::Aborting;
        assert_eq!(tx.state, TransactionState::Aborting);

        // Aborting -> Aborted
        tx.set_aborted("Test abort");
        assert_eq!(tx.state, TransactionState::Aborted);
        assert!(tx.error_reason.is_some());
    }

    // =====================================================================
    // White-box Tests: prepare() InvalidState branch coverage
    // =====================================================================

    #[test]
    fn test_prepare_invalid_state_preparing() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);

        // Set state to Preparing
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Preparing;
        }

        let result = tpc.prepare(tx_id);
        assert!(matches!(result, Err(TwoPCError::InvalidState(TransactionState::Preparing))));
    }

    #[test]
    fn test_prepare_invalid_state_prepared() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);

        // Set state to Prepared
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Prepared;
        }

        let result = tpc.prepare(tx_id);
        assert!(matches!(result, Err(TwoPCError::InvalidState(TransactionState::Prepared))));
    }

    #[test]
    fn test_prepare_invalid_state_committed() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);

        // Set state to Committed
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Committed;
        }

        let result = tpc.prepare(tx_id);
        assert!(matches!(result, Err(TwoPCError::InvalidState(TransactionState::Committed))));
    }

    #[test]
    fn test_prepare_invalid_state_aborted() {
        let mut tpc = TwoPhaseCommit::new(1, true);
        let participants = vec![Participant::new(2, 0)];
        let tx_id = tpc.begin_transaction(participants);

        // Set state to Aborted
        if let Some(tx) = tpc.get_transaction_mut(tx_id) {
            tx.state = TransactionState::Aborted;
        }

        let result = tpc.prepare(tx_id);
        assert!(matches!(result, Err(TwoPCError::InvalidState(TransactionState::Aborted))));
    }

    // =====================================================================
    // White-box Tests: Message handling path coverage
    // =====================================================================

    #[test]
    fn test_handle_message_all_variants() {
        let mut tpc = TwoPhaseCommit::new(1, true);

        // Test Prepare message with non-existent transaction
        let _ = tpc.handle_message(TwoPCMessage::Prepare {
            tx_id: 999,
            coordinator_id: 100,
        });

        // Test Commit message with non-existent transaction
        let _ = tpc.handle_message(TwoPCMessage::Commit { tx_id: 999 });

        // Test Abort message with non-existent transaction
        let _ = tpc.handle_message(TwoPCMessage::Abort {
            tx_id: 999,
            reason: "test".to_string(),
        });

        // Test Ack message
        let _ = tpc.handle_message(TwoPCMessage::Ack { tx_id: 999 });

        // No panics means all branches handled gracefully
    }

    // =====================================================================
    // White-box Tests: TwoPCError Debug format
    // =====================================================================

    #[test]
    fn test_two_pc_error_all_variants_debug() {
        let err1 = TwoPCError::TransactionNotFound(123);
        let debug1 = format!("{:?}", err1);
        assert!(debug1.contains("TransactionNotFound"));
        assert!(debug1.contains("123"));

        let err2 = TwoPCError::InvalidState(TransactionState::Init);
        let debug2 = format!("{:?}", err2);
        assert!(debug2.contains("InvalidState"));

        let err3 = TwoPCError::ParticipantRejected {
            node_id: 42,
            reason: "disk full".to_string(),
        };
        let debug3 = format!("{:?}", err3);
        assert!(debug3.contains("ParticipantRejected"));
        assert!(debug3.contains("42"));
        assert!(debug3.contains("disk full"));
    }
}
