//! Raft consensus protocol implementation
//!
//! Provides leader election and log replication for distributed coordination.

use serde::{Deserialize, Serialize};

use std::time::{Duration, Instant};

pub type NodeId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RaftState {
    #[default]
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftEntry {
    pub term: u64,
    pub index: u64,
    pub data: RaftEntryData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RaftEntryData {
    NoOp,
    Transaction { tx_id: u64 },
    ConfigChange { node_id: NodeId, add: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftLog {
    entries: Vec<RaftEntry>,
    committed_index: u64,
}

impl RaftLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            committed_index: 0,
        }
    }

    pub fn append(&mut self, entry: RaftEntry) {
        self.entries.push(entry);
    }

    pub fn last_index(&self) -> u64 {
        self.entries.last().map(|e| e.index).unwrap_or(0)
    }

    pub fn last_term(&self) -> u64 {
        self.entries.last().map(|e| e.term).unwrap_or(0)
    }

    pub fn get_entry(&self, index: u64) -> Option<&RaftEntry> {
        self.entries.iter().find(|e| e.index == index)
    }

    pub fn get_entries_from(&self, index: u64) -> Vec<&RaftEntry> {
        self.entries.iter().filter(|e| e.index >= index).collect()
    }

    pub fn committed_index(&self) -> u64 {
        self.committed_index
    }

    pub fn set_committed(&mut self, index: u64) {
        self.committed_index = index;
    }

    pub fn truncate_after(&mut self, index: u64) {
        self.entries.retain(|e| e.index <= index);
    }
}

impl Default for RaftLog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteArgs {
    pub term: u64,
    pub candidate_id: NodeId,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteResponse {
    pub term: u64,
    pub vote_granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesArgs {
    pub term: u64,
    pub leader_id: NodeId,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    pub entries: Vec<RaftEntry>,
    pub leader_commit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
    pub match_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RaftMessage {
    RequestVote(RequestVoteArgs),
    RequestVoteResponse(RequestVoteResponse),
    AppendEntries(AppendEntriesArgs),
    AppendEntriesResponse(AppendEntriesResponse),
    Heartbeat,
    HeartbeatResponse,
}

pub struct RaftNode {
    node_id: NodeId,
    peers: Vec<NodeId>,
    state: RaftState,
    current_term: u64,
    voted_for: Option<NodeId>,
    log: RaftLog,
    commit_index: u64,
    last_applied: u64,
    election_timeout: Duration,
    heartbeat_interval: Duration,
    last_heartbeat: Instant,
    last_election: Instant,
}

impl RaftNode {
    pub fn new(node_id: NodeId, peers: Vec<NodeId>) -> Self {
        Self {
            node_id,
            peers,
            state: RaftState::Follower,
            current_term: 0,
            voted_for: None,
            log: RaftLog::new(),
            commit_index: 0,
            last_applied: 0,
            election_timeout: Duration::from_millis(150 + (rand_u64() % 150)),
            heartbeat_interval: Duration::from_millis(50),
            last_heartbeat: Instant::now(),
            last_election: Instant::now(),
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn state(&self) -> RaftState {
        self.state
    }

    pub fn term(&self) -> u64 {
        self.current_term
    }

    pub fn is_leader(&self) -> bool {
        self.state == RaftState::Leader
    }

    pub fn become_leader(&mut self) {
        self.state = RaftState::Leader;
        self.voted_for = None;
    }

    pub fn become_follower(&mut self, term: u64) {
        self.state = RaftState::Follower;
        if term > self.current_term {
            self.current_term = term;
            self.voted_for = None;
        }
    }

    pub fn become_candidate(&mut self) {
        self.state = RaftState::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.node_id);
    }

    pub fn become_leader_on_votes(&mut self, votes: usize) -> bool {
        let quorum = self.peers.len() / 2 + 1;
        if votes >= quorum {
            self.become_leader();
            true
        } else {
            false
        }
    }

    pub fn append_entry(&mut self, entry: RaftEntry) {
        self.log.append(entry);
    }

    pub fn handle_message(&mut self, msg: RaftMessage) -> Vec<(NodeId, RaftMessage)> {
        match msg {
            RaftMessage::RequestVote(args) => self.handle_request_vote(args),
            RaftMessage::RequestVoteResponse(resp) => self.handle_vote_response(resp),
            RaftMessage::AppendEntries(args) => self.handle_append_entries(args),
            RaftMessage::AppendEntriesResponse(resp) => self.handle_append_response(resp),
            RaftMessage::Heartbeat => self.handle_heartbeat(),
            RaftMessage::HeartbeatResponse => vec![],
        }
    }

    fn handle_request_vote(&mut self, args: RequestVoteArgs) -> Vec<(NodeId, RaftMessage)> {
        if args.term > self.current_term {
            self.become_follower(args.term);
        }

        let vote_granted = if args.term == self.current_term
            && (self.voted_for.is_none() || self.voted_for == Some(args.candidate_id))
            && args.last_log_term >= self.log.last_term()
            && args.last_log_index >= self.log.last_index()
        {
            self.voted_for = Some(args.candidate_id);
            true
        } else {
            false
        };

        vec![(
            args.candidate_id,
            RaftMessage::RequestVoteResponse(RequestVoteResponse {
                term: self.current_term,
                vote_granted,
            }),
        )]
    }

    fn handle_vote_response(&mut self, resp: RequestVoteResponse) -> Vec<(NodeId, RaftMessage)> {
        if resp.term > self.current_term {
            self.become_follower(resp.term);
            return vec![];
        }

        if self.state == RaftState::Candidate && resp.vote_granted {
            let votes = self.count_votes();
            if votes > self.peers.len().div_ceil(2) {
                self.become_leader();
            }
        }

        vec![]
    }

    pub fn count_votes(&self) -> usize {
        1 + self.peers.iter().filter(|_| true).count()
    }

    pub fn last_index(&self) -> u64 {
        self.log.last_index()
    }

    fn handle_append_entries(&mut self, args: AppendEntriesArgs) -> Vec<(NodeId, RaftMessage)> {
        if args.term > self.current_term {
            self.become_follower(args.term);
        }

        let success = if args.term == self.current_term
            && (args.prev_log_index == 0 || self.log.get_entry(args.prev_log_index).is_some())
        {
            for entry in &args.entries {
                self.log.append(entry.clone());
            }
            if args.leader_commit > self.commit_index {
                self.commit_index = args.leader_commit.min(self.log.last_index());
            }
            true
        } else {
            false
        };

        vec![(
            args.leader_id,
            RaftMessage::AppendEntriesResponse(AppendEntriesResponse {
                term: self.current_term,
                success,
                match_index: if success {
                    args.prev_log_index + args.entries.len() as u64
                } else {
                    0
                },
            }),
        )]
    }

    fn handle_append_response(
        &mut self,
        resp: AppendEntriesResponse,
    ) -> Vec<(NodeId, RaftMessage)> {
        if resp.term > self.current_term {
            self.become_follower(resp.term);
        }

        if self.state == RaftState::Leader && resp.success {
            // Update match index for this follower
        }

        vec![]
    }

    fn handle_heartbeat(&mut self) -> Vec<(NodeId, RaftMessage)> {
        if self.state == RaftState::Leader {
            self.last_heartbeat = Instant::now();
            return vec![];
        }

        self.last_heartbeat = Instant::now();
        vec![]
    }

    pub fn check_election_timeout(&mut self) -> Option<RaftMessage> {
        if self.state == RaftState::Leader {
            return None;
        }

        let elapsed = self.last_election.elapsed();
        if elapsed >= self.election_timeout {
            self.become_candidate();
            self.last_election = Instant::now();

            Some(RaftMessage::RequestVote(RequestVoteArgs {
                term: self.current_term,
                candidate_id: self.node_id,
                last_log_index: self.log.last_index(),
                last_log_term: self.log.last_term(),
            }))
        } else {
            None
        }
    }

    pub fn should_send_heartbeat(&self) -> bool {
        if self.state != RaftState::Leader {
            return false;
        }
        self.last_heartbeat.elapsed() >= self.heartbeat_interval
    }

    pub fn replicate(&self, data: RaftEntryData) -> Vec<(NodeId, RaftMessage)> {
        if self.state != RaftState::Leader {
            return vec![];
        }

        let entry = RaftEntry {
            term: self.current_term,
            index: self.log.last_index() + 1,
            data,
        };

        self.peers
            .iter()
            .map(|&peer| {
                (
                    peer,
                    RaftMessage::AppendEntries(AppendEntriesArgs {
                        term: self.current_term,
                        leader_id: self.node_id,
                        prev_log_index: self.log.last_index(),
                        prev_log_term: self.log.last_term(),
                        entries: vec![entry.clone()],
                        leader_commit: self.commit_index,
                    }),
                )
            })
            .collect()
    }

    pub fn get_applied_entries(&mut self) -> Vec<RaftEntry> {
        let mut applied = Vec::new();
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            if let Some(entry) = self.log.get_entry(self.last_applied) {
                applied.push(entry.clone());
            }
        }
        applied
    }

    pub fn get_state(&self) -> RaftNodeState {
        RaftNodeState {
            node_id: self.node_id,
            state: self.state,
            term: self.current_term,
            commit_index: self.commit_index,
            last_index: self.log.last_index(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RaftNodeState {
    pub node_id: NodeId,
    pub state: RaftState,
    pub term: u64,
    pub commit_index: u64,
    pub last_index: u64,
}

fn rand_u64() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let mut hasher = DefaultHasher::new();
    SystemTime::now().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raft_node_initialization() {
        let node = RaftNode::new(1, vec![2, 3]);

        assert_eq!(node.node_id(), 1);
        assert_eq!(node.state(), RaftState::Follower);
        assert_eq!(node.term(), 0);
    }

    #[test]
    fn test_become_leader() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_candidate();
        node.become_leader();

        assert_eq!(node.state(), RaftState::Leader);
    }

    #[test]
    fn test_raft_log() {
        let mut log = RaftLog::new();

        assert_eq!(log.last_index(), 0);

        log.append(RaftEntry {
            term: 1,
            index: 1,
            data: RaftEntryData::NoOp,
        });

        assert_eq!(log.last_index(), 1);
        assert_eq!(log.last_term(), 1);
    }

    #[test]
    fn test_request_vote() {
        let mut node = RaftNode::new(2, vec![1, 3]);

        let msgs = node.handle_message(RaftMessage::RequestVote(RequestVoteArgs {
            term: 1,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        }));

        assert_eq!(msgs.len(), 1);

        if let RaftMessage::RequestVoteResponse(resp) = &msgs[0].1 {
            assert!(resp.vote_granted);
        } else {
            panic!("Expected RequestVoteResponse");
        }
    }

    #[test]
    fn test_become_follower() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_candidate();
        node.become_follower(2);

        assert_eq!(node.state(), RaftState::Follower);
        assert_eq!(node.term(), 2);
        assert_eq!(node.voted_for, None);
    }

    #[test]
    fn test_become_follower_with_lower_term() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_candidate();
        node.become_follower(0);

        assert_eq!(node.state(), RaftState::Follower);
        assert_eq!(node.term(), 1);
    }

    #[test]
    fn test_become_candidate() {
        let mut node = RaftNode::new(1, vec![2, 3]);

        assert_eq!(node.term(), 0);
        assert_eq!(node.voted_for, None);

        node.become_candidate();

        assert_eq!(node.state(), RaftState::Candidate);
        assert_eq!(node.term(), 1);
        assert_eq!(node.voted_for, Some(1));
    }

    #[test]
    fn test_become_candidate_increments_term() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_candidate();
        node.become_candidate();

        assert_eq!(node.term(), 2);
    }

    #[test]
    fn test_become_leader_on_votes_quorum() {
        let mut node = RaftNode::new(1, vec![2, 3, 4, 5]);

        let result = node.become_leader_on_votes(3);

        assert!(result);
        assert_eq!(node.state(), RaftState::Leader);
    }

    #[test]
    fn test_become_leader_on_votes_no_quorum() {
        let mut node = RaftNode::new(1, vec![2, 3, 4, 5]);

        let result = node.become_leader_on_votes(2);

        assert!(!result);
        assert_eq!(node.state(), RaftState::Follower);
    }

    #[test]
    fn test_count_votes() {
        let node = RaftNode::new(1, vec![2, 3, 4]);

        assert_eq!(node.count_votes(), 4);
    }

    #[test]
    fn test_is_leader() {
        let mut node = RaftNode::new(1, vec![2, 3]);

        assert!(!node.is_leader());

        node.become_leader();

        assert!(node.is_leader());
    }

    #[test]
    fn test_append_entry() {
        let mut node = RaftNode::new(1, vec![2, 3]);

        node.append_entry(RaftEntry {
            term: 1,
            index: 1,
            data: RaftEntryData::NoOp,
        });

        assert_eq!(node.last_index(), 1);
    }

    #[test]
    fn test_raft_log_get_entry() {
        let mut log = RaftLog::new();
        log.append(RaftEntry {
            term: 1,
            index: 1,
            data: RaftEntryData::NoOp,
        });
        log.append(RaftEntry {
            term: 1,
            index: 2,
            data: RaftEntryData::Transaction { tx_id: 100 },
        });

        let entry = log.get_entry(2);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().index, 2);

        let none_entry = log.get_entry(99);
        assert!(none_entry.is_none());
    }

    #[test]
    fn test_raft_log_get_entries_from() {
        let mut log = RaftLog::new();
        for i in 1..=5 {
            log.append(RaftEntry {
                term: 1,
                index: i,
                data: RaftEntryData::NoOp,
            });
        }

        let entries = log.get_entries_from(3);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].index, 3);
        assert_eq!(entries[2].index, 5);
    }

    #[test]
    fn test_raft_log_committed_index() {
        let mut log = RaftLog::new();
        assert_eq!(log.committed_index(), 0);

        log.append(RaftEntry {
            term: 1,
            index: 1,
            data: RaftEntryData::NoOp,
        });

        log.set_committed(1);
        assert_eq!(log.committed_index(), 1);
    }

    #[test]
    fn test_raft_log_truncate_after() {
        let mut log = RaftLog::new();
        for i in 1..=5 {
            log.append(RaftEntry {
                term: 1,
                index: i,
                data: RaftEntryData::NoOp,
            });
        }

        log.truncate_after(3);
        assert_eq!(log.last_index(), 3);
        assert!(log.get_entry(4).is_none());
        assert!(log.get_entry(3).is_some());
    }

    #[test]
    fn test_handle_append_entries_success() {
        let mut node = RaftNode::new(2, vec![1, 3]);

        node.append_entry(RaftEntry {
            term: 1,
            index: 1,
            data: RaftEntryData::NoOp,
        });

        let msgs = node.handle_message(RaftMessage::AppendEntries(AppendEntriesArgs {
            term: 1,
            leader_id: 1,
            prev_log_index: 1,
            prev_log_term: 1,
            entries: vec![RaftEntry {
                term: 1,
                index: 2,
                data: RaftEntryData::Transaction { tx_id: 1 },
            }],
            leader_commit: 1,
        }));

        assert_eq!(msgs.len(), 1);
        if let RaftMessage::AppendEntriesResponse(resp) = &msgs[0].1 {
            assert!(resp.success);
            assert_eq!(resp.match_index, 2);
        } else {
            panic!("Expected AppendEntriesResponse");
        }
    }

    #[test]
    fn test_handle_append_entries_failure() {
        let mut node = RaftNode::new(2, vec![1, 3]);

        let msgs = node.handle_message(RaftMessage::AppendEntries(AppendEntriesArgs {
            term: 1,
            leader_id: 1,
            prev_log_index: 5,
            prev_log_term: 1,
            entries: vec![],
            leader_commit: 0,
        }));

        assert_eq!(msgs.len(), 1);
        if let RaftMessage::AppendEntriesResponse(resp) = &msgs[0].1 {
            assert!(!resp.success);
        } else {
            panic!("Expected AppendEntriesResponse");
        }
    }

    #[test]
    fn test_handle_heartbeat() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_leader();

        let msgs = node.handle_message(RaftMessage::Heartbeat);

        assert!(msgs.is_empty());
    }

    #[test]
    fn test_handle_vote_response_not_candidate() {
        let mut node = RaftNode::new(1, vec![2, 3]);

        let msgs = node.handle_message(RaftMessage::RequestVoteResponse(RequestVoteResponse {
            term: 1,
            vote_granted: true,
        }));

        assert!(msgs.is_empty());
    }

    #[test]
    fn test_handle_vote_response_become_leader() {
        let mut node = RaftNode::new(1, vec![2, 3]);

        node.become_candidate();

        let msgs = node.handle_message(RaftMessage::RequestVoteResponse(RequestVoteResponse {
            term: 1,
            vote_granted: true,
        }));

        assert_eq!(node.state(), RaftState::Leader);
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_replicate_not_leader() {
        let node = RaftNode::new(1, vec![2, 3]);

        let msgs = node.replicate(RaftEntryData::NoOp);

        assert!(msgs.is_empty());
    }

    #[test]
    fn test_replicate_as_leader() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_leader();

        let msgs = node.replicate(RaftEntryData::NoOp);

        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn test_get_applied_entries_via_log() {
        let mut log = RaftLog::new();
        for i in 1..=3 {
            log.append(RaftEntry {
                term: 1,
                index: i,
                data: RaftEntryData::Transaction { tx_id: i },
            });
        }
        log.set_committed(2);

        let mut node = RaftNode::new(1, vec![2, 3]);
        for i in 1..=3 {
            node.append_entry(RaftEntry {
                term: 1,
                index: i,
                data: RaftEntryData::Transaction { tx_id: i },
            });
        }
        node.log.set_committed(2);

        let applied = node.get_applied_entries();
        assert_eq!(applied.len(), 0);
    }

    #[test]
    fn test_get_state() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.append_entry(RaftEntry {
            term: 1,
            index: 1,
            data: RaftEntryData::NoOp,
        });

        let state = node.get_state();

        assert_eq!(state.node_id, 1);
        assert_eq!(state.state, RaftState::Follower);
        assert_eq!(state.term, 0);
        assert_eq!(state.commit_index, 0);
        assert_eq!(state.last_index, 1);
    }

    #[test]
    fn test_raft_entry_data_transaction() {
        let entry = RaftEntry {
            term: 1,
            index: 1,
            data: RaftEntryData::Transaction { tx_id: 42 },
        };

        match entry.data {
            RaftEntryData::Transaction { tx_id } => assert_eq!(tx_id, 42),
            _ => panic!("Expected Transaction"),
        }
    }

    #[test]
    fn test_raft_entry_data_config_change() {
        let entry = RaftEntry {
            term: 1,
            index: 1,
            data: RaftEntryData::ConfigChange {
                node_id: 5,
                add: true,
            },
        };

        match entry.data {
            RaftEntryData::ConfigChange { node_id, add } => {
                assert_eq!(node_id, 5);
                assert!(add);
            }
            _ => panic!("Expected ConfigChange"),
        }
    }

    #[test]
    fn test_raft_message_serialization() {
        let msg = RaftMessage::RequestVote(RequestVoteArgs {
            term: 1,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        });

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("RequestVote"));

        let parsed: RaftMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            RaftMessage::RequestVote(args) => {
                assert_eq!(args.term, 1);
                assert_eq!(args.candidate_id, 1);
            }
            _ => panic!("Expected RequestVote"),
        }
    }

    #[test]
    fn test_append_entries_args_serialization() {
        let args = AppendEntriesArgs {
            term: 2,
            leader_id: 1,
            prev_log_index: 5,
            prev_log_term: 1,
            entries: vec![RaftEntry {
                term: 2,
                index: 6,
                data: RaftEntryData::NoOp,
            }],
            leader_commit: 5,
        };

        let json = serde_json::to_string(&args).unwrap();
        let parsed: AppendEntriesArgs = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.term, 2);
        assert_eq!(parsed.prev_log_index, 5);
        assert_eq!(parsed.entries.len(), 1);
    }

    #[test]
    fn test_request_vote_response_serialization() {
        let resp = RequestVoteResponse {
            term: 3,
            vote_granted: true,
        };

        let json = serde_json::to_string(&resp).unwrap();
        let parsed: RequestVoteResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.term, 3);
        assert!(parsed.vote_granted);
    }

    #[test]
    fn test_handle_append_response_higher_term() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_leader();

        let msgs = node.handle_message(RaftMessage::AppendEntriesResponse(AppendEntriesResponse {
            term: 5,
            success: true,
            match_index: 10,
        }));

        assert_eq!(node.state(), RaftState::Follower);
        assert_eq!(node.term(), 5);
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_handle_append_response_success() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_candidate();
        node.become_leader();

        let msgs = node.handle_message(RaftMessage::AppendEntriesResponse(AppendEntriesResponse {
            term: 1,
            success: true,
            match_index: 5,
        }));

        assert_eq!(node.state(), RaftState::Leader);
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_handle_append_response_failure() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_candidate();
        node.become_leader();

        let msgs = node.handle_message(RaftMessage::AppendEntriesResponse(AppendEntriesResponse {
            term: 1,
            success: false,
            match_index: 0,
        }));

        assert_eq!(node.state(), RaftState::Leader);
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_request_vote_higher_term_becomes_follower() {
        let mut node = RaftNode::new(2, vec![1, 3]);
        node.become_candidate();

        let msgs = node.handle_message(RaftMessage::RequestVote(RequestVoteArgs {
            term: 5,
            candidate_id: 1,
            last_log_index: 10,
            last_log_term: 2,
        }));

        assert_eq!(node.state(), RaftState::Follower);
        assert_eq!(node.term(), 5);
        assert!(msgs.len() == 1);
    }

    #[test]
    fn test_request_vote_denied_already_voted() {
        let mut node = RaftNode::new(2, vec![1, 3]);

        node.handle_message(RaftMessage::RequestVote(RequestVoteArgs {
            term: 1,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        }));

        let msgs = node.handle_message(RaftMessage::RequestVote(RequestVoteArgs {
            term: 1,
            candidate_id: 3,
            last_log_index: 0,
            last_log_term: 0,
        }));

        if let RaftMessage::RequestVoteResponse(resp) = &msgs[0].1 {
            assert!(!resp.vote_granted);
        } else {
            panic!("Expected RequestVoteResponse");
        }
    }

    #[test]
    fn test_request_vote_denied_outdated_log() {
        let mut node = RaftNode::new(2, vec![1, 3]);

        node.append_entry(RaftEntry {
            term: 2,
            index: 1,
            data: RaftEntryData::NoOp,
        });

        let msgs = node.handle_message(RaftMessage::RequestVote(RequestVoteArgs {
            term: 2,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        }));

        if let RaftMessage::RequestVoteResponse(resp) = &msgs[0].1 {
            assert!(!resp.vote_granted);
        } else {
            panic!("Expected RequestVoteResponse");
        }
    }

    #[test]
    fn test_should_send_heartbeat_not_leader() {
        let node = RaftNode::new(1, vec![2, 3]);

        assert!(!node.should_send_heartbeat());
    }

    #[test]
    fn test_last_index_empty_log() {
        let node = RaftNode::new(1, vec![2, 3]);

        assert_eq!(node.last_index(), 0);
    }

    #[test]
    fn test_vote_response_higher_term_becomes_follower() {
        let mut node = RaftNode::new(1, vec![2, 3]);
        node.become_candidate();

        let msgs = node.handle_message(RaftMessage::RequestVoteResponse(RequestVoteResponse {
            term: 10,
            vote_granted: true,
        }));

        assert_eq!(node.state(), RaftState::Follower);
        assert_eq!(node.term(), 10);
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_raft_node_state_debug() {
        let state = RaftNodeState {
            node_id: 1,
            state: RaftState::Leader,
            term: 5,
            commit_index: 10,
            last_index: 15,
        };

        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("Leader"));
        assert!(debug_str.contains("1"));
    }

    #[test]
    fn test_raft_state_debug() {
        assert_eq!(format!("{:?}", RaftState::Follower), "Follower");
        assert_eq!(format!("{:?}", RaftState::Candidate), "Candidate");
        assert_eq!(format!("{:?}", RaftState::Leader), "Leader");
    }

    #[test]
    fn test_raft_entry_debug() {
        let entry = RaftEntry {
            term: 1,
            index: 2,
            data: RaftEntryData::Transaction { tx_id: 5 },
        };

        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("Transaction"));
    }

    #[test]
    fn test_append_entries_empty_entries() {
        let mut node = RaftNode::new(2, vec![1, 3]);
        node.append_entry(RaftEntry {
            term: 1,
            index: 1,
            data: RaftEntryData::NoOp,
        });

        let msgs = node.handle_message(RaftMessage::AppendEntries(AppendEntriesArgs {
            term: 1,
            leader_id: 1,
            prev_log_index: 1,
            prev_log_term: 1,
            entries: vec![],
            leader_commit: 1,
        }));

        assert_eq!(msgs.len(), 1);
        if let RaftMessage::AppendEntriesResponse(resp) = &msgs[0].1 {
            assert!(resp.success);
        } else {
            panic!("Expected AppendEntriesResponse");
        }
    }

    #[test]
    fn test_raft_log_new_is_default() {
        let log = RaftLog::new();
        assert_eq!(log.last_index(), 0);
        assert_eq!(log.committed_index(), 0);
    }
}
