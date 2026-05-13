//! Group Membership Module
//!
//! Provides group membership management for Group Replication:
//! - Member registration and tracking
//! - View changes (join/leave)
//! - Member state management
//! - Heartbeat-based failure detection

use std::collections::{HashMap, HashSet};
#[allow(unused_imports)]
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Member role in the group
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MemberRole {
    #[default]
    Secondary,
    Primary,
    Arbitrator,
}

/// Member state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberState {
    Online,
    Recovering,
    Offline,
    Error,
}

#[allow(clippy::derivable_impls)]
impl Default for MemberState {
    fn default() -> Self {
        MemberState::Offline
    }
}

/// Member information
#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub node_id: u64,
    pub endpoint: String,
    pub role: MemberRole,
    pub state: MemberState,
    pub weight: u8,
    pub version: String,
    #[doc(hidden)]
    pub last_seen: Instant,
}

impl Serialize for MemberInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("MemberInfo", 6)?;
        state.serialize_field("node_id", &self.node_id)?;
        state.serialize_field("endpoint", &self.endpoint)?;
        state.serialize_field("role", &self.role)?;
        state.serialize_field("state", &self.state)?;
        state.serialize_field("weight", &self.weight)?;
        state.serialize_field("version", &self.version)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for MemberInfo {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(MemberInfo {
            node_id: 0,
            endpoint: String::new(),
            role: MemberRole::default(),
            state: MemberState::default(),
            weight: 100,
            version: "1.0.0".to_string(),
            last_seen: Instant::now(),
        })
    }
}

impl MemberInfo {
    pub fn new(node_id: u64, endpoint: String) -> Self {
        Self {
            node_id,
            endpoint,
            role: MemberRole::default(),
            state: MemberState::default(),
            weight: 100,
            version: "1.0.0".to_string(),
            last_seen: Instant::now(),
        }
    }

    pub fn is_online(&self) -> bool {
        self.state == MemberState::Online
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = Instant::now();
    }
}

/// View represents the current group membership at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    pub view_id: ViewId,
    pub members: HashSet<u64>,
    pub primary_member: Option<u64>,
    pub seq_number: u64,
    pub timestamp: u64,
}

impl View {
    pub fn new(view_id: ViewId, members: HashSet<u64>, primary: Option<u64>) -> Self {
        Self {
            view_id,
            members,
            primary_member: primary,
            seq_number: 0,
            timestamp: now_secs(),
        }
    }

    pub fn contains(&self, node_id: u64) -> bool {
        self.members.contains(&node_id)
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewId {
    pub id: u64,
    pub leader_id: u64,
}

impl ViewId {
    pub fn new(id: u64, leader_id: u64) -> Self {
        Self { id, leader_id }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// View change listener type
type ViewChangeListener = Box<dyn Fn(&View) + Send + Sync>;

/// Group membership manager
#[allow(clippy::type_complexity, dead_code)]
pub struct GroupMembership {
    /// Current view
    current_view: RwLock<View>,
    /// All known members
    members: RwLock<HashMap<u64, MemberInfo>>,
    /// Local node ID
    local_node_id: u64,
    /// Heartbeat interval
    heartbeat_interval: Duration,
    /// Failure detection threshold
    failure_threshold: Duration,
    /// View change listeners
    view_change_listeners: RwLock<Vec<ViewChangeListener>>,
}

impl GroupMembership {
    pub fn new(local_node_id: u64) -> Self {
        let mut members = HashMap::new();
        let mut member = MemberInfo::new(local_node_id, format!("node-{}", local_node_id));
        member.state = MemberState::Online;
        members.insert(local_node_id, member);

        let view = View::new(
            ViewId::new(1, local_node_id),
            vec![local_node_id].into_iter().collect(),
            Some(local_node_id),
        );

        Self {
            current_view: RwLock::new(view),
            members: RwLock::new(members),
            local_node_id,
            heartbeat_interval: Duration::from_millis(500),
            failure_threshold: Duration::from_secs(5),
            view_change_listeners: RwLock::new(Vec::new()),
        }
    }

    pub fn get_local_node_id(&self) -> u64 {
        self.local_node_id
    }

    pub fn get_current_view(&self) -> View {
        self.current_view.read().unwrap().clone()
    }

    pub fn get_online_members(&self) -> Vec<u64> {
        let members = self.members.read().unwrap();
        members
            .iter()
            .filter(|(_, m)| m.state == MemberState::Online)
            .map(|(&id, _)| id)
            .collect()
    }

    pub fn get_all_members(&self) -> Vec<MemberInfo> {
        let members = self.members.read().unwrap();
        members.values().cloned().collect()
    }

    pub fn get_member(&self, node_id: u64) -> Option<MemberInfo> {
        self.members.read().unwrap().get(&node_id).cloned()
    }

    pub fn is_primary(&self) -> bool {
        let view = self.current_view.read().unwrap();
        view.primary_member == Some(self.local_node_id)
    }

    pub fn get_primary(&self) -> Option<u64> {
        self.current_view.read().unwrap().primary_member
    }

    /// Register a new member
    pub fn add_member(&self, node_id: u64, endpoint: String) -> Result<(), MembershipError> {
        let mut members = self.members.write().unwrap();
        if members.contains_key(&node_id) {
            return Err(MembershipError::MemberAlreadyExists(node_id));
        }
        members.insert(
            node_id,
            MemberInfo {
                node_id,
                endpoint,
                role: MemberRole::Secondary,
                state: MemberState::Recovering,
                weight: 100,
                version: "1.0.0".to_string(),
                last_seen: Instant::now(),
            },
        );
        Ok(())
    }

    /// Remove a member from the group
    pub fn remove_member(&self, node_id: u64) -> Result<(), MembershipError> {
        if node_id == self.local_node_id {
            return Err(MembershipError::CannotRemoveSelf);
        }

        let mut members = self.members.write().unwrap();
        members.remove(&node_id).ok_or(MembershipError::MemberNotFound(node_id))?;

        let mut view = self.current_view.write().unwrap();
        if view.members.remove(&node_id) {
            drop(view);
            self.notify_view_change()?;
        }

        Ok(())
    }

    /// Update member state
    pub fn update_member_state(
        &self,
        node_id: u64,
        state: MemberState,
    ) -> Result<(), MembershipError> {
        let mut members = self.members.write().unwrap();
        let member = members
            .get_mut(&node_id)
            .ok_or(MembershipError::MemberNotFound(node_id))?;
        member.state = state;
        Ok(())
    }

    /// Mark member as seen (heartbeat)
    pub fn mark_member_alive(&self, node_id: u64) -> Result<(), MembershipError> {
        let mut members = self.members.write().unwrap();
        let member = members
            .get_mut(&node_id)
            .ok_or(MembershipError::MemberNotFound(node_id))?;
        member.update_last_seen();
        if member.state == MemberState::Offline {
            member.state = MemberState::Recovering;
        }
        Ok(())
    }

    /// Check for failed members
    pub fn detect_failures(&self) -> Vec<u64> {
        let now = Instant::now();
        let threshold = self.failure_threshold;
        let mut members = self.members.write().unwrap();
        let mut failed = Vec::new();

        for (node_id, member) in members.iter_mut() {
            if member.state == MemberState::Online && now.duration_since(member.last_seen) > threshold {
                member.state = MemberState::Offline;
                failed.push(*node_id);
            }
        }

        if !failed.is_empty() {
            drop(members);
            let mut view = self.current_view.write().unwrap();
            for node_id in &failed {
                view.members.remove(node_id);
            }
        }

        failed
    }

    /// Elect a new primary
    pub fn elect_primary(&self) -> Option<u64> {
        let members = self.members.read().unwrap();
        let online_members: Vec<u64> = members
            .iter()
            .filter(|(_, m)| m.state == MemberState::Online && m.role == MemberRole::Primary)
            .map(|(&id, _)| id)
            .collect();

        if online_members.is_empty() {
            // No primary, select the lowest ID as primary
            let lowest = members
                .iter()
                .filter(|(_, m)| m.state == MemberState::Online)
                .min_by_key(|(&id, _)| id)
                .map(|(&id, _)| id);
            if let Some(primary) = lowest {
                drop(members);
                let mut view = self.current_view.write().unwrap();
                view.primary_member = Some(primary);
            }
            lowest
        } else {
            Some(online_members[0])
        }
    }

    /// Create a new view with updated membership
    pub fn create_new_view(&self, members: HashSet<u64>, primary: Option<u64>) -> Result<View, MembershipError> {
        let mut view = self.current_view.write().unwrap();
        let new_id = ViewId::new(view.view_id.id + 1, view.view_id.leader_id);
        let new_view = View {
            view_id: new_id,
            members,
            primary_member: primary,
            seq_number: view.seq_number + 1,
            timestamp: now_secs(),
        };
        *view = new_view.clone();
        drop(view);
        self.notify_view_change()?;
        Ok(new_view)
    }

    /// Register a view change listener
    pub fn on_view_change<F>(&self, listener: F)
    where
        F: Fn(&View) + Send + Sync + 'static,
    {
        let mut listeners = self.view_change_listeners.write().unwrap();
        listeners.push(Box::new(listener));
    }

    fn notify_view_change(&self) -> Result<(), MembershipError> {
        let view = self.get_current_view();
        let listeners = self.view_change_listeners.read().unwrap();
        for listener in listeners.iter() {
            listener(&view);
        }
        Ok(())
    }

    /// Set member role
    pub fn set_member_role(&self, node_id: u64, role: MemberRole) -> Result<(), MembershipError> {
        let mut members = self.members.write().unwrap();
        let member = members
            .get_mut(&node_id)
            .ok_or(MembershipError::MemberNotFound(node_id))?;
        member.role = role;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum MembershipError {
    MemberNotFound(u64),
    MemberAlreadyExists(u64),
    CannotRemoveSelf,
    ViewChangeFailed(String),
}

impl std::fmt::Display for MembershipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MembershipError::MemberNotFound(id) => write!(f, "Member {} not found", id),
            MembershipError::MemberAlreadyExists(id) => write!(f, "Member {} already exists", id),
            MembershipError::CannotRemoveSelf => write!(f, "Cannot remove self from group"),
            MembershipError::ViewChangeFailed(msg) => write!(f, "View change failed: {}", msg),
        }
    }
}

impl std::error::Error for MembershipError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_group_has_local_member() {
        let gm = GroupMembership::new(1);
        let view = gm.get_current_view();
        assert!(view.contains(1));
        assert_eq!(view.primary_member, Some(1));
    }

    #[test]
    fn test_add_member() {
        let gm = GroupMembership::new(1);
        gm.add_member(2, "node-2".to_string()).unwrap();
        let members = gm.get_all_members();
        assert_eq!(members.len(), 2);
    }

    #[test]
    fn test_remove_member() {
        let gm = GroupMembership::new(1);
        gm.add_member(2, "node-2".to_string()).unwrap();
        gm.remove_member(2).unwrap();
        let members = gm.get_all_members();
        assert_eq!(members.len(), 1);
    }

    #[test]
    fn test_detect_failure() {
        let gm = GroupMembership::new(1);
        gm.add_member(2, "node-2".to_string()).unwrap();
        gm.update_member_state(2, MemberState::Online).unwrap();

        // Simulate failure by not updating last_seen
        let failed = gm.detect_failures();
        // Member should still be online (not enough time passed)
        assert!(failed.is_empty());
    }

    #[test]
    fn test_elect_primary() {
        let gm = GroupMembership::new(1);
        gm.add_member(2, "node-2".to_string()).unwrap();
        gm.add_member(3, "node-3".to_string()).unwrap();

        // Set node 2 as primary and online
        gm.set_member_role(2, MemberRole::Primary).unwrap();
        gm.update_member_state(2, MemberState::Online).unwrap();

        let primary = gm.elect_primary();
        assert_eq!(primary, Some(2));
    }

    #[test]
    fn test_view_change_notification() {
        let gm = GroupMembership::new(1);
        let notified = Arc::new(Mutex::new(false));

        let notified_clone = notified.clone();
        gm.on_view_change(move |_| {
            *notified_clone.lock().unwrap() = true;
        });

        let mut members = HashSet::new();
        members.insert(1);
        members.insert(2);
        gm.create_new_view(members, Some(1)).unwrap();

        assert!(*notified.lock().unwrap());
    }
}
