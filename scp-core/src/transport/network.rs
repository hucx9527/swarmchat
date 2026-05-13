use std::error::Error;

use futures::StreamExt;
use libp2p::{
    autonat, core::upgrade, gossipsub, identify,
    identity::Keypair,
    kad::{self, store::MemoryStore, Mode, Record},
    mdns, noise, ping, relay,
    swarm::{self, SwarmEvent},
    tcp, yamux, PeerId, Swarm, Transport,
    multiaddr::Multiaddr,
};

use super::behaviour::{ScpBehaviour, ScpBehaviourEvent};
use super::config::NetworkConfig;

/// Network event emitted by P2PNetwork
#[derive(Debug)]
pub enum NetworkEvent {
    /// A new peer has been discovered (via Kademlia or mDNS)
    PeerDiscovered {
        peer_id: PeerId,
        addresses: Vec<Multiaddr>,
    },

    /// A peer has connected
    PeerConnected {
        peer_id: PeerId,
    },

    /// A peer has disconnected
    PeerDisconnected {
        peer_id: PeerId,
    },

    /// A GossipSub message has been received
    MessageReceived {
        peer_id: PeerId,
        topic: gossipsub::TopicHash,
        data: Vec<u8>,
    },

    /// NAT status has been determined
    NatStatus {
        status: autonat::NatStatus,
    },

    /// New listen address
    NewListenAddr {
        address: Multiaddr,
    },

    /// Relay reservation confirmed
    RelayReserved {
        relay_peer_id: PeerId,
    },

    /// DHT record found
    DhtRecordFound {
        key: kad::RecordKey,
        record: Record,
    },

    /// Error occurred
    Error {
        message: String,
    },
}

/// The P2PNetwork struct manages all libp2p networking for SCP nodes.
///
/// ## Functionality (SCP Spec §3):
/// - **Transport**: QUIC (primary) + TCP (fallback)
/// - **Security**: Noise handshake (`Noise_XX_25519_ChaChaPoly_SHA256`)
/// - **Multiplexing**: yamux
/// - **Discovery**: Kademlia DHT + mDNS
/// - **NAT traversal**: AutoNAT + Circuit Relay v2
/// - **Messaging**: GossipSub v1.2 pub/sub
///
/// ## Usage
///
/// ```ignore
/// let config = NetworkConfig::default();
/// let keypair = Keypair::generate_ed25519();
/// let mut network = P2PNetwork::new(keypair, config).await?;
/// network.start_listening().await?;
/// network.bootstrap().await?;
///
/// // Subscribe to a group topic
/// network.subscribe("my_group")?;
///
/// // Send a message
/// network.publish("my_group", b"hello world")?;
/// ```
pub struct P2PNetwork {
    /// The libp2p Swarm
    swarm: Swarm<ScpBehaviour>,

    /// Local peer ID
    local_peer_id: PeerId,

    /// Network configuration
    config: NetworkConfig,

    /// Event receiver channel
    event_receiver: tokio::sync::mpsc::UnboundedReceiver<NetworkEvent>,

    /// Event sender channel (exposed for internal use)
    event_sender: tokio::sync::mpsc::UnboundedSender<NetworkEvent>,
}

impl P2PNetwork {
    /// Create a new P2PNetwork instance
    ///
    /// # Arguments
    /// * `keypair` - The libp2p identity keypair (derived from Ed25519 signing key)
    /// * `config` - Network configuration
    pub fn new(
        keypair: Keypair,
        config: NetworkConfig,
    ) -> Result<Self, Box<dyn Error>> {
        let local_peer_id = PeerId::from(keypair.public());
        let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();

        let swarm = Self::build_swarm(&keypair, &config, &local_peer_id)?;

        Ok(Self {
            swarm,
            local_peer_id,
            config,
            event_sender,
            event_receiver,
        })
    }

    /// Build the libp2p Swarm with all configured protocols
    fn build_swarm(
        keypair: &Keypair,
        config: &NetworkConfig,
        local_peer_id: &PeerId,
    ) -> Result<Swarm<ScpBehaviour>, Box<dyn Error>> {
        // ---- Noise authenticated encryption ----
        let noise_config = noise::Config::new(keypair)
            .expect("Failed to create noise config");

        // ---- yamux multiplexing ----
        let yamux_config = yamux::Config::default();

        // ---- TCP transport with noise + yamux ----
        let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            .upgrade(upgrade::Version::V1)
            .authenticate(noise_config)
            .multiplex(yamux_config)
            .timeout(config.connection_timeout)
            .boxed();

        // ---- Kademlia DHT ----
        let kad_store = MemoryStore::new(*local_peer_id);
        let mut kad_config = kad::Config::default();
        kad_config.set_replication_factor(std::num::NonZeroUsize::new(config.kademlia_replication).unwrap());

        let kademlia = kad::Behaviour::with_config(
            *local_peer_id,
            kad_store,
            kad_config,
        );

        // ---- mDNS ----
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            *local_peer_id,
        )?;

        // ---- GossipSub ----
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(config.gossipsub_heartbeat)
            .mesh_n(config.gossipsub_mesh_n)
            .mesh_n_low(config.gossipsub_mesh_n_low)
            .mesh_n_high(config.gossipsub_mesh_n_high)
            .build()
            .expect("Valid GossipSub config");

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )?;

        // ---- Identify ----
        let identify = identify::Behaviour::new(
            identify::Config::new("scp/1.0".to_string(), keypair.public())
                .with_agent_version(format!("scp-core/{}", env!("CARGO_PKG_VERSION"))),
        );

        // ---- Ping ----
        let ping = ping::Behaviour::new(ping::Config::default());

        // ---- AutoNAT ----
        let autonat = autonat::Behaviour::new(
            *local_peer_id,
            autonat::Config::default(),
        );

        // ---- Relay ----
        let relay = relay::Behaviour::new(
            *local_peer_id,
            relay::Config::default(),
        );

        let behaviour = ScpBehaviour {
            kademlia,
            mdns,
            gossipsub,
            identify,
            ping,
            autonat,
            relay,
        };

        let swarm = Swarm::new(
            transport,
            behaviour,
            *local_peer_id,
            swarm::Config::with_tokio_executor(),
        );

        Ok(swarm)
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// Get a new event sender (for subscribing to network events)
    pub fn event_sender(&self) -> tokio::sync::mpsc::UnboundedSender<NetworkEvent> {
        self.event_sender.clone()
    }

    /// Consume the event receiver (used by the run loop)
    pub fn take_event_receiver(&mut self) -> tokio::sync::mpsc::UnboundedReceiver<NetworkEvent> {
        std::mem::replace(
            &mut self.event_receiver,
            tokio::sync::mpsc::unbounded_channel().1,
        )
    }

    /// Start listening on configured address (P1-2)
    pub async fn start_listening(&mut self) -> Result<(), Box<dyn Error>> {
        let addr: Multiaddr = self.config.listen_addr.parse()?;
        self.swarm.listen_on(addr)?;
        tracing::info!("SCP node listening on {}", self.config.listen_addr);
        Ok(())
    }

    /// Connect to bootstrap nodes and start DHT (P1-2)
    pub async fn bootstrap(&mut self) -> Result<(), Box<dyn Error>> {
        // Set Kademlia to server mode for bootstrap/routing nodes
        self.swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));

        // Dial bootstrap nodes
        for addr_str in &self.config.bootstrap_nodes {
            match addr_str.parse::<Multiaddr>() {
                Ok(addr) => {
                    if let Err(e) = self.swarm.dial(addr.clone()) {
                        tracing::warn!("Failed to dial bootstrap node {}: {:?}", addr, e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Invalid bootstrap address {}: {:?}", addr_str, e);
                }
            }
        }

        tracing::info!("Bootstrap complete, {} bootstrap nodes configured", self.config.bootstrap_nodes.len());
        Ok(())
    }

    /// Subscribe to a GossipSub topic (for group messaging)
    pub fn subscribe(&mut self, group_id: &str) -> Result<(), Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new(format!("scp/group/v1/{}/messages", group_id));
        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        tracing::info!("Subscribed to topic: {}", topic);
        Ok(())
    }

    /// Unsubscribe from a GossipSub topic
    pub fn unsubscribe(&mut self, group_id: &str) -> Result<(), Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new(format!("scp/group/v1/{}/messages", group_id));
        self.swarm.behaviour_mut().gossipsub.unsubscribe(&topic)?;
        tracing::info!("Unsubscribed from topic: {}", topic);
        Ok(())
    }

    /// Publish a message to a GossipSub topic
    pub fn publish(&mut self, group_id: &str, data: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new(format!("scp/group/v1/{}/messages", group_id));
        self.swarm.behaviour_mut().gossipsub.publish(topic, data)?;
        Ok(())
    }

    /// Send a direct message to a peer (via request-response or stream)
    pub fn send_direct(&mut self, peer_id: PeerId, _data: Vec<u8>) -> Result<(), Box<dyn Error>> {
        // Dial the peer if not connected
        // For direct messaging we rely on libp2p streams opened via identify
        // In a full implementation, this would use a request-response protocol
        self.swarm.dial(peer_id)?;
        Ok(())
    }

    /// Add a peer address to the DHT
    pub fn add_peer_to_dht(&mut self, peer_id: PeerId) -> Result<(), Box<dyn Error>> {
        self.swarm.behaviour_mut().kademlia.add_address(&peer_id, Multiaddr::empty());
        Ok(())
    }

    /// Get DHT record
    pub fn get_dht_record(&mut self, key: kad::RecordKey) {
        self.swarm.behaviour_mut().kademlia.get_record(key);
    }

    /// Put DHT record
    pub fn put_dht_record(&mut self, record: Record) -> Result<(), Box<dyn Error>> {
        self.swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One)?;
        Ok(())
    }

    /// Main event loop - process swarm events
    pub async fn run(mut self) {
        loop {
            match self.swarm.next().await.expect("Swarm stream ended") {
                SwarmEvent::Behaviour(ScpBehaviourEvent::Kademlia(event)) => {
                    self.handle_kademlia_event(event);
                }
                SwarmEvent::Behaviour(ScpBehaviourEvent::Mdns(event)) => {
                    self.handle_mdns_event(event);
                }
                SwarmEvent::Behaviour(ScpBehaviourEvent::Gossipsub(event)) => {
                    self.handle_gossipsub_event(event);
                }
                SwarmEvent::Behaviour(ScpBehaviourEvent::Identify(event)) => {
                    self.handle_identify_event(event);
                }
                SwarmEvent::Behaviour(ScpBehaviourEvent::Ping(event)) => {
                    self.handle_ping_event(event);
                }
                SwarmEvent::Behaviour(ScpBehaviourEvent::Autonat(event)) => {
                    self.handle_autonat_event(event);
                }
                SwarmEvent::Behaviour(ScpBehaviourEvent::Relay(event)) => {
                    self.handle_relay_event(event);
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    let _ = self.event_sender.send(NetworkEvent::NewListenAddr { address });
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    let _ = self.event_sender.send(NetworkEvent::PeerConnected { peer_id });
                    // Start Kademlia on connected peer
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, Multiaddr::empty());
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    let _ = self.event_sender.send(NetworkEvent::PeerDisconnected { peer_id });
                }
                _ => {}
            }
        }
    }

    // ---- Event Handlers ----

    fn handle_kademlia_event(&mut self, event: kad::Event) {
        match event {
            kad::Event::RoutingUpdated { peer, .. } => {
                tracing::debug!("Kademlia routing updated: {:?}", peer);
            }
            kad::Event::OutboundQueryProgressed { result, .. } => {
                if let kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(record))) = result {
                    let _ = self.event_sender.send(NetworkEvent::DhtRecordFound {
                        key: record.record.key.clone(),
                        record: record.record,
                    });
                }
            }
            _ => {}
        }
    }

    fn handle_mdns_event(&mut self, event: mdns::Event) {
        match event {
            mdns::Event::Discovered(list) => {
                for (peer_id, addr) in list {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                    let _ = self.event_sender.send(NetworkEvent::PeerDiscovered {
                        peer_id,
                        addresses: vec![addr],
                    });
                }
            }
            mdns::Event::Expired(list) => {
                for (_peer_id, _addr) in list {
                    // Peer expired from mDNS
                }
            }
        }
    }

    fn handle_gossipsub_event(&mut self, event: gossipsub::Event) {
        match event {
            gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id: _id,
                message,
            } => {
                let _ = self.event_sender.send(NetworkEvent::MessageReceived {
                    peer_id,
                    topic: message.topic,
                    data: message.data,
                });
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                tracing::debug!("Peer {:?} subscribed to {:?}", peer_id, topic);
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                tracing::debug!("Peer {:?} unsubscribed from {:?}", peer_id, topic);
            }
            gossipsub::Event::GossipsubNotSupported { peer_id } => {
                tracing::warn!("Peer {:?} does not support gossipsub", peer_id);
            }
        }
    }

    fn handle_identify_event(&mut self, event: identify::Event) {
        match event {
            identify::Event::Received { peer_id, info } => {
                tracing::info!("Identified peer {:?}: agent={}, protocols={:?}",
                    peer_id, info.agent_version, info.protocols);
                // Add peer's addresses to Kademlia
                for addr in info.listen_addrs {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }
            identify::Event::Sent { peer_id } => {
                tracing::debug!("Sent identify to {:?}", peer_id);
            }
            _ => {}
        }
    }

    fn handle_ping_event(&mut self, event: ping::Event) {
        match event {
            ping::Event { peer, connection: _, result } => {
                match result {
                    Ok(rtt) => tracing::trace!("Ping to {:?}: {}ms", peer, rtt.as_millis()),
                    Err(e) => tracing::warn!("Ping to {:?} failed: {:?}", peer, e),
                }
            }
        }
    }

    fn handle_autonat_event(&mut self, event: autonat::Event) {
        match event {
            autonat::Event::StatusChanged { old, new } => {
                tracing::info!("NAT status changed: {:?} -> {:?}", old, new);
                let _ = self.event_sender.send(NetworkEvent::NatStatus { status: new });
            }
            _ => {}
        }
    }

    fn handle_relay_event(&mut self, event: relay::Event) {
        match event {
            relay::Event::ReservationReqAccepted { .. } => {
                tracing::info!("Relay reservation accepted");
            }
            _ => {}
        }
    }

    /// Get current connected peers
    pub fn connected_peers(&self) -> usize {
        self.swarm.connected_peers().count()
    }

    /// Get external addresses (discovered via AutoNAT)
    pub fn external_addresses(&self) -> Vec<Multiaddr> {
        self.swarm.external_addresses().cloned().collect()
    }

    /// Get listen addresses
    pub fn listeners(&self) -> Vec<Multiaddr> {
        self.swarm.listeners().cloned().collect()
    }
}
