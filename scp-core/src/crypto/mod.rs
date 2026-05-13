//! Cryptographic primitives for SCP (Swarm Communication Protocol)
//!
//! This module contains the core cryptographic building blocks:
//! - `x3dh` — Extended Triple Diffie-Hellman key agreement (§4.5.1)
//! - `double_ratchet` — Double Ratchet algorithm for 1:1 messaging (§4.5.1)
//! - `sender_key` — Sender Key protocol for group messaging (§4.5.2)

pub mod x3dh;
pub mod double_ratchet;
pub mod sender_key;

/// Top-level crypto error type that wraps errors from all sub-modules
#[derive(Debug)]
pub enum CryptoError {
    X3DH(String),
    DoubleRatchet(String),
    SenderKey(String),
    InvalidKeyLength { expected: usize, actual: usize },
    InvalidParameter(String),
    OperationFailed(String),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::X3DH(msg) => write!(f, "X3DH error: {}", msg),
            CryptoError::DoubleRatchet(msg) => write!(f, "Double Ratchet error: {}", msg),
            CryptoError::SenderKey(msg) => write!(f, "Sender Key error: {}", msg),
            CryptoError::InvalidKeyLength { expected, actual } =>
                write!(f, "Invalid key length: expected {}, got {}", expected, actual),
            CryptoError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            CryptoError::OperationFailed(msg) => write!(f, "Crypto operation failed: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

// Re-export key types for convenience
pub use x3dh::{perform_x3dh_initiator, perform_x3dh_responder, X3DHResult, X3DHError};
pub use double_ratchet::{DoubleRatchetState, MessageHeader, generate_dh_ratchet_key, DoubleRatchetError};
pub use sender_key::{SenderKeyState, SenderKeyMessageHeader, SenderKeyDistributionMessage, SenderKeyError};