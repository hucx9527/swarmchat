//! Identity Manager module for SwarmChat — P1-4
//!
//! Implements multi-identity management as specified in SCP §4.4.
//! Provides identity store, CRUD operations, default identity selection,
//! identity labels, metadata, and file-based persistence.
//!
//! ## Key Features
//! - **Multi-identity storage**: manage multiple identities in a single store
//! - **Identity switching**: set active/default identity
//! - **Metadata management**: labels, nicknames, timestamps per identity
//! - **File persistence**: save/load identity store to/from JSON files
//! - **Import/export**: import identities from mnemonic phrases or seed bytes

use std::collections::HashMap;
use std::path::Path;
use std::fs;

use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::identity::{Identity, IdentityError};
use crate::did::{Did, DidError};
use crate::peer_id::{PeerId, PeerIdError};

/// Error types for identity manager operations
#[derive(Debug, thiserror::Error)]
pub enum IdentityManagerError {
    #[error("Identity not found: {0}")]
    NotFound(String),
    #[error("Identity already exists: {0}")]
    AlreadyExists(String),
    #[error("No default identity configured")]
    NoDefaultIdentity,
    #[error("No identities in store")]
    EmptyStore,
    #[error("Invalid identity label: {0}")]
    InvalidLabel(String),
    #[error("Identity error: {0}")]
    Identity(#[from] IdentityError),
    #[error("DID error: {0}")]
    Did(#[from] DidError),
    #[error("PeerId error: {0}")]
    PeerId(#[from] PeerIdError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

// ============================================================================
// Identity Entry
// ============================================================================

/// Metadata wrapper around an Identity.
///
/// Each stored identity carries: a unique label, optional nickname,
/// timestamps, whether it is the default identity, and optional description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityEntry {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub identity: Identity,
    pub did: String,
    pub peer_id: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub is_default: bool,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
}

impl IdentityEntry {
    /// Create a new IdentityEntry from an Identity with a label.
    /// Automatically derives DID and PeerId from the identity's public key.
    pub fn new(label: &str, mut identity: Identity) -> Result<Self, IdentityManagerError> {
        if label.is_empty() || label.len() > 128 {
            return Err(IdentityManagerError::InvalidLabel(
                format!("Label must be 1-128 characters, got '{}'", label)
            ));
        }
        if !label.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(IdentityManagerError::InvalidLabel(
                format!("Label '{}' contains invalid characters", label)
            ));
        }
        let keys = identity.derive_keys()?;
        let did = Did::from_ed25519(&keys.verifying_key)?.to_string();
        let did_struct = Did::from_ed25519(&keys.verifying_key)?;
        let peer_id = PeerId::from_did(&did_struct)?.to_string();
        let now = Utc::now().to_rfc3339();
        Ok(IdentityEntry {
            label: label.to_string(),
            nickname: None,
            description: None,
            identity,
            did,
            peer_id,
            created_at: now.clone(),
            updated_at: now,
            is_default: false,
            metadata: HashMap::new(),
        })
    }

    pub fn with_nickname(mut self, nickname: &str) -> Self {
        self.nickname = Some(nickname.to_string());
        self.updated_at = Utc::now().to_rfc3339();
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self.updated_at = Utc::now().to_rfc3339();
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now().to_rfc3339();
        self
    }

    pub fn mnemonic_phrase(&self) -> String {
        self.identity.mnemonic_phrase()
    }

    pub fn public_signing_key(&mut self) -> Result<Vec<u8>, IdentityManagerError> {
        let keys = self.identity.derive_keys()?;
        Ok(keys.verifying_key.as_bytes().to_vec())
    }

    pub fn public_encryption_key(&mut self) -> Result<Vec<u8>, IdentityManagerError> {
        let keys = self.identity.derive_keys()?;
        Ok(keys.encryption_public.as_bytes().to_vec())
    }
}

// ============================================================================
// Identity Store Data (for serialization)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityStoreData {
    pub identities: HashMap<String, IdentityEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_label: Option<String>,
    pub schema_version: u32,
}

// ============================================================================
// Identity Manager
// ============================================================================

#[derive(Debug, Clone)]
pub struct IdentityManager {
    identities: HashMap<String, IdentityEntry>,
    default_label: Option<String>,
}

impl IdentityManager {
    pub fn new() -> Self {
        IdentityManager {
            identities: HashMap::new(),
            default_label: None,
        }
    }

    pub fn create_identity(
        &mut self,
        label: &str,
        nickname: Option<&str>,
        description: Option<&str>,
    ) -> Result<IdentityEntry, IdentityManagerError> {
        if self.identities.contains_key(label) {
            return Err(IdentityManagerError::AlreadyExists(label.to_string()));
        }
        let identity = Identity::new()?;
        let mut entry = IdentityEntry::new(label, identity)?;
        if let Some(nick) = nickname {
            entry = entry.with_nickname(nick);
        }
        if let Some(desc) = description {
            entry = entry.with_description(desc);
        }
        if self.identities.is_empty() {
            entry.is_default = true;
            self.default_label = Some(label.to_string());
        }
        self.identities.insert(label.to_string(), entry.clone());
        Ok(entry)
    }

    pub fn import_identity(
        &mut self,
        label: &str,
        mnemonic_phrase: &str,
        nickname: Option<&str>,
        set_default: bool,
    ) -> Result<IdentityEntry, IdentityManagerError> {
        if self.identities.contains_key(label) {
            return Err(IdentityManagerError::AlreadyExists(label.to_string()));
        }
        use bip39::Mnemonic;
        let mnemonic = Mnemonic::parse(mnemonic_phrase)
            .map_err(|e| IdentityManagerError::Identity(
                IdentityError::MnemonicGeneration(e.to_string())
            ))?;
        let identity = Identity::from_mnemonic(&mnemonic)?;
        let mut entry = IdentityEntry::new(label, identity)?;
        if let Some(nick) = nickname {
            entry = entry.with_nickname(nick);
        }
        if set_default || self.identities.is_empty() {
            entry.is_default = true;
            self.default_label = Some(label.to_string());
            for (_, e) in self.identities.iter_mut() {
                e.is_default = false;
            }
        }
        self.identities.insert(label.to_string(), entry.clone());
        Ok(entry)
    }

    pub fn remove_identity(&mut self, label: &str) -> Result<(), IdentityManagerError> {
        if !self.identities.contains_key(label) {
            return Err(IdentityManagerError::NotFound(label.to_string()));
        }
        self.identities.remove(label);
        if self.default_label.as_deref() == Some(label) {
            self.default_label = None;
            if let Some(first_label) = self.identities.keys().next().cloned() {
                self.default_label = Some(first_label.clone());
                if let Some(entry) = self.identities.get_mut(&first_label) {
                    entry.is_default = true;
                }
            }
        }
        Ok(())
    }

    pub fn set_default(&mut self, label: &str) -> Result<(), IdentityManagerError> {
        if !self.identities.contains_key(label) {
            return Err(IdentityManagerError::NotFound(label.to_string()));
        }
        for (_, entry) in self.identities.iter_mut() {
            entry.is_default = false;
        }
        if let Some(entry) = self.identities.get_mut(label) {
            entry.is_default = true;
        }
        self.default_label = Some(label.to_string());
        Ok(())
    }

    pub fn get_active(&self) -> Result<&IdentityEntry, IdentityManagerError> {
        match &self.default_label {
            Some(label) => self.identities.get(label)
                .ok_or_else(|| IdentityManagerError::NotFound(label.clone())),
            None => self.identities.values().next()
                .ok_or(IdentityManagerError::EmptyStore),
        }
    }

    pub fn get_active_mut(&mut self) -> Result<&mut IdentityEntry, IdentityManagerError> {
        let label = self.default_label.clone()
            .or_else(|| self.identities.keys().next().cloned())
            .ok_or(IdentityManagerError::EmptyStore)?;
        self.identities.get_mut(&label)
            .ok_or_else(|| IdentityManagerError::NotFound(label))
    }

    pub fn get(&self, label: &str) -> Result<&IdentityEntry, IdentityManagerError> {
        self.identities.get(label)
            .ok_or_else(|| IdentityManagerError::NotFound(label.to_string()))
    }

    pub fn get_mut(&mut self, label: &str) -> Result<&mut IdentityEntry, IdentityManagerError> {
        self.identities.get_mut(label)
            .ok_or_else(|| IdentityManagerError::NotFound(label.to_string()))
    }

    pub fn list_labels(&self) -> Vec<&str> {
        self.identities.keys().map(|k| k.as_str()).collect()
    }

    pub fn list_all(&self) -> Vec<&IdentityEntry> {
        self.identities.values().collect()
    }

    pub fn count(&self) -> usize { self.identities.len() }

    pub fn contains(&self, label: &str) -> bool {
        self.identities.contains_key(label)
    }

    pub fn default_label(&self) -> Option<&str> {
        self.default_label.as_deref()
    }

    pub fn to_store_data(&self) -> IdentityStoreData {
        IdentityStoreData {
            identities: self.identities.clone(),
            default_label: self.default_label.clone(),
            schema_version: 1,
        }
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), IdentityManagerError> {
        let store_data = self.to_store_data();
        let json = serde_json::to_string_pretty(&store_data)
            .map_err(|e| IdentityManagerError::Serialization(e.to_string()))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }
        tracing::info!("Identity store saved to {:?}", path);
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self, IdentityManagerError> {
        let json = fs::read_to_string(path)?;
        let store_data: IdentityStoreData = serde_json::from_str(&json)
            .map_err(|e| IdentityManagerError::Serialization(e.to_string()))?;
        if store_data.schema_version != 1 {
            tracing::warn!(
                "Loading identity store with schema version {} (expected 1)",
                store_data.schema_version
            );
        }
        Ok(IdentityManager {
            identities: store_data.identities,
            default_label: store_data.default_label,
        })
    }

    pub fn set_nickname(&mut self, label: &str, nickname: &str) -> Result<(), IdentityManagerError> {
        let entry = self.get_mut(label)?;
        entry.nickname = Some(nickname.to_string());
        entry.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn set_description(&mut self, label: &str, description: &str) -> Result<(), IdentityManagerError> {
        let entry = self.get_mut(label)?;
        entry.description = Some(description.to_string());
        entry.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn set_metadata(&mut self, label: &str, key: &str, value: &str) -> Result<(), IdentityManagerError> {
        let entry = self.get_mut(label)?;
        entry.metadata.insert(key.to_string(), value.to_string());
        entry.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn clear(&mut self) {
        self.identities.clear();
        self.default_label = None;
    }

    pub fn merge(&mut self, other: &IdentityManager) {
        for (label, entry) in &other.identities {
            if !self.identities.contains_key(label) {
                self.identities.insert(label.clone(), entry.clone());
            }
        }
        if self.default_label.is_none() {
            if let Some(other_default) = &other.default_label {
                if self.identities.contains_key(other_default) {
                    self.default_label = Some(other_default.clone());
                    if let Some(entry) = self.identities.get_mut(other_default) {
                        entry.is_default = true;
                    }
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_identity() {
        let mut mgr = IdentityManager::new();
        let entry = mgr.create_identity("test", Some("Test User"), Some("A test identity")).unwrap();
        assert_eq!(entry.label, "test");
        assert_eq!(entry.nickname.unwrap(), "Test User");
        assert_eq!(entry.description.unwrap(), "A test identity");
        assert_eq!(mgr.count(), 1);
        assert!(entry.is_default);
    }

    #[test]
    fn test_multiple_identities() {
        let mut mgr = IdentityManager::new();
        mgr.create_identity("alice", Some("Alice"), None).unwrap();
        mgr.create_identity("bob", Some("Bob"), None).unwrap();
        mgr.create_identity("carol", Some("Carol"), None).unwrap();
        assert_eq!(mgr.count(), 3);
        assert_eq!(mgr.list_labels().len(), 3);
        let active = mgr.get_active().unwrap();
        assert_eq!(active.label, "alice");
        assert!(active.is_default);
    }

    #[test]
    fn test_set_default() {
        let mut mgr = IdentityManager::new();
        mgr.create_identity("first", None, None).unwrap();
        mgr.create_identity("second", None, None).unwrap();
        mgr.create_identity("third", None, None).unwrap();
        assert_eq!(mgr.default_label().unwrap(), "first");
        mgr.set_default("third").unwrap();
        assert_eq!(mgr.default_label().unwrap(), "third");
        let active = mgr.get_active().unwrap();
        assert_eq!(active.label, "third");
        assert!(active.is_default);
        let first = mgr.get("first").unwrap();
        assert!(!first.is_default);
    }

    #[test]
    fn test_remove_identity() {
        let mut mgr = IdentityManager::new();
        mgr.create_identity("a", None, None).unwrap();
        mgr.create_identity("b", None, None).unwrap();
        mgr.create_identity("c", None, None).unwrap();
        mgr.remove_identity("a").unwrap();
        assert_eq!(mgr.count(), 2);
        assert!(!mgr.contains("a"));
        assert!(mgr.default_label().is_some());
        assert!(mgr.get_active().is_ok());
    }

    #[test]
    fn test_remove_last_identity() {
        let mut mgr = IdentityManager::new();
        mgr.create_identity("only", None, None).unwrap();
        mgr.remove_identity("only").unwrap();
        assert_eq!(mgr.count(), 0);
        assert!(mgr.default_label().is_none());
        assert!(mgr.get_active().is_err());
    }

    #[test]
    fn test_duplicate_label_error() {
        let mut mgr = IdentityManager::new();
        mgr.create_identity("unique", None, None).unwrap();
        let result = mgr.create_identity("unique", None, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IdentityManagerError::AlreadyExists(_)));
    }

    #[test]
    fn test_not_found_error() {
        let mgr = IdentityManager::new();
        let result = mgr.get("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IdentityManagerError::NotFound(_)));
    }

    #[test]
    fn test_import_identity() {
        let mut mgr = IdentityManager::new();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let entry = mgr.import_identity("imported", mnemonic, Some("Imported"), true).unwrap();
        assert_eq!(entry.label, "imported");
        assert_eq!(entry.nickname.unwrap(), "Imported");
        assert!(entry.is_default);
        assert!(entry.did.starts_with("did:key:"));
        assert!(!entry.peer_id.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut mgr = IdentityManager::new();
        mgr.create_identity("alice", Some("Alice"), Some("First user")).unwrap();
        mgr.create_identity("bob", Some("Bob"), None).unwrap();
        let store_data = mgr.to_store_data();
        let json = serde_json::to_string_pretty(&store_data).unwrap();
        let deserialized: IdentityStoreData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.identities.len(), 2);
        assert!(deserialized.default_label.is_some());
        assert_eq!(deserialized.schema_version, 1);
    }

    #[test]
    fn test_file_persistence() {
        let mut mgr = IdentityManager::new();
        mgr.create_identity("test", None, None).unwrap();
        let tmp = std::env::temp_dir().join("scp_test_identities.json");
        mgr.save_to_file(&tmp).unwrap();
        let loaded = IdentityManager::load_from_file(&tmp).unwrap();
        assert_eq!(loaded.count(), 1);
        assert!(loaded.contains("test"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_set_nickname_and_description() {
        let mut mgr = IdentityManager::new();
        mgr.create_identity("dev", None, None).unwrap();
        mgr.set_nickname("dev", "Developer").unwrap();
        mgr.set_description("dev", "Development identity").unwrap();
        let entry = mgr.get("dev").unwrap();
        assert_eq!(entry.nickname.as_deref(), Some("Developer"));
        assert_eq!(entry.description.as_deref(), Some("Development identity"));
    }

    #[test]
    fn test_metadata() {
        let mut mgr = IdentityManager::new();
        mgr.create_identity("bot", Some("Bot Agent"), None).unwrap();
        mgr.set_metadata("bot", "role", "moderator").unwrap();
        mgr.set_metadata("bot", "department", "engineering").unwrap();
        let entry = mgr.get("bot").unwrap();
        assert_eq!(entry.metadata.get("role").unwrap(), "moderator");
        assert_eq!(entry.metadata.get("department").unwrap(), "engineering");
    }

    #[test]
    fn test_merge() {
        let mut mgr1 = IdentityManager::new();
        mgr1.create_identity("alice", None, None).unwrap();
        let mut mgr2 = IdentityManager::new();
        mgr2.create_identity("bob", None, None).unwrap();
        mgr2.create_identity("carol", None, None).unwrap();
        mgr1.merge(&mgr2);
        assert_eq!(mgr1.count(), 3);
        assert!(mgr1.contains("alice"));
        assert!(mgr1.contains("bob"));
        assert!(mgr1.contains("carol"));
        assert_eq!(mgr1.default_label().unwrap(), "alice");
    }

    #[test]
    fn test_invalid_label() {
        let mut mgr = IdentityManager::new();
        assert!(mgr.create_identity("", None, None).is_err());
        assert!(mgr.create_identity("in valid", None, None).is_err());
        assert!(mgr.create_identity("valid-label_123", None, None).is_ok());
    }

    #[test]
    fn test_get_active_mut() {
        let mut mgr = IdentityManager::new();
        mgr.create_identity("main", Some("Main"), None).unwrap();
        {
            let active = mgr.get_active_mut().unwrap();
            active.nickname = Some("Updated".to_string());
        }
        let active = mgr.get_active().unwrap();
        assert_eq!(active.nickname.as_deref(), Some("Updated"));
    }
}

