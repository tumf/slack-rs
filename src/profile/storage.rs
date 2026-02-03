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

/// Get the default config file path using directories crate
pub fn default_config_path() -> Result<PathBuf> {
    directories::ProjectDirs::from("", "", "slackcli")
        .map(|dirs| dirs.config_dir().join("profiles.json"))
        .ok_or(StorageError::ConfigDirNotFound)
}

/// Load profiles config from a file
pub fn load_config(path: &Path) -> Result<ProfilesConfig> {
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
            },
        );
        config.set(
            "profile2".to_string(),
            Profile {
                team_id: "T2".to_string(),
                user_id: "U2".to_string(),
                team_name: Some("Team 2".to_string()),
                user_name: Some("User 2".to_string()),
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
                assert!(path.to_string_lossy().contains("slackcli"));
                assert!(path.to_string_lossy().contains("profiles.json"));
            }
            Err(StorageError::ConfigDirNotFound) => {
                // This might happen in some test environments
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
}
