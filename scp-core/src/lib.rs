//! SwarmChat Core Library - SCP Protocol Implementation
//! 
//! This library implements the Swarm Communication Protocol (SCP) v0.1.0.
//! It provides cryptographic primitives, identity management, and network
//! protocols for decentralized communication.

// pub mod crypto;  // Temporarily disabled due to dependency issues
pub mod identity;

/// Error types for the scp-core library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // #[error("Crypto error: {0}")]
    // Crypto(#[from] crypto::CryptoError),
    
    #[error("Identity error: {0}")]
    Identity(#[from] identity::IdentityError),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;