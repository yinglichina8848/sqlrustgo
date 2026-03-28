//! Failover Manager - Automatic failover for master-slave replication
//!
//! Monitors master health, performs leader election, and promotes slaves.

use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Master,
    Slave,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FailoverState {
    Normal,
    MasterUnreachable,
    ElectionInProgress,
    NewMasterPromoted,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub addr: SocketAddr,
    pub node_type: NodeType,
    pub priority: u32,
    pub last_seen: Instant,
    pub is_healthy: bool,
}

impl NodeInfo {
    pub fn new(addr: SocketAddr, node_type: NodeType, priority: u32) -> Self {
        Self {
            addr,
            node_type,
            priority,
            last_seen: Instant::now(),
            is_healthy: true,
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = Instant::now();
    }
}

pub trait FailoverCallback: Send + Sync {
    fn on_master_change(&self, new_master: SocketAddr);
    fn on_promote_to_master(&self) -> std::io::Result<()>;
    fn on_demote_to_slave(&self, new_master: SocketAddr) -> std::io::Result<()>;
    fn on_election_started(&self);
    fn on_node_unhealthy(&self, addr: SocketAddr);
}

impl<F> FailoverCallback for F
where
    F: Fn(SocketAddr) + Send + Sync,
{
    fn on_master_change(&self, _new_master: SocketAddr) {}
    fn on_promote_to_master(&self) -> std::io::Result<()> {
        Ok(())
    }
    fn on_demote_to_slave(&self, _new_master: SocketAddr) -> std::io::Result<()> {
        Ok(())
    }
    fn on_election_started(&self) {}
    fn on_node_unhealthy(&self, _addr: SocketAddr) {}
}

pub struct FailoverConfig {
    pub health_check_interval: Duration,
    pub election_timeout: Duration,
    pub failover_threshold: u32,
    pub retry_base_delay: Duration,
    pub max_retry_delay: Duration,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(5),
            election_timeout: Duration::from_secs(10),
            failover_threshold: 3,
            retry_base_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(30),
        }
    }
}

pub struct FailoverManager {
    node_addr: SocketAddr,
    node_type: Arc<AtomicBool>,
    all_nodes: Arc<RwLock<Vec<NodeInfo>>>,
    config: FailoverConfig,
    callback: Arc<dyn FailoverCallback>,

    current_master: Arc<RwLock<Option<SocketAddr>>>,
    state: Arc<RwLock<FailoverState>>,
    failover_count: Arc<AtomicU64>,
    consecutive_failures: Arc<AtomicU64>,
    is_leader: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
}

impl FailoverManager {
    pub fn new(
        node_addr: SocketAddr,
        node_type: NodeType,
        all_nodes: Vec<NodeInfo>,
        callback: Arc<dyn FailoverCallback>,
    ) -> Self {
        Self {
            node_addr,
            node_type: Arc::new(AtomicBool::new(node_type == NodeType::Master)),
            all_nodes: Arc::new(RwLock::new(all_nodes)),
            config: FailoverConfig::default(),
            callback,
            current_master: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(FailoverState::Normal)),
            failover_count: Arc::new(AtomicU64::new(0)),
            consecutive_failures: Arc::new(AtomicU64::new(0)),
            is_leader: Arc::new(AtomicBool::new(node_type == NodeType::Master)),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_config(mut self, config: FailoverConfig) -> Self {
        self.config = config;
        self
    }

    pub fn start_monitoring(&self) {
        self.is_running.store(true, Ordering::SeqCst);

        let is_running = self.is_running.clone();
        let all_nodes = self.all_nodes.clone();
        let state = self.state.clone();
        let config = self.config.clone();
        let callback = self.callback.clone();
        let current_master = self.current_master.clone();
        let node_type = self.node_type.clone();

        thread::spawn(move || {
            failover_monitor_loop(
                is_running,
                all_nodes,
                state,
                config,
                callback,
                current_master,
                node_type,
            );
        });
    }

    pub fn stop_monitoring(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn get_state(&self) -> FailoverState {
        self.state.read().unwrap().clone()
    }

    pub fn get_current_master(&self) -> Option<SocketAddr> {
        self.current_master.read().unwrap().clone()
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader.load(Ordering::SeqCst)
    }

    pub fn failover_count(&self) -> u64 {
        self.failover_count.load(Ordering::SeqCst)
    }

    pub fn add_node(&self, node: NodeInfo) {
        self.all_nodes.write().unwrap().push(node);
    }

    pub fn remove_node(&self, addr: SocketAddr) {
        self.all_nodes.write().unwrap().retain(|n| n.addr != addr);
    }

    fn execute_failover(&self) -> std::io::Result<SocketAddr> {
        {
            let mut state = self.state.write().unwrap();
            if *state == FailoverState::ElectionInProgress {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AddrInUse,
                    "Election already in progress",
                ));
            }
            *state = FailoverState::ElectionInProgress;
        }

        self.callback.on_election_started();

        let alive_nodes = self.get_alive_nodes()?;

        if alive_nodes.is_empty() {
            let mut state = self.state.write().unwrap();
            *state = FailoverState::Normal;
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "No alive nodes for election",
            ));
        }

        let new_master = self.elect_new_master(&alive_nodes)?;

        {
            let mut current = self.current_master.write().unwrap();
            *current = Some(new_master);
        }

        if new_master == self.node_addr {
            self.is_leader.store(true, Ordering::SeqCst);
            self.node_type.store(true, Ordering::SeqCst);
            self.callback.on_promote_to_master()?;
        } else {
            self.callback.on_demote_to_slave(new_master)?;
        }

        {
            let mut state = self.state.write().unwrap();
            *state = FailoverState::NewMasterPromoted;
        }

        self.callback.on_master_change(new_master);
        self.failover_count.fetch_add(1, Ordering::SeqCst);
        self.consecutive_failures.store(0, Ordering::SeqCst);

        {
            let mut state = self.state.write().unwrap();
            *state = FailoverState::Normal;
        }

        Ok(new_master)
    }

    fn check_master_health(&self, master_addr: SocketAddr) -> bool {
        TcpStream::connect_timeout(&master_addr, Duration::from_secs(1)).is_ok()
    }

    fn get_alive_nodes(&self) -> std::io::Result<Vec<NodeInfo>> {
        let nodes = self.all_nodes.read().unwrap();
        let alive: Vec<NodeInfo> = nodes
            .iter()
            .filter(|n| TcpStream::connect_timeout(&n.addr, Duration::from_secs(1)).is_ok())
            .cloned()
            .collect();

        Ok(alive)
    }

    fn elect_new_master(&self, alive_nodes: &[NodeInfo]) -> std::io::Result<SocketAddr> {
        let mut candidates: Vec<_> = alive_nodes.iter().collect();
        candidates.sort_by(|a, b| b.priority.cmp(&a.priority));

        let best = candidates.first().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "No candidate nodes")
        })?;

        let mut delay = self.config.retry_base_delay;
        for node in candidates.iter().skip(1) {
            if node.priority == best.priority {
                thread::sleep(delay);
                delay = std::cmp::min(delay * 2, self.config.max_retry_delay);
            }
        }

        Ok(best.addr)
    }
}

fn failover_monitor_loop(
    is_running: Arc<AtomicBool>,
    all_nodes: Arc<RwLock<Vec<NodeInfo>>>,
    state: Arc<RwLock<FailoverState>>,
    config: FailoverConfig,
    callback: Arc<dyn FailoverCallback>,
    current_master: Arc<RwLock<Option<SocketAddr>>>,
    node_type: Arc<AtomicBool>,
) {
    while is_running.load(Ordering::SeqCst) {
        let master = current_master.read().unwrap().clone();

        if let Some(master_addr) = master {
            let healthy = TcpStream::connect_timeout(&master_addr, Duration::from_secs(1)).is_ok();

            if !healthy {
                eprintln!("Master {} is unreachable", master_addr);
                callback.on_node_unhealthy(master_addr);

                let mut failures = this_mut(&state, |s| {
                    if *s == FailoverState::Normal {
                        *s = FailoverState::MasterUnreachable;
                    }
                    if *s == FailoverState::MasterUnreachable {
                        1u64
                    } else {
                        0u64
                    }
                });

                if failures >= config.failover_threshold as u64 {
                    eprintln!("Failover threshold reached, initiating election");
                    let manager = FailoverManager {
                        node_addr: master_addr,
                        node_type: node_type.clone(),
                        all_nodes: all_nodes.clone(),
                        config: config.clone(),
                        callback: callback.clone(),
                        current_master: current_master.clone(),
                        state: state.clone(),
                        failover_count: Arc::new(AtomicU64::new(0)),
                        consecutive_failures: Arc::new(AtomicU64::new(0)),
                        is_leader: Arc::new(AtomicBool::new(false)),
                        is_running: Arc::new(AtomicBool::new(false)),
                    };
                    let _ = manager.execute_failover();
                }
            }
        }

        thread::sleep(config.health_check_interval);
    }
}

fn this_mut<T, F>(arc: &Arc<RwLock<T>>, f: F) -> u64
where
    T: Clone,
    F: FnOnce(&mut T) -> u64,
{
    let guard = arc.write().unwrap();
    let mut val = guard.clone();
    let result = f(&mut val);
    result
}

impl Clone for FailoverManager {
    fn clone(&self) -> Self {
        Self {
            node_addr: self.node_addr,
            node_type: self.node_type.clone(),
            all_nodes: self.all_nodes.clone(),
            config: self.config.clone(),
            callback: self.callback.clone(),
            current_master: self.current_master.clone(),
            state: self.state.clone(),
            failover_count: self.failover_count.clone(),
            consecutive_failures: self.consecutive_failures.clone(),
            is_leader: self.is_leader.clone(),
            is_running: self.is_running.clone(),
        }
    }
}

impl Clone for FailoverConfig {
    fn clone(&self) -> Self {
        Self {
            health_check_interval: self.health_check_interval,
            election_timeout: self.election_timeout,
            failover_threshold: self.failover_threshold,
            retry_base_delay: self.retry_base_delay,
            max_retry_delay: self.max_retry_delay,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_info_creation() {
        let addr: SocketAddr = "127.0.0.1:3333".parse().unwrap();
        let node = NodeInfo::new(addr, NodeType::Master, 100);

        assert_eq!(node.addr, addr);
        assert_eq!(node.node_type, NodeType::Master);
        assert_eq!(node.priority, 100);
        assert!(node.is_healthy);
    }

    #[test]
    fn test_failover_config_default() {
        let config = FailoverConfig::default();
        assert_eq!(config.health_check_interval, Duration::from_secs(5));
        assert_eq!(config.failover_threshold, 3);
    }
}
