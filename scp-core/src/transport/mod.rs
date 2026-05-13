//! Transport layer module for SCP Protocol
//!
//! Implements libp2p-based P2P networking with QUIC/TCP transport,
//! Noise encryption, yamux multiplexing, Kademlia DHT discovery,
//! and GossipSub pubsub messaging.
//! Based on SCP Specification section 3.

pub mod network;
pub mod behaviour;
pub mod config;

pub use network::P2PNetwork;
pub use config::NetworkConfig;
pub use behaviour::ScpBehaviour;
