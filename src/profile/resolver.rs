use crate::profile::storage::{load_config, Result as StorageResult};
use crate::profile::types::{Profile, ProfilesConfig};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    #[error("Storage error: {0}")]
    Storage(#[from] crate::profile::storage::StorageError),
}

pub type Result<T> = std::result::Result<T, ResolverError>;

/// Resolve a profile name to (team_id, user_id)
pub fn resolve_profile(config_path: &Path, profile_name: &str) -> Result<(String, String)> {
    let config = load_config(config_path)?;
    let profile = config
        .get(profile_name)
        .ok_or_else(|| ResolverError::ProfileNotFound(profile_name.to_string()))?;

    Ok((profile.team_id.clone(), profile.user_id.clone()))
}

/// Resolve a profile name to the full Profile object
pub fn resolve_profile_full(config_path: &Path, profile_name: &str) -> Result<Profile> {
    let config = load_config(config_path)?;
    config
        .get(profile_name)
        .cloned()
        .ok_or_else(|| ResolverError::ProfileNotFound(profile_name.to_string()))
}

/// Get all profiles from config
pub fn list_profiles(config_path: &Path) -> StorageResult<ProfilesConfig> {
    load_config(config_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::storage::save_config;
    use crate::profile::types::Profile;
    use tempfile::TempDir;

    fn setup_test_config() -> (TempDir, std::path::PathBuf) {
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
        config.set(
            "work".to_string(),
            Profile {
                team_id: "T789".to_string(),
                user_id: "U012".to_string(),
                team_name: Some("Work Team".to_string()),
                user_name: Some("Work User".to_string()),
                client_id: None,
                redirect_uri: None,
                scopes: None,
            },
        );

        save_config(&config_path, &config).unwrap();
        (temp_dir, config_path)
    }

    #[test]
    fn test_resolve_profile_existing() {
        let (_temp_dir, config_path) = setup_test_config();

        let result = resolve_profile(&config_path, "default");
        assert!(result.is_ok());
        let (team_id, user_id) = result.unwrap();
        assert_eq!(team_id, "T123");
        assert_eq!(user_id, "U456");
    }

    #[test]
    fn test_resolve_profile_nonexistent() {
        let (_temp_dir, config_path) = setup_test_config();

        let result = resolve_profile(&config_path, "nonexistent");
        assert!(result.is_err());
        match result {
            Err(ResolverError::ProfileNotFound(name)) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected ProfileNotFound error"),
        }
    }

    #[test]
    fn test_resolve_profile_full() {
        let (_temp_dir, config_path) = setup_test_config();

        let result = resolve_profile_full(&config_path, "default");
        assert!(result.is_ok());
        let profile = result.unwrap();
        assert_eq!(profile.team_id, "T123");
        assert_eq!(profile.user_id, "U456");
        assert_eq!(profile.team_name, Some("Test Team".to_string()));
        assert_eq!(profile.user_name, Some("Test User".to_string()));
    }

    #[test]
    fn test_resolve_profile_multiple() {
        let (_temp_dir, config_path) = setup_test_config();

        let (team_id1, user_id1) = resolve_profile(&config_path, "default").unwrap();
        assert_eq!(team_id1, "T123");
        assert_eq!(user_id1, "U456");

        let (team_id2, user_id2) = resolve_profile(&config_path, "work").unwrap();
        assert_eq!(team_id2, "T789");
        assert_eq!(user_id2, "U012");
    }

    #[test]
    fn test_list_profiles() {
        let (_temp_dir, config_path) = setup_test_config();

        let config = list_profiles(&config_path).unwrap();
        assert_eq!(config.profiles.len(), 2);
        assert!(config.get("default").is_some());
        assert!(config.get("work").is_some());
    }

    #[test]
    fn test_list_profiles_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("empty_profiles.json");

        let config = list_profiles(&config_path).unwrap();
        assert_eq!(config.profiles.len(), 0);
    }
}
