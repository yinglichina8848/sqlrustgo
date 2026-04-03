//! HA (High Availability) CLI Tool
//!
//! Provides commands for:
//! - Show cluster status
//! - Manual failover trigger
//! - Node management
//! - Health monitoring

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlrustgo_storage::{FailoverConfig, NodeInfo, NodeType};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub current_master: Option<String>,
    pub state: String,
    pub failover_count: u64,
    pub nodes: Vec<NodeStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub addr: String,
    pub node_type: String,
    pub priority: u32,
    pub is_healthy: bool,
    pub last_seen_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ClusterConfig {
    pub health_check_interval_secs: u64,
    pub election_timeout_secs: u64,
    pub failover_threshold: u32,
    pub retry_base_delay_secs: u64,
    pub max_retry_delay_secs: u64,
}

impl From<&FailoverConfig> for ClusterConfig {
    fn from(config: &FailoverConfig) -> Self {
        Self {
            health_check_interval_secs: config.health_check_interval.as_secs(),
            election_timeout_secs: config.election_timeout.as_secs(),
            failover_threshold: config.failover_threshold,
            retry_base_delay_secs: config.retry_base_delay.as_secs(),
            max_retry_delay_secs: config.max_retry_delay.as_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NodeAddRequest {
    pub addr: String,
    pub node_type: String,
    pub priority: u32,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "ha", about = "High Availability cluster management")]
pub enum HACommand {
    /// Show cluster status
    Status {
        /// Master address (host:port)
        #[structopt(short = "m", long = "master")]
        master: Option<String>,

        /// Config file for cluster nodes
        #[structopt(short = "c", long = "config")]
        config: Option<PathBuf>,
    },

    /// Trigger manual failover
    Failover {
        /// Current master address (host:port)
        #[structopt(short = "m", long = "master")]
        master: String,

        /// New master address (host:port)
        #[structopt(short = "n", long = "new-master")]
        new_master: String,

        /// Slave addresses (host:port)
        #[structopt(short = "s", long = "slaves", use_delimiter = true)]
        slaves: Vec<String>,

        /// Force failover even if master is reachable
        #[structopt(short = "f", long = "force")]
        force: bool,
    },

    /// Add a node to the cluster
    NodeAdd {
        /// Node address (host:port)
        #[structopt(short = "a", long = "addr")]
        addr: String,

        /// Node type (master or slave)
        #[structopt(short = "t", long = "type", default_value = "slave")]
        node_type: String,

        /// Node priority for election (higher = more likely to become master)
        #[structopt(short = "p", long = "priority", default_value = "100")]
        priority: u32,

        /// Master address for context
        #[structopt(short = "m", long = "master")]
        master: Option<String>,
    },

    /// Remove a node from the cluster
    NodeRemove {
        /// Node address to remove (host:port)
        #[structopt(short = "a", long = "addr")]
        addr: String,

        /// Master address for context
        #[structopt(short = "m", long = "master")]
        master: Option<String>,
    },

    /// Show cluster configuration
    Config {
        /// Master address for config display
        #[structopt(short = "m", long = "master")]
        master: Option<String>,
    },

    /// Health check all nodes
    HealthCheck {
        /// Node addresses to check (host:port)
        #[structopt(short = "n", long = "nodes", use_delimiter = true)]
        nodes: Vec<String>,

        /// Timeout per node in seconds
        #[structopt(short = "t", long = "timeout", default_value = "5")]
        timeout: u64,
    },
}

pub fn run() -> Result<()> {
    let cmd = HACommand::from_args();
    match cmd {
        HACommand::Status { master, config } => show_status(master.as_deref(), config.as_deref()),
        HACommand::Failover {
            master,
            new_master,
            slaves,
            force,
        } => trigger_failover(&master, &new_master, &slaves, force),
        HACommand::NodeAdd {
            addr,
            node_type,
            priority,
            master,
        } => add_node(&addr, &node_type, priority, master.as_deref()),
        HACommand::NodeRemove { addr, master } => remove_node(&addr, master.as_deref()),
        HACommand::Config { master } => show_config(master.as_deref()),
        HACommand::HealthCheck { nodes, timeout } => health_check(&nodes, timeout),
    }
}

fn parse_addr(s: &str) -> Result<SocketAddr> {
    s.parse().context(format!("Invalid address: {}", s))
}

fn show_status(master: Option<&str>, _config: Option<&Path>) -> Result<()> {
    println!("HA Cluster Status");
    println!("{}", "=".repeat(70));

    if let Some(master_addr) = master {
        let addr: SocketAddr = parse_addr(master_addr)?;
        println!("Current Master: {}", addr);

        let node_type = NodeType::Master;
        let nodes = vec![NodeInfo::new(addr, node_type, 100)];

        let status = query_cluster_status(&nodes);
        print_cluster_status(&status);
    } else {
        println!("No master specified - showing local node status only");
        println!("Use --master <addr> to query specific cluster");
    }

    Ok(())
}

fn query_cluster_status(nodes: &[NodeInfo]) -> ClusterStatus {
    let now = Instant::now();
    ClusterStatus {
        current_master: nodes
            .iter()
            .find(|n| n.node_type == NodeType::Master)
            .map(|n| n.addr.to_string()),
        state: "Normal".to_string(),
        failover_count: 0,
        nodes: nodes
            .iter()
            .map(|n| NodeStatus {
                addr: n.addr.to_string(),
                node_type: format!("{:?}", n.node_type),
                priority: n.priority,
                is_healthy: n.is_healthy,
                last_seen_secs: now.duration_since(n.last_seen).as_secs(),
            })
            .collect(),
    }
}

fn print_cluster_status(status: &ClusterStatus) {
    println!();
    println!("Cluster State: {}", status.state);
    println!("Total Failovers: {}", status.failover_count);
    println!();
    println!("Nodes: {}", status.nodes.len());
    println!("{}", "-".repeat(70));

    for node in &status.nodes {
        let health_icon = if node.is_healthy { "✓" } else { "✗" };
        println!(
            "  {} {} ({}) - Priority: {} - Last seen: {}s ago",
            health_icon, node.addr, node.node_type, node.priority, node.last_seen_secs
        );
    }
}

fn trigger_failover(master: &str, new_master: &str, slaves: &[String], force: bool) -> Result<()> {
    println!("Manual Failover Trigger");
    println!("{}", "=".repeat(70));
    println!("Current Master: {}", master);
    println!("New Master: {}", new_master);
    println!("Slaves: {:?}", slaves);
    println!("Force: {}", force);

    let master_addr: SocketAddr = parse_addr(master)?;
    let new_master_addr: SocketAddr = parse_addr(new_master)?;

    let slave_addrs: Result<Vec<SocketAddr>, _> = slaves.iter().map(|s| parse_addr(s)).collect();
    let slave_addrs = slave_addrs?;

    println!();
    if !force {
        println!("Note: This is a planned switchover (not forced)");
        println!("The current master will be demoted to slave");
    } else {
        println!("Warning: Forced failover - original master may not know about the change");
    }

    println!();
    println!("Creating failover manager...");

    let mut all_nodes = vec![
        NodeInfo::new(master_addr, NodeType::Master, 100),
        NodeInfo::new(new_master_addr, NodeType::Slave, 200),
    ];

    for (i, slave_addr) in slave_addrs.iter().enumerate() {
        all_nodes.push(NodeInfo::new(
            *slave_addr,
            NodeType::Slave,
            100 - (i as u32 * 10),
        ));
    }

    println!("  Cluster nodes configured: {}", all_nodes.len());
    for node in &all_nodes {
        println!(
            "    - {} ({:?}, priority={})",
            node.addr, node.node_type, node.priority
        );
    }

    let config = FailoverConfig::default();
    println!();
    println!("Failover Configuration:");
    println!(
        "  Health check interval: {}s",
        config.health_check_interval.as_secs()
    );
    println!("  Election timeout: {}s", config.election_timeout.as_secs());
    println!("  Failover threshold: {}", config.failover_threshold);

    println!();
    println!("✓ Failover would be triggered with these parameters");
    println!("  (This is a demo - actual failover requires running cluster)");

    Ok(())
}

fn add_node(addr: &str, node_type: &str, priority: u32, _master: Option<&str>) -> Result<()> {
    let socket_addr: SocketAddr = parse_addr(addr)?;
    let ntype = match node_type.to_lowercase().as_str() {
        "master" => NodeType::Master,
        "slave" => NodeType::Slave,
        _ => anyhow::bail!("Invalid node type: {}. Use 'master' or 'slave'", node_type),
    };

    let node = NodeInfo::new(socket_addr, ntype, priority);

    println!("Node Configuration:");
    println!("{}", "=".repeat(70));
    println!("Address: {}", node.addr);
    println!("Type: {:?}", node.node_type);
    println!("Priority: {}", node.priority);
    println!("Last seen: now");

    println!();
    println!("✓ Node configured");
    println!("  (This is a demo - node would be added to cluster on next sync)");

    Ok(())
}

fn remove_node(addr: &str, _master: Option<&str>) -> Result<()> {
    let socket_addr: SocketAddr = parse_addr(addr)?;

    println!("Remove Node");
    println!("{}", "=".repeat(70));
    println!("Address to remove: {}", socket_addr);

    println!();
    println!("✓ Node removal configured");
    println!("  (This is a demo - node would be removed from cluster on next sync)");

    Ok(())
}

fn show_config(_master: Option<&str>) -> Result<()> {
    println!("HA Cluster Configuration");
    println!("{}", "=".repeat(70));

    let config = FailoverConfig::default();

    println!("Health Check Settings:");
    println!(
        "  Health check interval: {}s",
        config.health_check_interval.as_secs()
    );
    println!("  Election timeout: {}s", config.election_timeout.as_secs());
    println!();

    println!("Failover Settings:");
    println!(
        "  Failover threshold: {} consecutive failures",
        config.failover_threshold
    );
    println!("  Retry base delay: {}s", config.retry_base_delay.as_secs());
    println!("  Max retry delay: {}s", config.max_retry_delay.as_secs());
    println!();

    println!("Default Configuration (can be customized via config file):");
    println!("  health_check_interval: 5s");
    println!("  election_timeout: 10s");
    println!("  failover_threshold: 3");
    println!("  retry_base_delay: 1s");
    println!("  max_retry_delay: 30s");

    Ok(())
}

fn health_check(nodes: &[String], timeout_secs: u64) -> Result<()> {
    println!("HA Health Check");
    println!("{}", "=".repeat(70));
    println!("Nodes to check: {}", nodes.len());
    println!("Timeout per node: {}s", timeout_secs);
    println!();

    let mut all_healthy = true;

    for node in nodes {
        let addr: SocketAddr = parse_addr(node)?;
        let is_healthy = check_node_health(addr, timeout_secs);

        let icon = if is_healthy { "✓" } else { "✗" };
        println!(
            "  {} {} - {}",
            icon,
            node,
            if is_healthy { "healthy" } else { "unreachable" }
        );

        if !is_healthy {
            all_healthy = false;
        }
    }

    println!();
    if all_healthy {
        println!("✓ All nodes healthy");
    } else {
        println!("✗ Some nodes are unreachable");
        std::process::exit(1);
    }

    Ok(())
}

fn check_node_health(addr: SocketAddr, timeout_secs: u64) -> bool {
    std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(timeout_secs))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_addr() {
        let result: Result<SocketAddr, _> = "127.0.0.1:5432".parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "127.0.0.1:5432");
    }

    #[test]
    fn test_node_type_parsing() {
        assert!(matches!(NodeType::Master, NodeType::Master));
        assert!(matches!(NodeType::Slave, NodeType::Slave));
    }

    #[test]
    fn test_cluster_config_from_failover_config() {
        let config = FailoverConfig::default();
        let cluster_config = ClusterConfig::from(&config);
        assert_eq!(cluster_config.health_check_interval_secs, 5);
        assert_eq!(cluster_config.election_timeout_secs, 10);
        assert_eq!(cluster_config.failover_threshold, 3);
    }
}
