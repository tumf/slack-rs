//! Idempotency handler for write operations

use super::store::{IdempotencyError, IdempotencyStore};
use super::types::{IdempotencyStatus, RequestFingerprint, ScopedKey};
use serde_json::Value;

/// Result of idempotency check
pub enum IdempotencyCheckResult {
    /// No idempotency key provided, proceed normally
    NoKey,
    /// Key provided and operation should be replayed
    Replay {
        response: Value,
        key: String,
        status: IdempotencyStatus,
    },
    /// Key provided but no cached result, proceed and store
    Execute {
        key: ScopedKey,
        fingerprint: RequestFingerprint,
    },
}

/// Idempotency handler for write operations
pub struct IdempotencyHandler {
    store: IdempotencyStore,
}

impl IdempotencyHandler {
    /// Create a new handler
    pub fn new() -> Result<Self, IdempotencyError> {
        Ok(Self {
            store: IdempotencyStore::new()?,
        })
    }

    /// Check if operation should be executed or replayed
    ///
    /// # Arguments
    /// * `idempotency_key` - Optional idempotency key
    /// * `team_id` - Team ID
    /// * `user_id` - User ID
    /// * `method` - API method name
    /// * `params` - Request parameters for fingerprinting
    ///
    /// # Returns
    /// * `Ok(IdempotencyCheckResult)` with next action
    /// * `Err(IdempotencyError)` if fingerprint mismatch or other error
    pub fn check(
        &self,
        idempotency_key: Option<String>,
        team_id: String,
        user_id: String,
        method: String,
        params: &serde_json::Map<String, Value>,
    ) -> Result<IdempotencyCheckResult, IdempotencyError> {
        let Some(key_str) = idempotency_key else {
            return Ok(IdempotencyCheckResult::NoKey);
        };

        let scoped_key = ScopedKey::new(team_id, user_id, method, key_str.clone());
        let fingerprint = RequestFingerprint::from_params(params);

        match self.store.check(&scoped_key, &fingerprint)? {
            Some(response) => Ok(IdempotencyCheckResult::Replay {
                response,
                key: key_str,
                status: IdempotencyStatus::Replayed,
            }),
            None => Ok(IdempotencyCheckResult::Execute {
                key: scoped_key,
                fingerprint,
            }),
        }
    }

    /// Store operation result
    pub fn store(
        &mut self,
        key: ScopedKey,
        fingerprint: RequestFingerprint,
        response: Value,
    ) -> Result<(), IdempotencyError> {
        self.store.put(key, fingerprint, response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn create_test_handler() -> (IdempotencyHandler, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("idempotency_store.json");
        let store = IdempotencyStore::with_path(store_path).unwrap();
        (IdempotencyHandler { store }, temp_dir)
    }

    #[test]
    fn test_no_key() {
        let (handler, _temp) = create_test_handler();
        let params = serde_json::Map::new();

        let result = handler
            .check(
                None,
                "T123".into(),
                "U456".into(),
                "chat.postMessage".into(),
                &params,
            )
            .unwrap();

        assert!(matches!(result, IdempotencyCheckResult::NoKey));
    }

    #[test]
    fn test_execute_first_time() {
        let (handler, _temp) = create_test_handler();
        let mut params = serde_json::Map::new();
        params.insert("channel".into(), json!("C123"));
        params.insert("text".into(), json!("hello"));

        let result = handler
            .check(
                Some("test-key-1".into()),
                "T123".into(),
                "U456".into(),
                "chat.postMessage".into(),
                &params,
            )
            .unwrap();

        assert!(matches!(result, IdempotencyCheckResult::Execute { .. }));
    }

    #[test]
    fn test_replay_second_time() {
        let (mut handler, _temp) = create_test_handler();
        let mut params = serde_json::Map::new();
        params.insert("channel".into(), json!("C123"));
        params.insert("text".into(), json!("hello"));

        // First execution - should execute
        let result = handler
            .check(
                Some("test-key-2".into()),
                "T123".into(),
                "U456".into(),
                "chat.postMessage".into(),
                &params,
            )
            .unwrap();

        let (key, fingerprint) = match result {
            IdempotencyCheckResult::Execute { key, fingerprint } => (key, fingerprint),
            _ => panic!("Expected Execute"),
        };

        // Store result
        let response = json!({"ok": true, "ts": "1234567890.123456"});
        handler.store(key, fingerprint, response.clone()).unwrap();

        // Second execution - should replay
        let result2 = handler
            .check(
                Some("test-key-2".into()),
                "T123".into(),
                "U456".into(),
                "chat.postMessage".into(),
                &params,
            )
            .unwrap();

        match result2 {
            IdempotencyCheckResult::Replay { response: r, .. } => {
                assert_eq!(r, response);
            }
            _ => panic!("Expected Replay"),
        }
    }

    #[test]
    fn test_fingerprint_mismatch_error() {
        let (mut handler, _temp) = create_test_handler();

        // First request
        let mut params1 = serde_json::Map::new();
        params1.insert("channel".into(), json!("C123"));
        params1.insert("text".into(), json!("hello"));

        let result = handler
            .check(
                Some("test-key-3".into()),
                "T123".into(),
                "U456".into(),
                "chat.postMessage".into(),
                &params1,
            )
            .unwrap();

        let (key, fingerprint) = match result {
            IdempotencyCheckResult::Execute { key, fingerprint } => (key, fingerprint),
            _ => panic!("Expected Execute"),
        };

        let response = json!({"ok": true});
        handler.store(key, fingerprint, response).unwrap();

        // Second request with different params but same key - should error
        let mut params2 = serde_json::Map::new();
        params2.insert("channel".into(), json!("C123"));
        params2.insert("text".into(), json!("goodbye"));

        let result2 = handler.check(
            Some("test-key-3".into()),
            "T123".into(),
            "U456".into(),
            "chat.postMessage".into(),
            &params2,
        );

        assert!(matches!(
            result2,
            Err(IdempotencyError::FingerprintMismatch)
        ));
    }
}
