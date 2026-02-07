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
/// Stores tokens in ~/.local/share/slack-rs/tokens.json with restricted permissions (0600)
#[derive(Debug, Clone)]
pub struct FileTokenStore {
    file_path: PathBuf,
    tokens: Arc<Mutex<HashMap<String, String>>>,
}

impl FileTokenStore {
    /// Create a new FileTokenStore with the default path (~/.local/share/slack-rs/tokens.json)
    pub fn new() -> Result<Self> {
        let file_path = Self::default_path()?;
        Self::with_path(file_path)
    }

    /// Create a FileTokenStore with a custom path
    pub fn with_path(file_path: PathBuf) -> Result<Self> {
        Self::with_path_and_migration(file_path, None)
    }

    /// Create a FileTokenStore with a custom path and optional migration source
    /// Internal method that allows tests to specify a custom old path for migration
    fn with_path_and_migration(file_path: PathBuf, old_path: Option<PathBuf>) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                TokenStoreError::IoError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Auto-migrate from old path if needed (only when using default path)
        if std::env::var("SLACK_RS_TOKENS_PATH").is_err() {
            if let Some(old) = old_path {
                Self::migrate_from_path(&old, &file_path)?;
            } else {
                Self::migrate_from_old_path_if_needed(&file_path)?;
            }
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
    /// Respects XDG_DATA_HOME when set
    pub fn default_path() -> Result<PathBuf> {
        // Priority 1: Check for environment variable override (useful for testing)
        if let Ok(path) = std::env::var("SLACK_RS_TOKENS_PATH") {
            return Ok(PathBuf::from(path));
        }

        // Priority 2: Check for XDG_DATA_HOME
        if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
            // Guard against empty or whitespace-only values
            let trimmed = xdg_data_home.trim();
            if !trimmed.is_empty() {
                let xdg_path = PathBuf::from(trimmed);
                // Ensure the path is absolute to avoid confusion
                if xdg_path.is_absolute() {
                    let data_dir = xdg_path.join("slack-rs");
                    return Ok(data_dir.join("tokens.json"));
                }
            }
        }

        // Priority 3: Fallback to ~/.local/share/slack-rs/tokens.json
        let home = directories::BaseDirs::new()
            .ok_or_else(|| {
                TokenStoreError::IoError("Failed to determine home directory".to_string())
            })?
            .home_dir()
            .to_path_buf();

        let data_dir = home.join(".local").join("share").join("slack-rs");
        Ok(data_dir.join("tokens.json"))
    }

    /// Get the old config path for migration purposes
    fn old_config_path() -> Result<PathBuf> {
        let home = directories::BaseDirs::new()
            .ok_or_else(|| {
                TokenStoreError::IoError("Failed to determine home directory".to_string())
            })?
            .home_dir()
            .to_path_buf();

        // Old path: ~/.config/slack-rs/tokens.json
        let config_dir = home.join(".config").join("slack-rs");
        Ok(config_dir.join("tokens.json"))
    }

    /// Migrate from old path to new path if needed
    /// Only runs when new path doesn't exist and old path does exist
    fn migrate_from_old_path_if_needed(new_path: &Path) -> Result<()> {
        // Get the old path, but don't fail if we can't determine it
        let old_path = match Self::old_config_path() {
            Ok(path) => path,
            Err(_) => return Ok(()), // Can't determine old path, skip migration
        };

        Self::migrate_from_path(&old_path, new_path)
    }

    /// Migrate tokens from a specific old path to a new path
    /// Only runs when new path doesn't exist and old path does exist
    fn migrate_from_path(old_path: &Path, new_path: &Path) -> Result<()> {
        // Skip migration if new path already exists
        if new_path.exists() {
            return Ok(());
        }

        // Skip migration if old path doesn't exist
        if !old_path.exists() {
            return Ok(());
        }

        // Copy old file to new location
        fs::copy(old_path, new_path).map_err(|e| {
            TokenStoreError::IoError(format!("Failed to migrate tokens from old path: {}", e))
        })?;

        // Set file permissions to 0600 on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(new_path, permissions).map_err(|e| {
                TokenStoreError::IoError(format!(
                    "Failed to set file permissions during migration: {}",
                    e
                ))
            })?;
        }

        Ok(())
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

        // Convert HashMap to BTreeMap for deterministic key ordering
        use std::collections::BTreeMap;
        let sorted_tokens: BTreeMap<_, _> = tokens.iter().collect();

        let content = serde_json::to_string_pretty(&sorted_tokens).map_err(|e| {
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
    /// Verifies that FileTokenStore uses ~/.local/share/slack-rs/tokens.json by default
    #[test]
    #[serial_test::serial]
    fn test_file_token_store_default_path() {
        // Clear environment override to test actual default
        std::env::remove_var("SLACK_RS_TOKENS_PATH");

        let default_path = FileTokenStore::default_path().unwrap();
        let path_str = default_path.to_string_lossy();

        // Should contain .local/share/slack-rs/tokens.json
        assert!(
            path_str.contains(".local/share/slack-rs/tokens.json")
                || path_str.contains(".local\\share\\slack-rs\\tokens.json"),
            "Default path should be ~/.local/share/slack-rs/tokens.json, got: {}",
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

    /// Test migration from old path to new path
    #[test]
    #[serial_test::serial]
    fn test_migration_from_old_to_new_path() {
        use tempfile::TempDir;

        // Clear environment variables to ensure migration logic runs
        std::env::remove_var("SLACK_RS_TOKENS_PATH");
        std::env::remove_var("XDG_DATA_HOME");

        let temp_dir = TempDir::new().unwrap();

        // Create old-style directory structure
        let old_config_dir = temp_dir.path().join(".config").join("slack-rs");
        fs::create_dir_all(&old_config_dir).unwrap();
        let old_path = old_config_dir.join("tokens.json");

        // Create new-style directory structure
        let new_data_dir = temp_dir
            .path()
            .join(".local")
            .join("share")
            .join("slack-rs");
        fs::create_dir_all(&new_data_dir).unwrap();
        let new_path = new_data_dir.join("tokens.json");

        // Write test data to old path
        let mut old_tokens = HashMap::new();
        old_tokens.insert("T123:U456".to_string(), "xoxb-old-token".to_string());
        old_tokens.insert(
            "oauth-client-secret:default".to_string(),
            "old-secret".to_string(),
        );
        let old_content = serde_json::to_string_pretty(&old_tokens).unwrap();
        fs::write(&old_path, old_content).unwrap();

        // Verify old file exists and new file doesn't
        assert!(old_path.exists());
        assert!(!new_path.exists());

        // Create FileTokenStore with new path (should trigger migration)
        let store =
            FileTokenStore::with_path_and_migration(new_path.clone(), Some(old_path.clone()))
                .unwrap();

        // Verify new file exists after migration
        assert!(new_path.exists());

        // Verify content was migrated correctly
        assert_eq!(store.get("T123:U456").unwrap(), "xoxb-old-token");
        assert_eq!(
            store.get("oauth-client-secret:default").unwrap(),
            "old-secret"
        );

        // Verify file permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&new_path).unwrap();
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o600);
        }

        // Verify old file still exists (not deleted)
        assert!(old_path.exists());
    }

    /// Test that migration doesn't happen when new path already exists
    #[test]
    fn test_no_migration_when_new_path_exists() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create old-style directory structure
        let old_config_dir = temp_dir.path().join(".config").join("slack-rs");
        fs::create_dir_all(&old_config_dir).unwrap();
        let old_path = old_config_dir.join("tokens.json");

        // Create new-style directory structure
        let new_data_dir = temp_dir
            .path()
            .join(".local")
            .join("share")
            .join("slack-rs");
        fs::create_dir_all(&new_data_dir).unwrap();
        let new_path = new_data_dir.join("tokens.json");

        // Write different data to both paths
        let mut old_tokens = HashMap::new();
        old_tokens.insert("old:key".to_string(), "old-value".to_string());
        fs::write(
            &old_path,
            serde_json::to_string_pretty(&old_tokens).unwrap(),
        )
        .unwrap();

        let mut new_tokens = HashMap::new();
        new_tokens.insert("new:key".to_string(), "new-value".to_string());
        fs::write(
            &new_path,
            serde_json::to_string_pretty(&new_tokens).unwrap(),
        )
        .unwrap();

        // Create FileTokenStore with new path
        let store = FileTokenStore::with_path(new_path.clone()).unwrap();

        // Verify new path content is preserved (no migration happened)
        assert_eq!(store.get("new:key").unwrap(), "new-value");
        assert!(store.get("old:key").is_err());
    }

    /// Test that migration doesn't happen when old path doesn't exist
    #[test]
    fn test_no_migration_when_old_path_missing() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create new-style directory structure only
        let new_data_dir = temp_dir
            .path()
            .join(".local")
            .join("share")
            .join("slack-rs");
        fs::create_dir_all(&new_data_dir).unwrap();
        let new_path = new_data_dir.join("tokens.json");

        // Create FileTokenStore with new path (should not fail)
        let store = FileTokenStore::with_path(new_path.clone()).unwrap();

        // Should work normally
        store.set("test:key", "test-value").unwrap();
        assert_eq!(store.get("test:key").unwrap(), "test-value");
        assert!(new_path.exists());
    }

    /// Test that migration doesn't happen when SLACK_RS_TOKENS_PATH is set
    #[test]
    #[serial_test::serial]
    fn test_no_migration_with_env_override() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create old-style directory structure
        let old_config_dir = temp_dir.path().join(".config").join("slack-rs");
        fs::create_dir_all(&old_config_dir).unwrap();
        let old_path = old_config_dir.join("tokens.json");

        // Write test data to old path
        let mut old_tokens = HashMap::new();
        old_tokens.insert("old:key".to_string(), "old-value".to_string());
        fs::write(
            &old_path,
            serde_json::to_string_pretty(&old_tokens).unwrap(),
        )
        .unwrap();

        // Set custom path via environment variable
        let custom_path = temp_dir.path().join("custom-tokens.json");
        std::env::set_var("SLACK_RS_TOKENS_PATH", custom_path.to_str().unwrap());

        // Create FileTokenStore (should use custom path, no migration)
        let store = FileTokenStore::new().unwrap();

        // Verify custom path is used and old data is not migrated
        store.set("new:key", "new-value").unwrap();
        assert_eq!(store.get("new:key").unwrap(), "new-value");
        assert!(store.get("old:key").is_err());
        assert!(custom_path.exists());

        std::env::remove_var("SLACK_RS_TOKENS_PATH");
    }

    /// Test deterministic serialization with different insertion orders
    /// Regression test for Issue #24
    #[test]
    fn test_deterministic_serialization_different_insertion_orders() {
        use tempfile::TempDir;

        // Create separate temp directories for each store to avoid state sharing
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        // Create first store and insert keys in order: A, B, C
        let file_path_1 = temp_dir1.path().join("tokens.json");
        let store1 = FileTokenStore::with_path(file_path_1.clone()).unwrap();
        store1.set("key_a", "value_a").unwrap();
        store1.set("key_b", "value_b").unwrap();
        store1.set("key_c", "value_c").unwrap();

        // Create second store and insert keys in order: C, A, B
        let file_path_2 = temp_dir2.path().join("tokens.json");
        let store2 = FileTokenStore::with_path(file_path_2.clone()).unwrap();
        store2.set("key_c", "value_c").unwrap();
        store2.set("key_a", "value_a").unwrap();
        store2.set("key_b", "value_b").unwrap();

        // Read both files and compare content
        let content1 = fs::read_to_string(&file_path_1).unwrap();
        let content2 = fs::read_to_string(&file_path_2).unwrap();

        // Content should be identical despite different insertion orders
        assert_eq!(content1, content2,
            "Files should have identical content regardless of insertion order.\nFile1:\n{}\nFile2:\n{}",
            content1, content2);

        // Verify keys are sorted alphabetically in the output
        let content_lines: Vec<&str> = content1.lines().collect();
        let key_a_idx = content_lines
            .iter()
            .position(|l| l.contains("key_a"))
            .unwrap();
        let key_b_idx = content_lines
            .iter()
            .position(|l| l.contains("key_b"))
            .unwrap();
        let key_c_idx = content_lines
            .iter()
            .position(|l| l.contains("key_c"))
            .unwrap();

        assert!(key_a_idx < key_b_idx, "key_a should appear before key_b");
        assert!(key_b_idx < key_c_idx, "key_b should appear before key_c");
    }

    /// Test no diff on consecutive saves with unchanged content
    /// Regression test for Issue #24
    #[test]
    fn test_no_diff_on_consecutive_saves() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("tokens.json");
        let store = FileTokenStore::with_path(file_path.clone()).unwrap();

        // First save
        store.set("key1", "value1").unwrap();
        store.set("key2", "value2").unwrap();
        let content_after_first_save = fs::read_to_string(&file_path).unwrap();

        // Second save with no changes (re-save existing data)
        store.set("key1", "value1").unwrap();
        let content_after_second_save = fs::read_to_string(&file_path).unwrap();

        // Third save with no changes
        store.set("key2", "value2").unwrap();
        let content_after_third_save = fs::read_to_string(&file_path).unwrap();

        // All saves should produce identical content
        assert_eq!(
            content_after_first_save, content_after_second_save,
            "Second save should not change file content"
        );
        assert_eq!(
            content_after_second_save, content_after_third_save,
            "Third save should not change file content"
        );
    }

    /// Test existing key format compatibility (regression test)
    #[test]
    fn test_existing_key_format_compatibility() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("tokens.json");
        let store = FileTokenStore::with_path(file_path.clone()).unwrap();

        // Test team_id:user_id format
        let token_key = make_token_key("T123", "U456");
        assert_eq!(token_key, "T123:U456");
        store.set(&token_key, "xoxb-test-token").unwrap();
        assert_eq!(store.get(&token_key).unwrap(), "xoxb-test-token");

        // Test oauth-client-secret:profile_name format
        let secret_key = make_oauth_client_secret_key("default");
        assert_eq!(secret_key, "oauth-client-secret:default");
        store.set(&secret_key, "test-secret").unwrap();
        assert_eq!(store.get(&secret_key).unwrap(), "test-secret");

        // Verify helper functions work correctly
        store_oauth_client_secret(&store, "profile1", "secret1").unwrap();
        assert_eq!(
            get_oauth_client_secret(&store, "profile1").unwrap(),
            "secret1"
        );

        delete_oauth_client_secret(&store, "profile1").unwrap();
        assert!(get_oauth_client_secret(&store, "profile1").is_err());

        // Verify file content has correct keys
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("T123:U456"));
        assert!(content.contains("oauth-client-secret:default"));
    }

    /// Test XDG_DATA_HOME resolution (when set to valid absolute path)
    #[test]
    #[serial_test::serial]
    fn test_xdg_data_home_resolution() {
        use tempfile::TempDir;

        // Clear SLACK_RS_TOKENS_PATH to ensure XDG_DATA_HOME is tested
        std::env::remove_var("SLACK_RS_TOKENS_PATH");

        let temp_dir = TempDir::new().unwrap();
        let xdg_data_home = temp_dir.path().to_str().unwrap();
        std::env::set_var("XDG_DATA_HOME", xdg_data_home);

        let path = FileTokenStore::default_path().unwrap();
        let expected = temp_dir.path().join("slack-rs").join("tokens.json");

        assert_eq!(
            path, expected,
            "XDG_DATA_HOME should resolve to $XDG_DATA_HOME/slack-rs/tokens.json"
        );

        std::env::remove_var("XDG_DATA_HOME");
    }

    /// Test that SLACK_RS_TOKENS_PATH takes priority over XDG_DATA_HOME
    #[test]
    #[serial_test::serial]
    fn test_slack_rs_tokens_path_priority_over_xdg() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let custom_path = temp_dir.path().join("custom-tokens.json");
        let xdg_data_home = temp_dir.path().join("xdg-data");

        // Set both environment variables
        std::env::set_var("SLACK_RS_TOKENS_PATH", custom_path.to_str().unwrap());
        std::env::set_var("XDG_DATA_HOME", xdg_data_home.to_str().unwrap());

        let path = FileTokenStore::default_path().unwrap();

        // SLACK_RS_TOKENS_PATH should win
        assert_eq!(
            path, custom_path,
            "SLACK_RS_TOKENS_PATH should take priority over XDG_DATA_HOME"
        );

        std::env::remove_var("SLACK_RS_TOKENS_PATH");
        std::env::remove_var("XDG_DATA_HOME");
    }

    /// Test fallback to ~/.local/share when XDG_DATA_HOME is not set
    #[test]
    #[serial_test::serial]
    fn test_fallback_when_xdg_data_home_not_set() {
        // Clear both environment variables
        std::env::remove_var("SLACK_RS_TOKENS_PATH");
        std::env::remove_var("XDG_DATA_HOME");

        let path = FileTokenStore::default_path().unwrap();
        let path_str = path.to_string_lossy();

        // Should fallback to ~/.local/share/slack-rs/tokens.json
        assert!(
            path_str.contains(".local/share/slack-rs/tokens.json")
                || path_str.contains(".local\\share\\slack-rs\\tokens.json"),
            "Should fallback to ~/.local/share/slack-rs/tokens.json when XDG_DATA_HOME is not set, got: {}",
            path_str
        );
    }

    /// Test that empty XDG_DATA_HOME falls back to default
    #[test]
    #[serial_test::serial]
    fn test_empty_xdg_data_home_fallback() {
        // Clear SLACK_RS_TOKENS_PATH
        std::env::remove_var("SLACK_RS_TOKENS_PATH");

        // Set XDG_DATA_HOME to empty string
        std::env::set_var("XDG_DATA_HOME", "");

        let path = FileTokenStore::default_path().unwrap();
        let path_str = path.to_string_lossy();

        // Should fallback to ~/.local/share/slack-rs/tokens.json
        assert!(
            path_str.contains(".local/share/slack-rs/tokens.json")
                || path_str.contains(".local\\share\\slack-rs\\tokens.json"),
            "Empty XDG_DATA_HOME should fallback to ~/.local/share/slack-rs/tokens.json, got: {}",
            path_str
        );

        std::env::remove_var("XDG_DATA_HOME");
    }

    /// Test that whitespace-only XDG_DATA_HOME falls back to default
    #[test]
    #[serial_test::serial]
    fn test_whitespace_xdg_data_home_fallback() {
        // Clear SLACK_RS_TOKENS_PATH
        std::env::remove_var("SLACK_RS_TOKENS_PATH");

        // Set XDG_DATA_HOME to whitespace
        std::env::set_var("XDG_DATA_HOME", "   ");

        let path = FileTokenStore::default_path().unwrap();
        let path_str = path.to_string_lossy();

        // Should fallback to ~/.local/share/slack-rs/tokens.json
        assert!(
            path_str.contains(".local/share/slack-rs/tokens.json")
                || path_str.contains(".local\\share\\slack-rs\\tokens.json"),
            "Whitespace XDG_DATA_HOME should fallback to ~/.local/share/slack-rs/tokens.json, got: {}",
            path_str
        );

        std::env::remove_var("XDG_DATA_HOME");
    }

    /// Test that relative XDG_DATA_HOME path falls back to default
    #[test]
    #[serial_test::serial]
    fn test_relative_xdg_data_home_fallback() {
        // Clear SLACK_RS_TOKENS_PATH
        std::env::remove_var("SLACK_RS_TOKENS_PATH");

        // Set XDG_DATA_HOME to relative path
        std::env::set_var("XDG_DATA_HOME", "relative/path");

        let path = FileTokenStore::default_path().unwrap();
        let path_str = path.to_string_lossy();

        // Should fallback to ~/.local/share/slack-rs/tokens.json
        assert!(
            path_str.contains(".local/share/slack-rs/tokens.json")
                || path_str.contains(".local\\share\\slack-rs\\tokens.json"),
            "Relative XDG_DATA_HOME should fallback to ~/.local/share/slack-rs/tokens.json, got: {}",
            path_str
        );

        std::env::remove_var("XDG_DATA_HOME");
    }
}
