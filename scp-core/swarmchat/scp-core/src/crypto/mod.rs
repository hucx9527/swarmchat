//! Cryptographic primitives for SCP

pub mod x3dh;

// Simple error type for now
#[derive(Debug)]
pub enum CryptoError {
    X3DH(String),
    InvalidKeyLength { expected: usize, actual: usize },
    InvalidParameter(String),
    OperationFailed(String),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::X3DH(msg) => write!(f, "X3DH error: {}", msg),
            CryptoError::InvalidKeyLength { expected, actual } => 
                write!(f, "Invalid key length: expected {}, got {}", expected, actual),
            CryptoError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            CryptoError::OperationFailed(msg) => write!(f, "Crypto operation failed: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}