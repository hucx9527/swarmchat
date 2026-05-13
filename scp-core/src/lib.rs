//! SwarmChat Core Library - SCP Protocol Implementation
//!
//! This library implements the Swarm Communication Protocol (SCP) v0.1.0.
//! It provides cryptographic primitives, identity management, and network
//! protocols for decentralized communication.
//!
//! ## Architecture
//!
//! ```text
//! scp-core
//! ├── crypto/         — X3DH, Double Ratchet, Sender Key encryption
//! ├── identity/       — BIP39 mnemonic, seed derivation, Ed25519/X25519 keys
//! ├── did/            — W3C did:key method, DID document generation
//! ├── peer_id/        — SHA-256 multihash-based PeerId for P2P networking
//! ├── identity_manager/ — Multi-identity management (P1-4)
//! ├── identity_discovery/ — Local identity discovery & resolution (P1-5)
//! ├── transport/      — libp2p P2P network (QUIC, Kademlia DHT, GossipSub)
//! └── messaging/      — SCP message envelope & type system
//! ```

pub mod crypto;
pub mod identity;
pub mod did;
pub mod peer_id;
pub mod identity_manager;
pub mod identity_discovery;
pub mod transport;
pub mod messaging;

/// Error types for the scp-core library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Identity error: {0}")]
    Identity(#[from] identity::IdentityError),

    #[error("DID error: {0}")]
    Did(#[from] did::DidError),

    #[error("Identity manager error: {0}")]
    IdentityManager(#[from] identity_manager::IdentityManagerError),

    #[error("Identity discovery error: {0}")]
    IdentityDiscovery(#[from] identity_discovery::IdentityDiscoveryError),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;