//! Messaging layer module for SCP Protocol
//!
//! Implements SCP message envelope serialization/deserialization,
//! signature/verification, and the complete message type system.
//! Based on SCP Specification sections 5.1-5.3.

pub mod envelope;
pub mod types;

pub use envelope::Envelope;
pub use types::{MessageType, MessagePayload, PayloadContent};
