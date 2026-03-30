//! Raft consensus protocol implementation
//!
//! Provides leader election and log replication for distributed coordination.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

pub type NodeId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

impl Default for RaftState {
    fn default() -> Self {
        RaftState::Follower
    }
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
            if votes > (self.peers.len() + 1) / 2 {
                self.become_leader();
            }
        }

        vec![]
    }

    fn count_votes(&self) -> usize {
        1 + self.peers.iter().filter(|_| true).count()
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
}
