use std::time::Duration;

/// Network configuration for SCP transport layer
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Local listen address (e.g., "/ip4/0.0.0.0/udp/0/quic-v1")
    pub listen_addr: String,

    /// Bootstrap nodes for DHT discovery
    pub bootstrap_nodes: Vec<String>,

    /// Whether to enable mDNS for LAN discovery
    pub enable_mdns: bool,

    /// Whether to enable AutoNAT
    pub enable_autonat: bool,

    /// Whether to enable relay client (for NAT traversal)
    pub enable_relay_client: bool,

    /// Whether to enable relay server
    pub enable_relay_server: bool,

    /// Kademlia DHT replication factor
    pub kademlia_replication: usize,

    /// GossipSub heartbeat interval
    pub gossipsub_heartbeat: Duration,

    /// GossipSub mesh parameters
    pub gossipsub_mesh_n: usize,
    pub gossipsub_mesh_n_low: usize,
    pub gossipsub_mesh_n_high: usize,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Maximum number of connected peers
    pub max_peers: u32,

    /// NAT traversal: enable hole punching
    pub enable_hole_punching: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/udp/0/quic-v1".to_string(),
            bootstrap_nodes: vec![],
            enable_mdns: true,
            enable_autonat: true,
            enable_relay_client: true,
            enable_relay_server: false,
            kademlia_replication: 20,
            gossipsub_heartbeat: Duration::from_secs(1),
            gossipsub_mesh_n: 6,
            gossipsub_mesh_n_low: 4,
            gossipsub_mesh_n_high: 12,
            connection_timeout: Duration::from_secs(30),
            max_peers: 50,
            enable_hole_punching: true,
        }
    }
}

impl NetworkConfig {
    /// Create config with bootstrap nodes
    pub fn with_bootstrap(mut self, nodes: Vec<String>) -> Self {
        self.bootstrap_nodes = nodes;
        self
    }

    /// Create config for relay node
    pub fn relay_node() -> Self {
        Self {
            enable_relay_server: true,
            enable_relay_client: false,
            listen_addr: "/ip4/0.0.0.0/udp/9090/quic-v1".to_string(),
            max_peers: 1000,
            ..Default::default()
        }
    }

    /// Create config for client node
    pub fn client_node() -> Self {
        Self {
            enable_relay_client: true,
            enable_relay_server: false,
            ..Default::default()
        }
    }
}
