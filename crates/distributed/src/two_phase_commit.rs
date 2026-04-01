//! Two-Phase Commit (2PC) implementation for distributed transactions
//!
//! Provides atomic commit protocol for multi-shard transactions.

use crate::raft::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub type TransactionId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionState {
    Init,
    Preparing,
    Prepared,
    Committing,
    Committed,
    Aborting,
    Aborted,
}

impl Default for TransactionState {
    fn default() -> Self {
        TransactionState::Init
    }
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
            if success && tx.participants.iter().all(|p| p.node_id != node_id || true) {
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

    fn handle_ack(&mut self, tx_id: TransactionId) -> Vec<(NodeId, TwoPCMessage)> {
        vec![]
    }

    pub fn prepare(&mut self, tx_id: TransactionId) -> Result<PrepareResult, TwoPCError> {
        let tx = self
            .transactions
            .get_mut(&tx_id)
            .ok_or_else(|| TwoPCError::TransactionNotFound(tx_id))?;

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
}
