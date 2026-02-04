use crate::profile::types::ProfilesConfig;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Config directory not found")]
    ConfigDirNotFound,
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Get the legacy config file path (slack-cli) for migration purposes
fn legacy_config_path() -> Result<PathBuf> {
    directories::ProjectDirs::from("", "", "slack-cli")
        .map(|dirs| dirs.config_dir().join("profiles.json"))
        .ok_or(StorageError::ConfigDirNotFound)
}

/// Get the default config file path using directories crate
pub fn default_config_path() -> Result<PathBuf> {
    directories::ProjectDirs::from("", "", "slack-rs")
        .map(|dirs| dirs.config_dir().join("profiles.json"))
        .ok_or(StorageError::ConfigDirNotFound)
}

/// Migrate legacy config file to new path if needed
/// This function is only called when using the default config path
fn migrate_legacy_config_internal() -> Result<bool> {
    // Get new default path
    let new_path = default_config_path()?;

    // If new path already exists, no migration needed
    if new_path.exists() {
        return Ok(false);
    }

    // Try to get legacy path
    let legacy_path = match legacy_config_path() {
        Ok(path) => path,
        Err(_) => return Ok(false),
    };

    // If legacy path doesn't exist, no migration needed
    if !legacy_path.exists() {
        return Ok(false);
    }

    // Create parent directory for new path if it doesn't exist
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Try to rename (move) the file first
    match fs::rename(&legacy_path, &new_path) {
        Ok(_) => Ok(true),
        Err(_) => {
            // If rename fails (e.g., different filesystems), copy and keep the old file
            let content = fs::read_to_string(&legacy_path)?;
            fs::write(&new_path, content)?;
            Ok(true)
        }
    }
}

/// Migrate legacy config file for a specific path (used for testing)
/// Returns true if migration was performed
#[cfg(test)]
fn migrate_legacy_config(legacy_path: &Path, new_path: &Path) -> Result<bool> {
    // If new path already exists, no migration needed
    if new_path.exists() {
        return Ok(false);
    }

    // If legacy path doesn't exist, no migration needed
    if !legacy_path.exists() {
        return Ok(false);
    }

    // Create parent directory for new path if it doesn't exist
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Try to rename (move) the file first
    match fs::rename(legacy_path, new_path) {
        Ok(_) => Ok(true),
        Err(_) => {
            // If rename fails (e.g., different filesystems), copy and keep the old file
            let content = fs::read_to_string(legacy_path)?;
            fs::write(new_path, content)?;
            Ok(true)
        }
    }
}

/// Load profiles config from a file
pub fn load_config(path: &Path) -> Result<ProfilesConfig> {
    // Try to migrate legacy config if this is the default path
    // Only attempt migration when using default_config_path
    if let Ok(default_path) = default_config_path() {
        if path == default_path {
            let _ = migrate_legacy_config_internal();
        }
    }

    if !path.exists() {
        return Ok(ProfilesConfig::new());
    }

    let content = fs::read_to_string(path)?;
    let config: ProfilesConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// Save profiles config to a file
pub fn save_config(path: &Path, config: &ProfilesConfig) -> Result<()> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::types::Profile;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("profiles.json");

        let mut config = ProfilesConfig::new();
        config.set(
            "default".to_string(),
            Profile {
                team_id: "T123".to_string(),
                user_id: "U456".to_string(),
                team_name: Some("Test Team".to_string()),
                user_name: Some("Test User".to_string()),
                client_id: None,
                redirect_uri: None,
                scopes: None,
            },
        );

        // Save config
        save_config(&config_path, &config).unwrap();
        assert!(config_path.exists());

        // Load config
        let loaded = load_config(&config_path).unwrap();
        assert_eq!(config, loaded);
    }

    #[test]
    fn test_load_nonexistent_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.json");

        let loaded = load_config(&config_path).unwrap();
        assert_eq!(loaded, ProfilesConfig::new());
    }

    #[test]
    fn test_save_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nested/dir/profiles.json");

        let config = ProfilesConfig::new();
        save_config(&config_path, &config).unwrap();

        assert!(config_path.exists());
        assert!(config_path.parent().unwrap().exists());
    }

    #[test]
    fn test_load_save_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("profiles.json");

        let mut config = ProfilesConfig::new();
        config.set(
            "profile1".to_string(),
            Profile {
                team_id: "T1".to_string(),
                user_id: "U1".to_string(),
                team_name: None,
                user_name: None,
                client_id: None,
                redirect_uri: None,
                scopes: None,
            },
        );
        config.set(
            "profile2".to_string(),
            Profile {
                team_id: "T2".to_string(),
                user_id: "U2".to_string(),
                team_name: Some("Team 2".to_string()),
                user_name: Some("User 2".to_string()),
                client_id: None,
                redirect_uri: None,
                scopes: None,
            },
        );

        save_config(&config_path, &config).unwrap();
        let loaded = load_config(&config_path).unwrap();
        assert_eq!(config, loaded);
    }

    #[test]
    fn test_default_config_path() {
        // Just verify it doesn't panic and returns something
        let result = default_config_path();
        match result {
            Ok(path) => {
                assert!(path.to_string_lossy().contains("slack-rs"));
                assert!(path.to_string_lossy().contains("profiles.json"));
            }
            Err(StorageError::ConfigDirNotFound) => {
                // This might happen in some test environments
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_migrate_legacy_config_path() {
        let temp_dir = TempDir::new().unwrap();
        let legacy_path = temp_dir.path().join("legacy").join("profiles.json");
        let new_path = temp_dir.path().join("new").join("profiles.json");

        // Create legacy config
        let mut config = ProfilesConfig::new();
        config.set(
            "legacy".to_string(),
            Profile {
                team_id: "T123".to_string(),
                user_id: "U456".to_string(),
                team_name: Some("Legacy Team".to_string()),
                user_name: Some("Legacy User".to_string()),
                client_id: None,
                redirect_uri: None,
                scopes: None,
            },
        );
        fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
        save_config(&legacy_path, &config).unwrap();
        assert!(legacy_path.exists());

        // Perform migration
        let migrated = migrate_legacy_config(&legacy_path, &new_path).unwrap();
        assert!(migrated);
        assert!(new_path.exists());

        // Verify migrated content
        let loaded = load_config(&new_path).unwrap();
        assert_eq!(config, loaded);

        // Test that migration is skipped if new path exists
        let legacy_path2 = temp_dir.path().join("legacy2").join("profiles.json");
        fs::create_dir_all(legacy_path2.parent().unwrap()).unwrap();
        save_config(&legacy_path2, &config).unwrap();

        let migrated_again = migrate_legacy_config(&legacy_path2, &new_path).unwrap();
        assert!(!migrated_again);
    }

    #[test]
    fn test_load_config_with_migration() {
        let temp_dir = TempDir::new().unwrap();
        let legacy_path = temp_dir.path().join("legacy").join("profiles.json");
        let new_path = temp_dir.path().join("new").join("profiles.json");

        // Create legacy config
        let mut config = ProfilesConfig::new();
        config.set(
            "test".to_string(),
            Profile {
                team_id: "T999".to_string(),
                user_id: "U888".to_string(),
                team_name: None,
                user_name: None,
                client_id: None,
                redirect_uri: None,
                scopes: None,
            },
        );
        fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
        save_config(&legacy_path, &config).unwrap();

        // Manually trigger migration by calling migrate_legacy_config
        migrate_legacy_config(&legacy_path, &new_path).unwrap();

        // Load from new path should work
        let loaded = load_config(&new_path).unwrap();
        assert_eq!(config, loaded);
    }
}
