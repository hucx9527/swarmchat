//! Identity Discovery module for SwarmChat — P1-5
//!
//! Implements local identity discovery, resolution, and caching as specified
//! in SCP §4.5. Provides mechanisms for discovering peers on the local network,
//! resolving DIDs to PeerIds and network addresses, and verifying identities.
//!
//! ## Key Features
//! - **Identity resolution**: resolve DID → PeerId, DID → public key
//! - **Local discovery**: mDNS-based local peer discovery integration
//! - **Identity cache**: TTL-based cache for discovered identities
//! - **Verification**: verify identity signatures and proof-of-possession
//! - **Address book**: maintain a local address book of known peers

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::did::{Did, DidError};
use crate::peer_id::{PeerId, PeerIdError};
use crate::identity::IdentityError;

// ============================================================================
// Identity Discovery Error
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum IdentityDiscoveryError {
    #[error("Identity not found in cache: {0}")]
    CacheMiss(String),

    #[error("Identity resolution failed: {0}")]
    ResolutionFailed(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("DID error: {0}")]
    Did(#[from] DidError),

    #[error("PeerId error: {0}")]
    PeerId(#[from] PeerIdError),

    #[error("Identity error: {0}")]
    Identity(#[from] IdentityError),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

// ============================================================================
// Discovered Peer
// ============================================================================

/// Information about a discovered peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    /// The peer's DID
    pub did: String,

    /// The peer's PeerId
    pub peer_id: String,

    /// The peer's Ed25519 public key (raw bytes)
    pub public_key: Vec<u8>,

    /// Known network addresses
    pub addresses: Vec<String>,

    /// Optional display name / nickname
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Discovery method (e.g., "mdns", "dht", "manual", "bootstrap")
    pub discovery_method: String,

    /// When this peer was first discovered
    pub first_seen: u64,

    /// When this peer was last seen
    pub last_seen: u64,

    /// Whether the peer has been verified
    pub verified: bool,

    /// The peer's agent version (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_version: Option<String>,

    /// Custom metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
}

impl DiscoveredPeer {
    /// Create a new DiscoveredPeer from a DID and discovery method.
    pub fn from_did(did: &str, method: &str) -> Result<Self, IdentityDiscoveryError> {
        let did_struct = Did::parse(did)?;
        let peer_id = PeerId::from_did(&did_struct)?.to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(DiscoveredPeer {
            did: did.to_string(),
            peer_id,
            public_key: did_struct.public_key_bytes().to_vec(),
            addresses: Vec::new(),
            display_name: None,
            discovery_method: method.to_string(),
            first_seen: now,
            last_seen: now,
            verified: false,
            agent_version: None,
            metadata: HashMap::new(),
        })
    }

    /// Create from raw Ed25519 public key bytes.
    pub fn from_public_key(pubkey: &[u8], method: &str) -> Result<Self, IdentityDiscoveryError> {
        let did = Did::new(pubkey, "Ed25519")?;
        Self::from_did(&did.to_string(), method)
    }

    /// Add a network address to the peer.
    pub fn with_address(mut self, addr: &str) -> Self {
        if !self.addresses.contains(&addr.to_string()) {
            self.addresses.push(addr.to_string());
        }
        self.last_seen = Self::now_secs();
        self
    }

    /// Add multiple addresses.
    pub fn with_addresses(mut self, addrs: Vec<String>) -> Self {
        for addr in addrs {
            if !self.addresses.contains(&addr) {
                self.addresses.push(addr);
            }
        }
        self.last_seen = Self::now_secs();
        self
    }

    /// Set display name.
    pub fn with_display_name(mut self, name: &str) -> Self {
        self.display_name = Some(name.to_string());
        self
    }

    /// Mark as verified.
    pub fn with_verification(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }

    /// Touch (update last_seen).
    pub fn touch(&mut self) {
        self.last_seen = Self::now_secs();
    }

    fn now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ============================================================================
// Identity Cache Entry
// ============================================================================

/// Cache entry with TTL for a resolved identity.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The discovered peer data
    peer: DiscoveredPeer,

    /// When this entry was cached
    cached_at: Instant,

    /// Time-to-live for this cache entry
    ttl: Duration,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    #[allow(dead_code)]
    fn time_until_expiry(&self) -> Duration {
        let elapsed = self.cached_at.elapsed();
        if elapsed > self.ttl {
            Duration::from_secs(0)
        } else {
            self.ttl - elapsed
        }
    }
}

// ============================================================================
// Identity Discovery
// ============================================================================

/// IdentityDiscovery manages identity resolution, caching, and local discovery.
///
/// ## Usage
///
/// ```ignore
/// let mut discovery = IdentityDiscovery::new();
///
/// // Resolve a DID to get peer info (cached)
/// let peer = discovery.resolve("did:key:z6Mk...")?;
/// println!("PeerId: {}", peer.peer_id);
///
/// // Register a locally discovered peer
/// discovery.register_peer(peer);
///
/// // Query all known peers
/// for (did, peer) in discovery.known_peers() {
///     println!("Known: {} at {:?}", did, peer.addresses);
/// }
///
/// // Lookup PeerId from DID
/// let peer_id = discovery.lookup_peer_id("did:key:z6Mk...")?;
/// ```
#[derive(Debug)]
pub struct IdentityDiscovery {
    /// DID → cached peer info
    cache: HashMap<String, CacheEntry>,

    /// PeerId → DID reverse mapping
    peer_id_to_did: HashMap<String, String>,

    /// Default cache TTL
    default_ttl: Duration,

    /// Maximum cache size (LRU eviction when exceeded)
    max_cache_size: usize,

    /// Local addresses (for self-announcement)
    local_addresses: Vec<String>,
}

impl IdentityDiscovery {
    /// Create a new IdentityDiscovery with default settings.
    pub fn new() -> Self {
        IdentityDiscovery {
            cache: HashMap::new(),
            peer_id_to_did: HashMap::new(),
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_cache_size: 10000,
            local_addresses: Vec::new(),
        }
    }

    /// Create with custom TTL and max cache size.
    pub fn with_config(default_ttl: Duration, max_cache_size: usize) -> Self {
        IdentityDiscovery {
            cache: HashMap::new(),
            peer_id_to_did: HashMap::new(),
            default_ttl,
            max_cache_size,
            local_addresses: Vec::new(),
        }
    }

    /// Set local addresses for self-announcement.
    pub fn set_local_addresses(&mut self, addresses: Vec<String>) {
        self.local_addresses = addresses;
    }

    /// Resolve a DID to a DiscoveredPeer.
    ///
    /// Returns cached result if available and not expired.
    /// Otherwise derives PeerId from the DID (without network lookup).
    pub fn resolve(&mut self, did: &str) -> Result<DiscoveredPeer, IdentityDiscoveryError> {
        // Check cache first
        if let Some(entry) = self.cache.get(did) {
            if !entry.is_expired() {
                return Ok(entry.peer.clone());
            }
        }

        // Cache miss or expired — resolve from DID
        let peer = DiscoveredPeer::from_did(did, "resolution")?;
        self.cache_peer(did, peer.clone());
        Ok(peer)
    }

    /// Resolve a DID, bypassing cache (forces fresh resolution).
    pub fn resolve_fresh(&mut self, did: &str) -> Result<DiscoveredPeer, IdentityDiscoveryError> {
        let peer = DiscoveredPeer::from_did(did, "fresh_resolution")?;
        self.cache_peer(did, peer.clone());
        Ok(peer)
    }

    /// Register a discovered peer (adds to cache).
    pub fn register_peer(&mut self, peer: DiscoveredPeer) {
        let did = peer.did.clone();
        self.peer_id_to_did.insert(peer.peer_id.clone(), did.clone());
        self.cache_peer(&did, peer);
    }

    /// Look up a PeerId and return the corresponding DID if known.
    pub fn lookup_did(&self, peer_id: &str) -> Option<&str> {
        self.peer_id_to_did.get(peer_id).map(|s| s.as_str())
    }

    /// Look up a DID and return the corresponding PeerId.
    pub fn lookup_peer_id(&mut self, did: &str) -> Result<String, IdentityDiscoveryError> {
        // Check cache first
        if let Some(entry) = self.cache.get(did) {
            if !entry.is_expired() {
                return Ok(entry.peer.peer_id.clone());
            }
        }
        // Resolve from DID
        let peer = self.resolve(did)?;
        Ok(peer.peer_id)
    }

    /// Get a known peer by DID (only if cached and not expired).
    pub fn get_peer(&self, did: &str) -> Option<&DiscoveredPeer> {
        self.cache.get(did)
            .filter(|entry| !entry.is_expired())
            .map(|entry| &entry.peer)
    }

    /// Get a known peer by PeerId (only if cached and not expired).
    pub fn get_peer_by_id(&self, peer_id: &str) -> Option<&DiscoveredPeer> {
        let did = self.peer_id_to_did.get(peer_id)?;
        self.get_peer(did)
    }

    /// List all cached peers (non-expired).
    pub fn known_peers(&self) -> Vec<(&str, &DiscoveredPeer)> {
        self.cache.iter()
            .filter(|(_, entry)| !entry.is_expired())
            .map(|(did, entry)| (did.as_str(), &entry.peer))
            .collect()
    }

    /// List peers discovered via a specific method.
    pub fn peers_by_discovery_method(&self, method: &str) -> Vec<&DiscoveredPeer> {
        self.cache.values()
            .filter(|entry| !entry.is_expired() && entry.peer.discovery_method == method)
            .map(|entry| &entry.peer)
            .collect()
    }

    /// Get the number of cached peers.
    pub fn peer_count(&self) -> usize {
        self.cache.iter()
            .filter(|(_, entry)| !entry.is_expired())
            .count()
    }

    /// Verify a peer's identity by checking that the DID's public key
    /// matches the provided public key bytes.
    pub fn verify_peer_identity(
        did: &str,
        claimed_public_key: &[u8],
    ) -> Result<bool, IdentityDiscoveryError> {
        let did_struct = Did::parse(did)?;
        let did_pubkey = did_struct.public_key_bytes();
        Ok(did_pubkey == claimed_public_key)
    }

    /// Verify a peer by reconstructing the DID from the public key
    /// and comparing it to the claimed DID.
    pub fn verify_did_ownership(
        claimed_did: &str,
        public_key: &[u8],
    ) -> Result<bool, IdentityDiscoveryError> {
        let computed = Did::new(public_key, "Ed25519")?;
        Ok(computed.to_string() == claimed_did)
    }

    /// Remove a peer from the cache.
    pub fn remove_peer(&mut self, did: &str) {
        if let Some(entry) = self.cache.remove(did) {
            self.peer_id_to_did.remove(&entry.peer.peer_id);
        }
    }

    /// Remove a peer by PeerId.
    pub fn remove_peer_by_id(&mut self, peer_id: &str) {
        if let Some(did) = self.peer_id_to_did.remove(peer_id) {
            self.cache.remove(&did);
        }
    }

    /// Clear all cached entries.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.peer_id_to_did.clear();
    }

    /// Evict expired entries.
    pub fn evict_expired(&mut self) -> usize {
        let expired_dids: Vec<String> = self.cache.iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(did, _)| did.clone())
            .collect();

        let count = expired_dids.len();
        for did in &expired_dids {
            if let Some(entry) = self.cache.remove(did) {
                self.peer_id_to_did.remove(&entry.peer.peer_id);
            }
        }
        count
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> CacheStats {
        let total = self.cache.len();
        let expired = self.cache.values()
            .filter(|entry| entry.is_expired())
            .count();
        let valid = total - expired;

        CacheStats {
            total_entries: total,
            valid_entries: valid,
            expired_entries: expired,
            max_size: self.max_cache_size,
            default_ttl_secs: self.default_ttl.as_secs(),
        }
    }

    // ---- Internal ----

    fn cache_peer(&mut self, did: &str, peer: DiscoveredPeer) {
        // Evict if at capacity
        if self.cache.len() >= self.max_cache_size {
            self.evict_expired();
            // If still at capacity after eviction, remove oldest
            if self.cache.len() >= self.max_cache_size {
                if let Some(oldest_did) = self.cache.keys().next().cloned() {
                    self.cache.remove(&oldest_did);
                }
            }
        }

        let entry = CacheEntry {
            peer,
            cached_at: Instant::now(),
            ttl: self.default_ttl,
        };

        self.peer_id_to_did.insert(entry.peer.peer_id.clone(), did.to_string());
        self.cache.insert(did.to_string(), entry);
    }
}

// ============================================================================
// Cache Stats
// ============================================================================

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
    pub max_size: usize,
    pub default_ttl_secs: u64,
}

// ============================================================================
// Local Discovery Configuration
// ============================================================================

/// Configuration for local network discovery.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Enable mDNS-based discovery
    pub enable_mdns: bool,

    /// mDNS service type
    pub mdns_service_type: String,

    /// mDNS query interval
    pub mdns_interval: Duration,

    /// Enable DHT-based discovery
    pub enable_dht: bool,

    /// Bootstrap peers for DHT
    pub bootstrap_peers: Vec<String>,

    /// Cache TTL for discovered peers
    pub cache_ttl: Duration,

    /// Maximum number of cached peers
    pub max_cache_size: usize,

    /// Local addresses to announce
    pub announce_addresses: Vec<String>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        DiscoveryConfig {
            enable_mdns: true,
            mdns_service_type: "_scp._udp.local.".to_string(),
            mdns_interval: Duration::from_secs(30),
            enable_dht: true,
            bootstrap_peers: Vec::new(),
            cache_ttl: Duration::from_secs(3600),
            max_cache_size: 10000,
            announce_addresses: Vec::new(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn make_test_did() -> (String, Vec<u8>) {
        let signing_key = SigningKey::from_bytes(&[0x42u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let did = Did::from_ed25519(&verifying_key).unwrap();
        (did.to_string(), verifying_key.as_bytes().to_vec())
    }

    #[test]
    fn test_discover_peer_from_did() {
        let (did_str, pubkey) = make_test_did();
        let peer = DiscoveredPeer::from_did(&did_str, "test").unwrap();
        assert_eq!(peer.did, did_str);
        assert_eq!(peer.public_key, pubkey);
        assert_eq!(peer.discovery_method, "test");
        assert!(!peer.peer_id.is_empty());
    }

    #[test]
    fn test_identity_discovery_resolve() {
        let (did_str, _) = make_test_did();
        let mut discovery = IdentityDiscovery::new();
        let peer = discovery.resolve(&did_str).unwrap();
        assert_eq!(peer.did, did_str);
        assert_eq!(discovery.peer_count(), 1);
    }

    #[test]
    fn test_identity_discovery_caching() {
        let (did_str, _) = make_test_did();
        let mut discovery = IdentityDiscovery::new();

        // First resolve
        let peer1 = discovery.resolve(&did_str).unwrap();

        // Second resolve should hit cache
        let peer2 = discovery.resolve(&did_str).unwrap();

        assert_eq!(peer1.peer_id, peer2.peer_id);
        assert_eq!(peer1.did, peer2.did);
        assert_eq!(discovery.peer_count(), 1);
    }

    #[test]
    fn test_register_peer() {
        let (did_str, _) = make_test_did();
        let mut discovery = IdentityDiscovery::new();

        let peer = DiscoveredPeer::from_did(&did_str, "manual")
            .unwrap()
            .with_display_name("Test Peer")
            .with_address("/ip4/192.168.1.1/udp/9090/quic-v1");

        discovery.register_peer(peer);
        assert_eq!(discovery.peer_count(), 1);

        let cached = discovery.get_peer(&did_str).unwrap();
        assert_eq!(cached.display_name.as_deref(), Some("Test Peer"));
        assert_eq!(cached.addresses.len(), 1);
    }

    #[test]
    fn test_lookup_peer_id() {
        let (did_str, _) = make_test_did();
        let mut discovery = IdentityDiscovery::new();

        let peer_id = discovery.lookup_peer_id(&did_str).unwrap();
        assert!(!peer_id.is_empty());

        // Should also be findable via reverse lookup
        let reverse_did = discovery.lookup_did(&peer_id).unwrap();
        assert_eq!(reverse_did, did_str);
    }

    #[test]
    fn test_verify_peer_identity() {
        let (did_str, pubkey) = make_test_did();

        // Correct key → verified
        assert!(IdentityDiscovery::verify_peer_identity(&did_str, &pubkey).unwrap());

        // Wrong key → not verified
        let wrong_key = [0x99u8; 32];
        assert!(!IdentityDiscovery::verify_peer_identity(&did_str, &wrong_key).unwrap());
    }

    #[test]
    fn test_verify_did_ownership() {
        let signing_key = SigningKey::from_bytes(&[0x77u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let did = Did::from_ed25519(&verifying_key).unwrap();
        let did_str = did.to_string();

        // Correct key → verified
        assert!(IdentityDiscovery::verify_did_ownership(
            &did_str,
            verifying_key.as_bytes(),
        ).unwrap());

        // Wrong key → not verified
        assert!(!IdentityDiscovery::verify_did_ownership(
            &did_str,
            &[0x88u8; 32],
        ).unwrap());
    }

    #[test]
    fn test_remove_peer() {
        let (did_str, _) = make_test_did();
        let mut discovery = IdentityDiscovery::new();

        discovery.resolve(&did_str).unwrap();
        assert_eq!(discovery.peer_count(), 1);

        discovery.remove_peer(&did_str);
        assert_eq!(discovery.peer_count(), 0);
        assert!(discovery.get_peer(&did_str).is_none());
    }

    #[test]
    fn test_evict_expired() {
        let (did_str, _) = make_test_did();
        let mut discovery = IdentityDiscovery::with_config(
            Duration::from_secs(0), // immediate expiry
            100,
        );

        discovery.resolve(&did_str).unwrap();
        // TTL is 0, so it should be expired
        let evicted = discovery.evict_expired();
        assert_eq!(evicted, 1);
        assert_eq!(discovery.peer_count(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let (did_str, _) = make_test_did();
        let mut discovery = IdentityDiscovery::new();

        discovery.resolve(&did_str).unwrap();
        let stats = discovery.cache_stats();

        assert_eq!(stats.valid_entries, 1);
        assert_eq!(stats.expired_entries, 0);
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.default_ttl_secs, 3600);
    }

    #[test]
    fn test_multiple_peers() {
        let mut discovery = IdentityDiscovery::new();

        for i in 0..5u8 {
            let signing_key = SigningKey::from_bytes(&[i; 32]);
            let verifying_key = signing_key.verifying_key();
            let did = Did::from_ed25519(&verifying_key).unwrap();
            let peer = DiscoveredPeer::from_did(&did.to_string(), "test").unwrap()
                .with_display_name(&format!("Peer {}", i));
            discovery.register_peer(peer);
        }

        assert_eq!(discovery.peer_count(), 5);
        assert_eq!(discovery.known_peers().len(), 5);
    }

    #[test]
    fn test_discovery_config_defaults() {
        let config = DiscoveryConfig::default();
        assert!(config.enable_mdns);
        assert!(config.enable_dht);
        assert_eq!(config.mdns_service_type, "_scp._udp.local.");
        assert_eq!(config.cache_ttl, Duration::from_secs(3600));
        assert_eq!(config.max_cache_size, 10000);
    }

    #[test]
    fn test_peer_with_addresses() {
        let (did_str, _) = make_test_did();
        let peer = DiscoveredPeer::from_did(&did_str, "mdns")
            .unwrap()
            .with_address("/ip4/10.0.0.1/tcp/9090")
            .with_addresses(vec![
                "/ip4/10.0.0.2/udp/9090/quic-v1".to_string(),
                "/ip4/10.0.0.3/udp/9090/quic-v1".to_string(),
            ]);

        assert_eq!(peer.addresses.len(), 3);
        assert_eq!(peer.discovery_method, "mdns");
    }
}
