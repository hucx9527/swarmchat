//! Swarm Communication Protocol core library

pub mod crypto;

/// Simple error type
#[derive(Debug)]
pub enum Error {
    Crypto(String),
    InvalidParameter(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Crypto(msg) => write!(f, "Crypto error: {}", msg),
            Error::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}