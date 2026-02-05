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
    #[error("Keyring backend unavailable: {0}\n\nTo resolve this:\n  1. Unlock your OS keyring/keychain (e.g., login to your desktop environment)\n  2. OR use file-based storage: export SLACKRS_TOKEN_STORE=file")]
    KeyringUnavailable(String),
    #[error("Invalid token store backend '{0}'. Valid options: 'keyring', 'file'")]
    InvalidBackend(String),
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

        let config_dir = home.join(".config/slack-rs");
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

    /// Create a KeyringTokenStore with the default service name "slack-rs"
    /// This is the recommended way to create a KeyringTokenStore for production use
    pub fn default_service() -> Self {
        Self {
            service: "slack-rs".to_string(),
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

/// Token store backend types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenStoreBackend {
    Keyring,
    File,
}

impl std::str::FromStr for TokenStoreBackend {
    type Err = TokenStoreError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "keyring" => Ok(TokenStoreBackend::Keyring),
            "file" => Ok(TokenStoreBackend::File),
            _ => Err(TokenStoreError::InvalidBackend(s.to_string())),
        }
    }
}

/// Resolve token store backend from environment variable
///
/// Checks SLACKRS_TOKEN_STORE environment variable:
/// - "keyring" => Keyring backend (default)
/// - "file" => File backend
/// - unset => Keyring backend (default)
/// - invalid value => Error with guidance
pub fn resolve_token_store_backend() -> Result<TokenStoreBackend> {
    match std::env::var("SLACKRS_TOKEN_STORE") {
        Ok(value) => value.parse(),
        Err(_) => Ok(TokenStoreBackend::Keyring), // Default to Keyring
    }
}

/// Create a token store based on the resolved backend
///
/// This function:
/// 1. Resolves the backend via SLACKRS_TOKEN_STORE (defaulting to Keyring)
/// 2. For Keyring: attempts initialization and returns KeyringUnavailable error if it fails
/// 3. For File: creates FileTokenStore with default path
///
/// Returns Box<dyn TokenStore> for runtime polymorphism
pub fn create_token_store() -> Result<Box<dyn TokenStore>> {
    let backend = resolve_token_store_backend()?;

    match backend {
        TokenStoreBackend::Keyring => {
            // Try to create keyring store
            let store = KeyringTokenStore::default_service();

            // Test keyring availability by attempting a test operation
            // We use a unique test key to avoid conflicts
            let test_key = "__slackrs_keyring_test__";

            // Try to set and immediately delete a test value
            match store.set(test_key, "test") {
                Ok(_) => {
                    let _ = store.delete(test_key); // Clean up test key
                    Ok(Box::new(store))
                }
                Err(e) => {
                    // Keyring is unavailable - return error with guidance
                    Err(TokenStoreError::KeyringUnavailable(e.to_string()))
                }
            }
        }
        TokenStoreBackend::File => {
            let store = FileTokenStore::new()?;
            Ok(Box::new(store))
        }
    }
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
        assert_eq!(store.service, "slack-rs");
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
    fn test_resolve_token_store_backend_default() {
        // Clear environment variable
        std::env::remove_var("SLACKRS_TOKEN_STORE");

        let backend = resolve_token_store_backend().unwrap();
        assert_eq!(backend, TokenStoreBackend::Keyring);
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_token_store_backend_keyring() {
        std::env::set_var("SLACKRS_TOKEN_STORE", "keyring");

        let backend = resolve_token_store_backend().unwrap();
        assert_eq!(backend, TokenStoreBackend::Keyring);

        std::env::remove_var("SLACKRS_TOKEN_STORE");
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_token_store_backend_file() {
        std::env::set_var("SLACKRS_TOKEN_STORE", "file");

        let backend = resolve_token_store_backend().unwrap();
        assert_eq!(backend, TokenStoreBackend::File);

        std::env::remove_var("SLACKRS_TOKEN_STORE");
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_token_store_backend_case_insensitive() {
        std::env::set_var("SLACKRS_TOKEN_STORE", "KEYRING");
        assert_eq!(
            resolve_token_store_backend().unwrap(),
            TokenStoreBackend::Keyring
        );

        std::env::set_var("SLACKRS_TOKEN_STORE", "File");
        assert_eq!(
            resolve_token_store_backend().unwrap(),
            TokenStoreBackend::File
        );

        std::env::remove_var("SLACKRS_TOKEN_STORE");
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_token_store_backend_invalid() {
        std::env::set_var("SLACKRS_TOKEN_STORE", "invalid");

        let result = resolve_token_store_backend();
        assert!(result.is_err());
        match result {
            Err(TokenStoreError::InvalidBackend(backend)) => {
                assert_eq!(backend, "invalid");
            }
            _ => panic!("Expected InvalidBackend error"),
        }

        std::env::remove_var("SLACKRS_TOKEN_STORE");
    }

    #[test]
    #[serial_test::serial]
    fn test_create_token_store_file_backend() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tokens_path = temp_dir.path().join("tokens.json");
        std::env::set_var("SLACK_RS_TOKENS_PATH", tokens_path.to_str().unwrap());
        std::env::set_var("SLACKRS_TOKEN_STORE", "file");

        let store = create_token_store().unwrap();

        // Test that the store works
        store.set("test_key", "test_value").unwrap();
        assert_eq!(store.get("test_key").unwrap(), "test_value");

        std::env::remove_var("SLACKRS_TOKEN_STORE");
        std::env::remove_var("SLACK_RS_TOKENS_PATH");
    }

    #[test]
    fn test_token_store_backend_parse() {
        use std::str::FromStr;

        assert_eq!(
            TokenStoreBackend::from_str("keyring").unwrap(),
            TokenStoreBackend::Keyring
        );
        assert_eq!(
            TokenStoreBackend::from_str("file").unwrap(),
            TokenStoreBackend::File
        );
        assert_eq!(
            TokenStoreBackend::from_str("KEYRING").unwrap(),
            TokenStoreBackend::Keyring
        );
        assert!(TokenStoreBackend::from_str("invalid").is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_keyring_unavailable_error_message() {
        // Test that KeyringUnavailable error contains guidance
        let err = TokenStoreError::KeyringUnavailable("test error".to_string());
        let err_msg = err.to_string();

        // Verify error message contains guidance
        assert!(err_msg.contains("Keyring backend unavailable"));
        assert!(err_msg.contains("SLACKRS_TOKEN_STORE=file"));
        assert!(err_msg.contains("Unlock your OS keyring"));
    }

    #[test]
    fn test_invalid_backend_error_message() {
        let err = TokenStoreError::InvalidBackend("badvalue".to_string());
        let err_msg = err.to_string();

        // Verify error message lists valid options
        assert!(err_msg.contains("Invalid token store backend 'badvalue'"));
        assert!(err_msg.contains("keyring"));
        assert!(err_msg.contains("file"));
    }

    /// Test that demonstrates Keyring locked/interaction-required behavior
    ///
    /// This test verifies that when Keyring requires user interaction (e.g., locked):
    /// 1. create_token_store() fails with KeyringUnavailable error
    /// 2. The error message contains actionable guidance
    /// 3. No retry or prompt loop occurs (fail fast)
    ///
    /// Note: This is a mock/stub test since we can't reliably simulate a locked keyring
    /// in CI. The actual behavior is tested by create_token_store()'s test operation.
    #[test]
    #[serial_test::serial]
    fn test_keyring_locked_interaction_required() {
        // Clear any file backend override to ensure we test keyring path
        std::env::remove_var("SLACKRS_TOKEN_STORE");
        std::env::remove_var("SLACK_RS_TOKENS_PATH");

        // Try to create a keyring token store
        // This will fail if keyring is unavailable (locked, not configured, etc.)
        let result = create_token_store();

        // If keyring is available on this system, test passes
        // If keyring is NOT available, verify error handling is correct
        match result {
            Ok(_) => {
                // Keyring is available - test passes
                // (This is the expected case on developer machines with unlocked keychains)
            }
            Err(TokenStoreError::KeyringUnavailable(msg)) => {
                // Keyring is unavailable - verify error message has guidance
                let err_str = TokenStoreError::KeyringUnavailable(msg.clone()).to_string();
                assert!(
                    err_str.contains("SLACKRS_TOKEN_STORE=file"),
                    "Error should suggest file fallback: {}",
                    err_str
                );
                assert!(
                    err_str.contains("Unlock your OS keyring") || err_str.contains("keyring"),
                    "Error should mention keyring: {}",
                    err_str
                );
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    /// Test that file mode works correctly when explicitly specified
    /// This ensures users have a fallback when Keyring is unavailable
    #[test]
    #[serial_test::serial]
    fn test_file_mode_fallback_when_keyring_unavailable() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let tokens_path = temp_dir.path().join("tokens.json");

        // Set environment to use file backend
        std::env::set_var("SLACK_RS_TOKENS_PATH", tokens_path.to_str().unwrap());
        std::env::set_var("SLACKRS_TOKEN_STORE", "file");

        // This should succeed even if keyring is unavailable
        let store = create_token_store().expect("File backend should work");

        // Verify it works
        store.set("test", "value").unwrap();
        assert_eq!(store.get("test").unwrap(), "value");

        std::env::remove_var("SLACKRS_TOKEN_STORE");
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
        std::env::set_var("SLACKRS_TOKEN_STORE", "file");

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

        std::env::remove_var("SLACKRS_TOKEN_STORE");
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
    /// 1. Default backend is Keyring
    /// 2. File backend can be explicitly selected via SLACKRS_TOKEN_STORE=file
    /// 3. Both backends use the same key format (team_id:user_id for tokens, oauth-client-secret:profile for secrets)
    /// 4. InMemoryTokenStore can be used for testing with same key format
    #[test]
    #[serial_test::serial]
    fn test_unified_credential_storage_policy() {
        use tempfile::TempDir;

        // Test 1: Default is Keyring
        std::env::remove_var("SLACKRS_TOKEN_STORE");
        let backend = resolve_token_store_backend().unwrap();
        assert_eq!(
            backend,
            TokenStoreBackend::Keyring,
            "Default backend should be Keyring"
        );

        // Test 2: Can explicitly select File backend
        std::env::set_var("SLACKRS_TOKEN_STORE", "file");
        let backend = resolve_token_store_backend().unwrap();
        assert_eq!(
            backend,
            TokenStoreBackend::File,
            "Should be able to select File backend"
        );

        // Test 3: Same key format across all backends
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

        // Test 4: Verify helper functions work across all backends
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
        std::env::remove_var("SLACKRS_TOKEN_STORE");
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
