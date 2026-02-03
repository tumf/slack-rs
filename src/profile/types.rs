use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Profile configuration data structure
/// Contains non-secret information only
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub team_id: String,
    pub user_id: String,
    pub team_name: Option<String>,
    pub user_name: Option<String>,
}

/// Root configuration structure with versioning for future migration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfilesConfig {
    pub version: u32,
    pub profiles: HashMap<String, Profile>,
}

impl ProfilesConfig {
    pub fn new() -> Self {
        Self {
            version: 1,
            profiles: HashMap::new(),
        }
    }

    /// Get profile by name
    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// Add or update a profile
    pub fn set(&mut self, name: String, profile: Profile) {
        self.profiles.insert(name, profile);
    }

    /// Remove a profile
    pub fn remove(&mut self, name: &str) -> Option<Profile> {
        self.profiles.remove(name)
    }

    /// List all profile names
    pub fn list_names(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }
}

impl Default for ProfilesConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiles_config_new() {
        let config = ProfilesConfig::new();
        assert_eq!(config.version, 1);
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn test_profiles_config_get_set() {
        let mut config = ProfilesConfig::new();
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Test Team".to_string()),
            user_name: Some("Test User".to_string()),
        };

        config.set("default".to_string(), profile.clone());
        assert_eq!(config.get("default"), Some(&profile));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_profiles_config_remove() {
        let mut config = ProfilesConfig::new();
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: None,
            user_name: None,
        };

        config.set("test".to_string(), profile.clone());
        let removed = config.remove("test");
        assert_eq!(removed, Some(profile));
        assert_eq!(config.get("test"), None);
    }

    #[test]
    fn test_profiles_config_list_names() {
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
                team_name: None,
                user_name: None,
            },
        );

        let mut names = config.list_names();
        names.sort();
        assert_eq!(names, vec!["profile1", "profile2"]);
    }

    #[test]
    fn test_profile_serialization() {
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Test Team".to_string()),
            user_name: Some("Test User".to_string()),
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, deserialized);
    }

    #[test]
    fn test_profiles_config_serialization() {
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

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: ProfilesConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}
