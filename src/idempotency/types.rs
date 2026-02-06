//! Types for idempotency tracking

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// Idempotency status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IdempotencyStatus {
    /// Operation was executed
    Executed,
    /// Operation was replayed from cache
    Replayed,
}

/// Scoped idempotency key
///
/// Format: team_id/user_id/method/idempotency_key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ScopedKey {
    pub team_id: String,
    pub user_id: String,
    pub method: String,
    pub idempotency_key: String,
}

impl ScopedKey {
    /// Create a new scoped key
    pub fn new(team_id: String, user_id: String, method: String, idempotency_key: String) -> Self {
        Self {
            team_id,
            user_id,
            method,
            idempotency_key,
        }
    }
}

impl std::fmt::Display for ScopedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.team_id, self.user_id, self.method, self.idempotency_key
        )
    }
}

/// Request fingerprint for duplicate detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestFingerprint {
    /// SHA-256 hash of normalized request parameters
    pub hash: String,
}

impl RequestFingerprint {
    /// Create fingerprint from request parameters
    pub fn from_params(params: &serde_json::Map<String, Value>) -> Self {
        use sha2::{Digest, Sha256};

        // Create a sorted JSON string for stable hashing
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);

        let mut hasher = Sha256::new();
        for (key, value) in sorted_params {
            hasher.update(key.as_bytes());
            hasher.update(b":");
            // Serialize value to stable JSON string
            let value_str = serde_json::to_string(value).unwrap_or_default();
            hasher.update(value_str.as_bytes());
            hasher.update(b";");
        }

        let result = hasher.finalize();
        Self {
            hash: format!("{:x}", result),
        }
    }
}

/// Idempotency entry stored in the cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyEntry {
    /// Request fingerprint
    pub fingerprint: RequestFingerprint,

    /// Stored response
    pub response: Value,

    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,

    /// Expiration timestamp (Unix epoch seconds)
    pub expires_at: u64,
}

impl IdempotencyEntry {
    /// Create a new entry with TTL in seconds
    pub fn new(fingerprint: RequestFingerprint, response: Value, ttl_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            fingerprint,
            response,
            created_at: now,
            expires_at: now + ttl_seconds,
        }
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_scoped_key_creation() {
        let key = ScopedKey::new(
            "T123".into(),
            "U456".into(),
            "chat.postMessage".into(),
            "my-key".into(),
        );

        assert_eq!(key.team_id, "T123");
        assert_eq!(key.user_id, "U456");
        assert_eq!(key.method, "chat.postMessage");
        assert_eq!(key.idempotency_key, "my-key");
    }

    #[test]
    fn test_scoped_key_to_string() {
        let key = ScopedKey::new(
            "T123".into(),
            "U456".into(),
            "chat.postMessage".into(),
            "my-key".into(),
        );

        assert_eq!(key.to_string(), "T123/U456/chat.postMessage/my-key");
    }

    #[test]
    fn test_fingerprint_same_params() {
        let mut params1 = serde_json::Map::new();
        params1.insert("channel".into(), json!("C123"));
        params1.insert("text".into(), json!("hello"));

        let mut params2 = serde_json::Map::new();
        params2.insert("channel".into(), json!("C123"));
        params2.insert("text".into(), json!("hello"));

        let fp1 = RequestFingerprint::from_params(&params1);
        let fp2 = RequestFingerprint::from_params(&params2);

        assert_eq!(fp1.hash, fp2.hash);
    }

    #[test]
    fn test_fingerprint_different_params() {
        let mut params1 = serde_json::Map::new();
        params1.insert("channel".into(), json!("C123"));
        params1.insert("text".into(), json!("hello"));

        let mut params2 = serde_json::Map::new();
        params2.insert("channel".into(), json!("C123"));
        params2.insert("text".into(), json!("goodbye"));

        let fp1 = RequestFingerprint::from_params(&params1);
        let fp2 = RequestFingerprint::from_params(&params2);

        assert_ne!(fp1.hash, fp2.hash);
    }

    #[test]
    fn test_fingerprint_order_independence() {
        let mut params1 = serde_json::Map::new();
        params1.insert("channel".into(), json!("C123"));
        params1.insert("text".into(), json!("hello"));
        params1.insert("thread_ts".into(), json!("1234567890.123456"));

        let mut params2 = serde_json::Map::new();
        params2.insert("text".into(), json!("hello"));
        params2.insert("thread_ts".into(), json!("1234567890.123456"));
        params2.insert("channel".into(), json!("C123"));

        let fp1 = RequestFingerprint::from_params(&params1);
        let fp2 = RequestFingerprint::from_params(&params2);

        // Should be the same regardless of insertion order
        assert_eq!(fp1.hash, fp2.hash);
    }

    #[test]
    fn test_entry_expiration() {
        let mut params = serde_json::Map::new();
        params.insert("test".into(), json!("value"));
        let fingerprint = RequestFingerprint::from_params(&params);

        let response = json!({"ok": true});

        // Entry with 1 second TTL
        let entry = IdempotencyEntry::new(fingerprint, response, 1);
        assert!(!entry.is_expired());

        // Wait 2 seconds
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(entry.is_expired());
    }
}
