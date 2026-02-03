use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenStoreError {
    #[error("Token not found for key: {0}")]
    NotFound(String),
    #[error("Failed to store token: {0}")]
    StoreFailed(String),
    #[error("Failed to delete token: {0}")]
    DeleteFailed(String),
}

pub type Result<T> = std::result::Result<T, TokenStoreError>;

/// Trait for storing and retrieving tokens securely
pub trait TokenStore: Send + Sync {
    /// Store a token with the given key
    fn set(&self, key: &str, token: &str) -> Result<()>;

    /// Retrieve a token by key
    fn get(&self, key: &str) -> Result<String>;

    /// Delete a token by key
    fn delete(&self, key: &str) -> Result<()>;

    /// Check if a token exists for the given key
    fn exists(&self, key: &str) -> bool;
}

/// In-memory implementation of TokenStore for testing
#[derive(Debug, Clone)]
pub struct InMemoryTokenStore {
    tokens: Arc<Mutex<HashMap<String, String>>>,
}

impl InMemoryTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenStore for InMemoryTokenStore {
    fn set(&self, key: &str, token: &str) -> Result<()> {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.insert(key.to_string(), token.to_string());
        Ok(())
    }

    fn get(&self, key: &str) -> Result<String> {
        let tokens = self.tokens.lock().unwrap();
        tokens
            .get(key)
            .cloned()
            .ok_or_else(|| TokenStoreError::NotFound(key.to_string()))
    }

    fn delete(&self, key: &str) -> Result<()> {
        let mut tokens = self.tokens.lock().unwrap();
        tokens
            .remove(key)
            .ok_or_else(|| TokenStoreError::NotFound(key.to_string()))?;
        Ok(())
    }

    fn exists(&self, key: &str) -> bool {
        let tokens = self.tokens.lock().unwrap();
        tokens.contains_key(key)
    }
}

/// Keyring-based implementation of TokenStore
pub struct KeyringTokenStore {
    service: String,
}

impl KeyringTokenStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }
}

impl TokenStore for KeyringTokenStore {
    fn set(&self, key: &str, token: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, key)
            .map_err(|e| TokenStoreError::StoreFailed(e.to_string()))?;
        entry
            .set_password(token)
            .map_err(|e| TokenStoreError::StoreFailed(e.to_string()))?;
        Ok(())
    }

    fn get(&self, key: &str) -> Result<String> {
        let entry = keyring::Entry::new(&self.service, key)
            .map_err(|e| TokenStoreError::NotFound(e.to_string()))?;
        entry
            .get_password()
            .map_err(|_| TokenStoreError::NotFound(key.to_string()))
    }

    fn delete(&self, key: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, key)
            .map_err(|e| TokenStoreError::DeleteFailed(e.to_string()))?;
        entry
            .delete_credential()
            .map_err(|e| TokenStoreError::DeleteFailed(e.to_string()))?;
        Ok(())
    }

    fn exists(&self, key: &str) -> bool {
        if let Ok(entry) = keyring::Entry::new(&self.service, key) {
            entry.get_password().is_ok()
        } else {
            false
        }
    }
}

/// Helper function to create a token key from team_id and user_id
pub fn make_token_key(team_id: &str, user_id: &str) -> String {
    format!("{}:{}", team_id, user_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_token_store_set_get() {
        let store = InMemoryTokenStore::new();
        let key = "T123:U456";
        let token = "xoxb-test-token";

        store.set(key, token).unwrap();
        assert_eq!(store.get(key).unwrap(), token);
    }

    #[test]
    fn test_in_memory_token_store_delete() {
        let store = InMemoryTokenStore::new();
        let key = "T123:U456";
        let token = "xoxb-test-token";

        store.set(key, token).unwrap();
        assert!(store.exists(key));

        store.delete(key).unwrap();
        assert!(!store.exists(key));
        assert!(store.get(key).is_err());
    }

    #[test]
    fn test_in_memory_token_store_not_found() {
        let store = InMemoryTokenStore::new();
        let result = store.get("nonexistent");
        assert!(result.is_err());
        match result {
            Err(TokenStoreError::NotFound(_)) => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_in_memory_token_store_exists() {
        let store = InMemoryTokenStore::new();
        let key = "T123:U456";

        assert!(!store.exists(key));
        store.set(key, "token").unwrap();
        assert!(store.exists(key));
    }

    #[test]
    fn test_make_token_key() {
        let key = make_token_key("T123", "U456");
        assert_eq!(key, "T123:U456");
    }

    #[test]
    fn test_in_memory_token_store_multiple_keys() {
        let store = InMemoryTokenStore::new();

        store.set("T1:U1", "token1").unwrap();
        store.set("T2:U2", "token2").unwrap();

        assert_eq!(store.get("T1:U1").unwrap(), "token1");
        assert_eq!(store.get("T2:U2").unwrap(), "token2");
    }
}
