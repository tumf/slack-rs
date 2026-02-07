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

        // Use cross-platform home directory detection
        let home = directories::BaseDirs::new()
            .ok_or_else(|| {
                TokenStoreError::IoError("Failed to determine home directory".to_string())
            })?
            .home_dir()
            .to_path_buf();

        // Use separate join calls to ensure consistent path separators on Windows
        let config_dir = home.join(".config").join("slack-rs");
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

/// Helper function to create a token key from team_id and user_id
pub fn make_token_key(team_id: &str, user_id: &str) -> String {
    format!("{}:{}", team_id, user_id)
}

/// Helper function to create an OAuth client secret key for a profile
pub fn make_oauth_client_secret_key(profile_name: &str) -> String {
    format!("oauth-client-secret:{}", profile_name)
}

/// Store OAuth client secret in the token store
pub fn store_oauth_client_secret(
    token_store: &dyn TokenStore,
    profile_name: &str,
    client_secret: &str,
) -> Result<()> {
    let key = make_oauth_client_secret_key(profile_name);
    token_store.set(&key, client_secret)
}

/// Retrieve OAuth client secret from the token store
pub fn get_oauth_client_secret(token_store: &dyn TokenStore, profile_name: &str) -> Result<String> {
    let key = make_oauth_client_secret_key(profile_name);
    token_store.get(&key)
}

/// Delete OAuth client secret from the token store
pub fn delete_oauth_client_secret(token_store: &dyn TokenStore, profile_name: &str) -> Result<()> {
    let key = make_oauth_client_secret_key(profile_name);
    token_store.delete(&key)
}

/// Create a token store using FileTokenStore
///
/// This function creates a FileTokenStore with the default path.
///
/// Returns Box<dyn TokenStore> for runtime polymorphism
pub fn create_token_store() -> Result<Box<dyn TokenStore>> {
    let store = FileTokenStore::new()?;
    Ok(Box::new(store))
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

    #[test]
    #[serial_test::serial]
    fn test_create_token_store_file_backend() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tokens_path = temp_dir.path().join("tokens.json");
        std::env::set_var("SLACK_RS_TOKENS_PATH", tokens_path.to_str().unwrap());

        let store = create_token_store().unwrap();

        // Test that the store works
        store.set("test_key", "test_value").unwrap();
        assert_eq!(store.get("test_key").unwrap(), "test_value");

        std::env::remove_var("SLACK_RS_TOKENS_PATH");
    }

    /// Test that file mode uses existing tokens.json path and key format
    /// This verifies backward compatibility with existing token storage
    #[test]
    #[serial_test::serial]
    fn test_file_mode_uses_existing_path_and_key_format() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tokens_path = temp_dir.path().join("tokens.json");

        // Set environment to use file backend with custom path
        std::env::set_var("SLACK_RS_TOKENS_PATH", tokens_path.to_str().unwrap());

        let store = create_token_store().expect("File backend should work");

        // Test token key format: {team_id}:{user_id}
        let token_key = make_token_key("T123", "U456");
        assert_eq!(token_key, "T123:U456");
        store.set(&token_key, "xoxb-test-token").unwrap();
        assert_eq!(store.get(&token_key).unwrap(), "xoxb-test-token");

        // Test OAuth client secret key format: oauth-client-secret:{profile_name}
        let secret_key = make_oauth_client_secret_key("default");
        assert_eq!(secret_key, "oauth-client-secret:default");
        store.set(&secret_key, "test-secret").unwrap();
        assert_eq!(store.get(&secret_key).unwrap(), "test-secret");

        // Verify the file exists and contains both keys
        assert!(tokens_path.exists());
        let content = std::fs::read_to_string(&tokens_path).unwrap();
        assert!(content.contains("T123:U456"));
        assert!(content.contains("oauth-client-secret:default"));

        std::env::remove_var("SLACK_RS_TOKENS_PATH");
    }

    /// Test default path for FileTokenStore
    /// Verifies that FileTokenStore uses ~/.config/slack-rs/tokens.json by default
    #[test]
    #[serial_test::serial]
    fn test_file_token_store_default_path() {
        // Clear environment override to test actual default
        std::env::remove_var("SLACK_RS_TOKENS_PATH");

        let default_path = FileTokenStore::default_path().unwrap();
        let path_str = default_path.to_string_lossy();

        // Should contain .config/slack-rs/tokens.json
        assert!(
            path_str.contains(".config/slack-rs/tokens.json")
                || path_str.contains(".config\\slack-rs\\tokens.json"),
            "Default path should be ~/.config/slack-rs/tokens.json, got: {}",
            path_str
        );
    }

    /// Comprehensive test for unified credential storage policy
    ///
    /// This test verifies the complete specification:
    /// 1. FileTokenStore is the default and only backend
    /// 2. Both InMemoryTokenStore and FileTokenStore use the same key format (team_id:user_id for tokens, oauth-client-secret:profile for secrets)
    /// 3. InMemoryTokenStore can be used for testing with same key format
    #[test]
    #[serial_test::serial]
    fn test_unified_credential_storage_policy() {
        use tempfile::TempDir;

        // Test 1: Same key format across all backends
        let temp_dir = TempDir::new().unwrap();
        let tokens_path = temp_dir.path().join("tokens.json");
        std::env::set_var("SLACK_RS_TOKENS_PATH", tokens_path.to_str().unwrap());

        // Test with InMemoryTokenStore (for testing)
        let memory_store = InMemoryTokenStore::new();

        // Test with FileTokenStore
        let file_store = FileTokenStore::with_path(tokens_path.clone()).unwrap();

        // Both should use the same key format
        let token_key = make_token_key("T123", "U456");
        let secret_key = make_oauth_client_secret_key("default");

        // Store in memory store
        memory_store.set(&token_key, "token1").unwrap();
        memory_store.set(&secret_key, "secret1").unwrap();

        // Store in file store
        file_store.set(&token_key, "token2").unwrap();
        file_store.set(&secret_key, "secret2").unwrap();

        // Verify retrieval works with same keys
        assert_eq!(memory_store.get(&token_key).unwrap(), "token1");
        assert_eq!(memory_store.get(&secret_key).unwrap(), "secret1");
        assert_eq!(file_store.get(&token_key).unwrap(), "token2");
        assert_eq!(file_store.get(&secret_key).unwrap(), "secret2");

        // Test 2: Verify helper functions work across all backends
        store_oauth_client_secret(&memory_store, "test", "secret123").unwrap();
        assert_eq!(
            get_oauth_client_secret(&memory_store, "test").unwrap(),
            "secret123"
        );

        store_oauth_client_secret(&file_store, "test", "secret456").unwrap();
        assert_eq!(
            get_oauth_client_secret(&file_store, "test").unwrap(),
            "secret456"
        );

        // Clean up
        std::env::remove_var("SLACK_RS_TOKENS_PATH");
    }

    /// Test that InMemoryTokenStore works as a test/mock backend
    /// with the same interface as production backends
    #[test]
    fn test_in_memory_token_store_as_mock() {
        let store = InMemoryTokenStore::new();

        // Test token storage and retrieval
        let token_key = make_token_key("T999", "U888");
        store.set(&token_key, "xoxb-mock-token").unwrap();
        assert_eq!(store.get(&token_key).unwrap(), "xoxb-mock-token");

        // Test OAuth secret storage and retrieval
        let secret_key = make_oauth_client_secret_key("mock-profile");
        store.set(&secret_key, "mock-secret").unwrap();
        assert_eq!(store.get(&secret_key).unwrap(), "mock-secret");

        // Test existence checks
        assert!(store.exists(&token_key));
        assert!(store.exists(&secret_key));
        assert!(!store.exists("nonexistent"));

        // Test deletion
        store.delete(&token_key).unwrap();
        assert!(!store.exists(&token_key));

        // Test helper functions
        store_oauth_client_secret(&store, "test", "test-secret").unwrap();
        assert_eq!(
            get_oauth_client_secret(&store, "test").unwrap(),
            "test-secret"
        );
        delete_oauth_client_secret(&store, "test").unwrap();
        assert!(!store.exists(&make_oauth_client_secret_key("test")));
    }
}
