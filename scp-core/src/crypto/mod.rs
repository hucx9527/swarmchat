//! Cryptographic primitives for SCP

pub mod x3dh;
pub mod double_ratchet;
pub mod sender_key;

// Simple error type for now
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