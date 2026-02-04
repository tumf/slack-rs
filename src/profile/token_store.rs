use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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
    #[error("IO error: {0}")]
    IoError(String),
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

/// File-based implementation of TokenStore
/// Stores tokens in ~/.config/slack-rs/tokens.json with restricted permissions (0600)
#[derive(Debug, Clone)]
pub struct FileTokenStore {
    file_path: PathBuf,
    tokens: Arc<Mutex<HashMap<String, String>>>,
}

impl FileTokenStore {
    /// Create a new FileTokenStore with the default path (~/.config/slack-rs/tokens.json)
    pub fn new() -> Result<Self> {
        let file_path = Self::default_path()?;
        Self::with_path(file_path)
    }

    /// Create a FileTokenStore with a custom path
    pub fn with_path(file_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                TokenStoreError::IoError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Load existing tokens or create empty map
        let tokens = if file_path.exists() {
            Self::load_tokens(&file_path)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            file_path,
            tokens: Arc::new(Mutex::new(tokens)),
        })
    }

    /// Get the default path for the tokens file
    /// Can be overridden with SLACK_RS_TOKENS_PATH environment variable (useful for testing)
    pub fn default_path() -> Result<PathBuf> {
        // Check for environment variable override (useful for testing)
        if let Ok(path) = std::env::var("SLACK_RS_TOKENS_PATH") {
            return Ok(PathBuf::from(path));
        }

        // Use ~/.config/slack-rs/ instead of platform-specific config directory
        let home = std::env::var("HOME").map_err(|_| {
            TokenStoreError::IoError("HOME environment variable not set".to_string())
        })?;
        let config_dir = PathBuf::from(home).join(".config/slack-rs");
        Ok(config_dir.join("tokens.json"))
    }

    /// Load tokens from file
    fn load_tokens(path: &Path) -> Result<HashMap<String, String>> {
        let content = fs::read_to_string(path)
            .map_err(|e| TokenStoreError::IoError(format!("Failed to read tokens file: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| TokenStoreError::IoError(format!("Failed to parse tokens file: {}", e)))
    }

    /// Save tokens to file with restricted permissions
    fn save_tokens(&self) -> Result<()> {
        let tokens = self.tokens.lock().unwrap();
        let content = serde_json::to_string_pretty(&*tokens).map_err(|e| {
            TokenStoreError::StoreFailed(format!("Failed to serialize tokens: {}", e))
        })?;

        // Write to file
        fs::write(&self.file_path, content).map_err(|e| {
            TokenStoreError::StoreFailed(format!("Failed to write tokens file: {}", e))
        })?;

        // Set file permissions to 0600 (owner read/write only) on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.file_path, permissions).map_err(|e| {
                TokenStoreError::StoreFailed(format!("Failed to set file permissions: {}", e))
            })?;
        }

        Ok(())
    }
}

impl Default for FileTokenStore {
    fn default() -> Self {
        Self::new().expect("Failed to create FileTokenStore")
    }
}

impl TokenStore for FileTokenStore {
    fn set(&self, key: &str, token: &str) -> Result<()> {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.insert(key.to_string(), token.to_string());
        drop(tokens); // Release lock before saving
        self.save_tokens()
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
        drop(tokens); // Release lock before saving
        self.save_tokens()
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
    /// Create a new KeyringTokenStore with a custom service name
    /// For production use, prefer `default()` which uses the standard service name
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    /// Create a KeyringTokenStore with the default service name "slackcli"
    /// This is the recommended way to create a KeyringTokenStore for production use
    /// Note: Service name kept as "slackcli" for backward compatibility with existing keyring entries
    pub fn default_service() -> Self {
        Self {
            service: "slackcli".to_string(),
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

/// Helper function to create an OAuth client secret key for a profile
pub fn make_oauth_client_secret_key(profile_name: &str) -> String {
    format!("oauth-client-secret:{}", profile_name)
}

/// Store OAuth client secret in the keyring
pub fn store_oauth_client_secret(
    token_store: &impl TokenStore,
    profile_name: &str,
    client_secret: &str,
) -> Result<()> {
    let key = make_oauth_client_secret_key(profile_name);
    token_store.set(&key, client_secret)
}

/// Retrieve OAuth client secret from the keyring
pub fn get_oauth_client_secret(
    token_store: &impl TokenStore,
    profile_name: &str,
) -> Result<String> {
    let key = make_oauth_client_secret_key(profile_name);
    token_store.get(&key)
}

/// Delete OAuth client secret from the keyring
pub fn delete_oauth_client_secret(token_store: &impl TokenStore, profile_name: &str) -> Result<()> {
    let key = make_oauth_client_secret_key(profile_name);
    token_store.delete(&key)
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

    #[test]
    fn test_keyring_token_store_default_service() {
        let store = KeyringTokenStore::default_service();
        assert_eq!(store.service, "slackcli");
    }

    #[test]
    fn test_make_oauth_client_secret_key() {
        let key = make_oauth_client_secret_key("default");
        assert_eq!(key, "oauth-client-secret:default");
    }

    #[test]
    fn test_store_and_get_oauth_client_secret() {
        let store = InMemoryTokenStore::new();
        let profile_name = "test-profile";
        let client_secret = "test-secret-123";

        store_oauth_client_secret(&store, profile_name, client_secret).unwrap();
        let retrieved = get_oauth_client_secret(&store, profile_name).unwrap();
        assert_eq!(retrieved, client_secret);
    }

    #[test]
    fn test_delete_oauth_client_secret() {
        let store = InMemoryTokenStore::new();
        let profile_name = "test-profile";
        let client_secret = "test-secret-123";

        store_oauth_client_secret(&store, profile_name, client_secret).unwrap();
        assert!(get_oauth_client_secret(&store, profile_name).is_ok());

        delete_oauth_client_secret(&store, profile_name).unwrap();
        assert!(get_oauth_client_secret(&store, profile_name).is_err());
    }

    #[test]
    fn test_file_token_store_set_get() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("tokens.json");
        let store = FileTokenStore::with_path(file_path.clone()).unwrap();

        let key = "T123:U456";
        let token = "xoxb-test-token";

        store.set(key, token).unwrap();
        assert_eq!(store.get(key).unwrap(), token);

        // Verify file exists
        assert!(file_path.exists());

        // Verify file permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&file_path).unwrap();
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn test_file_token_store_delete() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("tokens.json");
        let store = FileTokenStore::with_path(file_path).unwrap();

        let key = "T123:U456";
        let token = "xoxb-test-token";

        store.set(key, token).unwrap();
        assert!(store.exists(key));

        store.delete(key).unwrap();
        assert!(!store.exists(key));
        assert!(store.get(key).is_err());
    }

    #[test]
    fn test_file_token_store_persistence() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("tokens.json");

        // Create store and save token
        {
            let store = FileTokenStore::with_path(file_path.clone()).unwrap();
            store.set("T123:U456", "xoxb-test-token").unwrap();
        }

        // Create new store instance and verify token persists
        {
            let store = FileTokenStore::with_path(file_path).unwrap();
            assert_eq!(store.get("T123:U456").unwrap(), "xoxb-test-token");
        }
    }

    #[test]
    fn test_file_token_store_multiple_keys() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("tokens.json");
        let store = FileTokenStore::with_path(file_path).unwrap();

        store.set("T1:U1", "token1").unwrap();
        store.set("T2:U2", "token2").unwrap();
        store
            .set("oauth-client-secret:default", "secret123")
            .unwrap();

        assert_eq!(store.get("T1:U1").unwrap(), "token1");
        assert_eq!(store.get("T2:U2").unwrap(), "token2");
        assert_eq!(
            store.get("oauth-client-secret:default").unwrap(),
            "secret123"
        );
    }

    #[test]
    fn test_file_token_store_not_found() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("tokens.json");
        let store = FileTokenStore::with_path(file_path).unwrap();

        let result = store.get("nonexistent");
        assert!(result.is_err());
        match result {
            Err(TokenStoreError::NotFound(_)) => {}
            _ => panic!("Expected NotFound error"),
        }
    }
}
