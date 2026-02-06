//! Idempotency store implementation with JSON persistence

use super::types::{IdempotencyEntry, RequestFingerprint, ScopedKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Default TTL: 7 days in seconds
pub const DEFAULT_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

/// Default capacity limit
pub const DEFAULT_CAPACITY: usize = 10_000;

/// Idempotency store errors
#[derive(Debug, Error)]
pub enum IdempotencyError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Fingerprint mismatch: different request with same idempotency key")]
    FingerprintMismatch,

    #[error("Store error: {0}")]
    StoreError(String),
}

/// Persistent idempotency store
#[derive(Debug, Serialize, Deserialize)]
pub struct IdempotencyStore {
    /// Map of scoped keys to entries
    entries: HashMap<String, IdempotencyEntry>,

    /// Capacity limit
    #[serde(skip)]
    capacity: usize,

    /// Store file path
    #[serde(skip)]
    store_path: PathBuf,
}

impl IdempotencyStore {
    /// Create a new store with default config dir
    pub fn new() -> Result<Self, IdempotencyError> {
        let store_path = Self::default_store_path()?;
        Self::load_or_create(store_path, DEFAULT_CAPACITY)
    }

    /// Create a new store with custom path
    pub fn with_path(store_path: PathBuf) -> Result<Self, IdempotencyError> {
        Self::load_or_create(store_path, DEFAULT_CAPACITY)
    }

    /// Get default store path in config directory
    fn default_store_path() -> Result<PathBuf, IdempotencyError> {
        let project_dirs = directories::ProjectDirs::from("", "", "slack-rs")
            .ok_or_else(|| IdempotencyError::StoreError("Cannot find config directory".into()))?;
        let config_dir = project_dirs.config_dir();

        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        Ok(config_dir.join("idempotency_store.json"))
    }

    /// Load store from disk or create new if doesn't exist
    fn load_or_create(store_path: PathBuf, capacity: usize) -> Result<Self, IdempotencyError> {
        if store_path.exists() {
            let content = fs::read_to_string(&store_path)?;
            let mut store: IdempotencyStore = serde_json::from_str(&content)?;
            store.store_path = store_path;
            store.capacity = capacity;

            // Run GC on load
            store.gc()?;
            Ok(store)
        } else {
            let store = IdempotencyStore {
                entries: HashMap::new(),
                capacity,
                store_path,
            };

            // Create parent directory if needed
            if let Some(parent) = store.store_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            // Set file permissions on Unix (0600)
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                let mut options = fs::OpenOptions::new();
                options.write(true).create(true).mode(0o600);
                options.open(&store.store_path)?;
            }

            #[cfg(not(unix))]
            {
                // On non-Unix, just create the file
                fs::write(&store.store_path, "{}")?;
            }

            store.save()?;
            Ok(store)
        }
    }

    /// Get entry if exists
    pub fn get(&self, key: &ScopedKey) -> Option<&IdempotencyEntry> {
        let key_str = key.to_string();
        self.entries.get(&key_str).filter(|e| !e.is_expired())
    }

    /// Check and validate entry, returning response if valid
    ///
    /// Returns:
    /// - Ok(Some(response)) if entry exists and fingerprint matches
    /// - Ok(None) if entry doesn't exist
    /// - Err if entry exists but fingerprint doesn't match
    pub fn check(
        &self,
        key: &ScopedKey,
        fingerprint: &RequestFingerprint,
    ) -> Result<Option<serde_json::Value>, IdempotencyError> {
        if let Some(entry) = self.get(key) {
            if entry.fingerprint == *fingerprint {
                Ok(Some(entry.response.clone()))
            } else {
                Err(IdempotencyError::FingerprintMismatch)
            }
        } else {
            Ok(None)
        }
    }

    /// Store entry
    pub fn put(
        &mut self,
        key: ScopedKey,
        fingerprint: RequestFingerprint,
        response: serde_json::Value,
    ) -> Result<(), IdempotencyError> {
        // Run GC before adding new entry
        self.gc()?;

        let entry = IdempotencyEntry::new(fingerprint, response, DEFAULT_TTL_SECONDS);
        let key_str = key.to_string();
        self.entries.insert(key_str, entry);

        self.save()
    }

    /// Garbage collection: remove expired and enforce capacity limit
    fn gc(&mut self) -> Result<(), IdempotencyError> {
        // Remove expired entries
        self.entries.retain(|_, entry| !entry.is_expired());

        // If over capacity, remove oldest entries
        if self.entries.len() > self.capacity {
            let mut entries: Vec<_> = self
                .entries
                .iter()
                .map(|(k, v)| (k.clone(), v.created_at))
                .collect();
            entries.sort_by_key(|(_, created_at)| *created_at);

            let to_remove = self.entries.len() - self.capacity;
            for (key, _) in entries.iter().take(to_remove) {
                self.entries.remove(key);
            }
        }

        Ok(())
    }

    /// Save store to disk
    fn save(&self) -> Result<(), IdempotencyError> {
        let content = serde_json::to_string_pretty(&self)?;
        fs::write(&self.store_path, content)?;

        // Ensure permissions on Unix
        #[cfg(unix)]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;
            let perms = Permissions::from_mode(0o600);
            fs::set_permissions(&self.store_path, perms)?;
        }

        Ok(())
    }

    /// Get number of entries in store
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if store is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn create_test_store() -> (IdempotencyStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("test_store.json");
        let store = IdempotencyStore::with_path(store_path).unwrap();
        (store, temp_dir)
    }

    #[test]
    fn test_store_creation() {
        let (store, _temp) = create_test_store();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_put_and_get() {
        let (mut store, _temp) = create_test_store();

        let key = ScopedKey::new(
            "T123".into(),
            "U456".into(),
            "chat.postMessage".into(),
            "test-key".into(),
        );

        let mut params = serde_json::Map::new();
        params.insert("channel".into(), json!("C123"));
        params.insert("text".into(), json!("hello"));
        let fingerprint = RequestFingerprint::from_params(&params);

        let response = json!({"ok": true, "ts": "1234567890.123456"});

        // Store entry
        store
            .put(key.clone(), fingerprint.clone(), response.clone())
            .unwrap();

        // Check retrieval
        let result = store.check(&key, &fingerprint).unwrap();
        assert_eq!(result, Some(response));
    }

    #[test]
    fn test_fingerprint_mismatch() {
        let (mut store, _temp) = create_test_store();

        let key = ScopedKey::new(
            "T123".into(),
            "U456".into(),
            "chat.postMessage".into(),
            "test-key".into(),
        );

        let mut params1 = serde_json::Map::new();
        params1.insert("channel".into(), json!("C123"));
        params1.insert("text".into(), json!("hello"));
        let fingerprint1 = RequestFingerprint::from_params(&params1);

        let response = json!({"ok": true, "ts": "1234567890.123456"});
        store.put(key.clone(), fingerprint1, response).unwrap();

        // Different params with same key
        let mut params2 = serde_json::Map::new();
        params2.insert("channel".into(), json!("C123"));
        params2.insert("text".into(), json!("different"));
        let fingerprint2 = RequestFingerprint::from_params(&params2);

        // Should fail with fingerprint mismatch
        let result = store.check(&key, &fingerprint2);
        assert!(matches!(result, Err(IdempotencyError::FingerprintMismatch)));
    }

    #[test]
    fn test_gc_expired_entries() {
        let (mut store, _temp) = create_test_store();

        let key = ScopedKey::new(
            "T123".into(),
            "U456".into(),
            "chat.postMessage".into(),
            "test-key".into(),
        );

        let mut params = serde_json::Map::new();
        params.insert("channel".into(), json!("C123"));
        let fingerprint = RequestFingerprint::from_params(&params);

        // Create expired entry (negative TTL to ensure it's already expired)
        let response = json!({"ok": true});
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let entry = IdempotencyEntry {
            fingerprint: fingerprint.clone(),
            response,
            created_at: now - 10,
            expires_at: now - 5, // Already expired
        };
        store.entries.insert(key.to_string(), entry);

        assert_eq!(store.len(), 1);

        // GC should remove expired entry
        store.gc().unwrap();
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_gc_capacity_limit() {
        let (mut store, _temp) = create_test_store();
        store.capacity = 3;

        // Add entries with staggered timestamps
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for i in 0..5 {
            let key = ScopedKey::new(
                "T123".into(),
                "U456".into(),
                "chat.postMessage".into(),
                format!("key-{}", i),
            );

            let mut params = serde_json::Map::new();
            params.insert("i".into(), json!(i));
            let fingerprint = RequestFingerprint::from_params(&params);

            let response = json!({"ok": true, "i": i});

            // Manually create entry with staggered timestamps
            let entry = IdempotencyEntry {
                fingerprint,
                response,
                created_at: now + i,
                expires_at: now + DEFAULT_TTL_SECONDS + i,
            };

            // Add directly to bypass GC in put()
            let key_str = key.to_string();
            store.entries.insert(key_str, entry);
        }

        // Now run GC manually
        store.gc().unwrap();

        // Should only have 3 entries (capacity limit)
        assert_eq!(store.len(), 3);

        // Oldest entries should be removed (key-0 and key-1)
        let key0 = ScopedKey::new(
            "T123".into(),
            "U456".into(),
            "chat.postMessage".into(),
            "key-0".into(),
        );
        assert!(store.get(&key0).is_none());
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("test_store.json");

        let key = ScopedKey::new(
            "T123".into(),
            "U456".into(),
            "chat.postMessage".into(),
            "test-key".into(),
        );

        let mut params = serde_json::Map::new();
        params.insert("channel".into(), json!("C123"));
        let fingerprint = RequestFingerprint::from_params(&params);

        let response = json!({"ok": true});

        // Create and save
        {
            let mut store = IdempotencyStore::with_path(store_path.clone()).unwrap();
            store
                .put(key.clone(), fingerprint.clone(), response.clone())
                .unwrap();
        }

        // Load and verify
        {
            let store = IdempotencyStore::with_path(store_path).unwrap();
            let result = store.check(&key, &fingerprint).unwrap();
            assert_eq!(result, Some(response));
        }
    }
}
