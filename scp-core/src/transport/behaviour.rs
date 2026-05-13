use libp2p::{
    autonat, gossipsub, identify,
    kad,
    mdns, ping, relay,
    swarm::NetworkBehaviour,
};

/// Composed network behaviour for SCP nodes
///
/// Aggregates all libp2p protocols required by SCP spec section 3.2:
/// - Kademlia DHT for peer discovery
/// - mDNS for LAN discovery
/// - GossipSub for group messaging
/// - Identify for peer info exchange
/// - Ping for keepalive
/// - AutoNAT for NAT status detection
/// - Relay for NAT traversal
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ScpBehaviourEvent")]
pub struct ScpBehaviour {
    /// Kademlia DHT for peer and content discovery
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,

    /// mDNS for local network peer discovery
    pub mdns: mdns::tokio::Behaviour,

    /// GossipSub for pubsub messaging (group chat)
    pub gossipsub: gossipsub::Behaviour,

    /// Identify protocol for peer metadata exchange
    pub identify: identify::Behaviour,

    /// Ping for connection keepalive
    pub ping: ping::Behaviour,

    /// AutoNAT for NAT traversal status detection
    pub autonat: autonat::Behaviour,

    /// Circuit relay v2 for NAT traversal
    pub relay: relay::Behaviour,
}

/// Custom event enum for SCP behaviour events
#[derive(Debug)]
pub enum ScpBehaviourEvent {
    Kademlia(kad::Event),
    Mdns(mdns::Event),
    Gossipsub(gossipsub::Event),
    Identify(identify::Event),
    Ping(ping::Event),
    Autonat(autonat::Event),
    Relay(relay::Event),
}

// Implement From for each event type
impl From<kad::Event> for ScpBehaviourEvent {
    fn from(e: kad::Event) -> Self { ScpBehaviourEvent::Kademlia(e) }
}
impl From<mdns::Event> for ScpBehaviourEvent {
    fn from(e: mdns::Event) -> Self { ScpBehaviourEvent::Mdns(e) }
}
impl From<gossipsub::Event> for ScpBehaviourEvent {
    fn from(e: gossipsub::Event) -> Self { ScpBehaviourEvent::Gossipsub(e) }
}
impl From<identify::Event> for ScpBehaviourEvent {
    fn from(e: identify::Event) -> Self { ScpBehaviourEvent::Identify(e) }
}
impl From<ping::Event> for ScpBehaviourEvent {
    fn from(e: ping::Event) -> Self { ScpBehaviourEvent::Ping(e) }
}
impl From<autonat::Event> for ScpBehaviourEvent {
    fn from(e: autonat::Event) -> Self { ScpBehaviourEvent::Autonat(e) }
}
impl From<relay::Event> for ScpBehaviourEvent {
    fn from(e: relay::Event) -> Self { ScpBehaviourEvent::Relay(e) }
}
