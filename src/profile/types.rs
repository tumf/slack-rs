use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("Profile name already exists: {0}")]
    DuplicateName(String),
}

/// Profile configuration data structure
/// Contains non-secret information only
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub team_id: String,
    pub user_id: String,
    pub team_name: Option<String>,
    pub user_name: Option<String>,
    /// OAuth client ID for this profile (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// OAuth redirect URI for this profile (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    /// OAuth scopes for this profile (legacy field, migrated to bot_scopes for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    /// Bot OAuth scopes (new field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_scopes: Option<Vec<String>>,
    /// User OAuth scopes (new field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_scopes: Option<Vec<String>>,
}

impl Profile {
    /// Get bot scopes, falling back to legacy scopes field if bot_scopes is None
    pub fn get_bot_scopes(&self) -> Option<Vec<String>> {
        self.bot_scopes.clone().or_else(|| self.scopes.clone())
    }

    /// Get user scopes
    pub fn get_user_scopes(&self) -> Option<Vec<String>> {
        self.user_scopes.clone()
    }

    /// Create a new profile with bot and user scopes
    #[allow(clippy::too_many_arguments)]
    pub fn with_scopes(
        team_id: String,
        user_id: String,
        team_name: Option<String>,
        user_name: Option<String>,
        client_id: Option<String>,
        redirect_uri: Option<String>,
        bot_scopes: Option<Vec<String>>,
        user_scopes: Option<Vec<String>>,
    ) -> Self {
        Self {
            team_id,
            user_id,
            team_name,
            user_name,
            client_id,
            redirect_uri,
            scopes: None, // Deprecated field, kept for backward compatibility
            bot_scopes,
            user_scopes,
        }
    }
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
    /// This method allows overwriting existing profiles
    pub fn set(&mut self, name: String, profile: Profile) {
        self.profiles.insert(name, profile);
    }

    /// Add a new profile, returning an error if the name already exists
    pub fn add(&mut self, name: String, profile: Profile) -> Result<(), ProfileError> {
        if self.profiles.contains_key(&name) {
            return Err(ProfileError::DuplicateName(name));
        }
        self.profiles.insert(name, profile);
        Ok(())
    }

    /// Update or create a profile for a given (team_id, user_id) pair
    /// If a profile with the same (team_id, user_id) exists, it will be updated
    /// If the profile name already exists but points to a different (team_id, user_id), returns an error
    /// PLACEHOLDER values are treated specially: they are always replaced by real values
    pub fn set_or_update(&mut self, name: String, profile: Profile) -> Result<(), ProfileError> {
        // Check if profile name already exists
        if let Some(existing) = self.profiles.get(&name) {
            // Special case: if existing has PLACEHOLDER values, allow replacement with real values
            let existing_is_placeholder =
                existing.team_id == "PLACEHOLDER" || existing.user_id == "PLACEHOLDER";
            let profile_is_placeholder =
                profile.team_id == "PLACEHOLDER" || profile.user_id == "PLACEHOLDER";

            // If existing is placeholder, always allow update with real or placeholder values
            if existing_is_placeholder {
                self.profiles.insert(name, profile);
                return Ok(());
            }

            // If new profile is placeholder but existing is real, keep existing (don't downgrade)
            if profile_is_placeholder {
                return Ok(());
            }

            // Both are real values - check if they match
            if existing.team_id != profile.team_id || existing.user_id != profile.user_id {
                return Err(ProfileError::DuplicateName(name));
            }
            // Same identity - just update
            self.profiles.insert(name, profile);
            return Ok(());
        }

        // Check if another profile with the same (team_id, user_id) exists
        // Skip this check for PLACEHOLDER values
        if profile.team_id != "PLACEHOLDER" && profile.user_id != "PLACEHOLDER" {
            if let Some((existing_name, _)) = self.profiles.iter().find(|(_, p)| {
                p.team_id != "PLACEHOLDER"
                    && p.user_id != "PLACEHOLDER"
                    && p.team_id == profile.team_id
                    && p.user_id == profile.user_id
            }) {
                // Update the existing profile
                let existing_name = existing_name.clone();
                self.profiles.insert(existing_name, profile);
                return Ok(());
            }
        }

        // No conflicts - add new profile
        self.profiles.insert(name, profile);
        Ok(())
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
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
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
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
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
                client_id: None,
                redirect_uri: None,
                scopes: None,
                bot_scopes: None,
                user_scopes: None,
            },
        );
        config.set(
            "profile2".to_string(),
            Profile {
                team_id: "T2".to_string(),
                user_id: "U2".to_string(),
                team_name: None,
                user_name: None,
                client_id: None,
                redirect_uri: None,
                scopes: None,
                bot_scopes: None,
                user_scopes: None,
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
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
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
                client_id: None,
                redirect_uri: None,
                scopes: None,
                bot_scopes: None,
                user_scopes: None,
            },
        );

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: ProfilesConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_profiles_config_add_duplicate_name() {
        let mut config = ProfilesConfig::new();
        let profile1 = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: None,
            user_name: None,
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };
        let profile2 = Profile {
            team_id: "T789".to_string(),
            user_id: "U012".to_string(),
            team_name: None,
            user_name: None,
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };

        // First add should succeed
        assert!(config.add("default".to_string(), profile1).is_ok());

        // Second add with same name should fail
        let result = config.add("default".to_string(), profile2);
        assert!(result.is_err());
        match result {
            Err(ProfileError::DuplicateName(name)) => {
                assert_eq!(name, "default");
            }
            _ => panic!("Expected DuplicateName error"),
        }
    }

    #[test]
    fn test_profiles_config_set_or_update_new() {
        let mut config = ProfilesConfig::new();
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Test Team".to_string()),
            user_name: Some("Test User".to_string()),
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };

        // Adding new profile should succeed
        assert!(config
            .set_or_update("default".to_string(), profile.clone())
            .is_ok());
        assert_eq!(config.get("default"), Some(&profile));
    }

    #[test]
    fn test_profiles_config_set_or_update_same_identity() {
        let mut config = ProfilesConfig::new();
        let profile1 = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Test Team".to_string()),
            user_name: Some("Test User".to_string()),
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };
        let profile2 = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Updated Team".to_string()),
            user_name: Some("Updated User".to_string()),
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };

        config
            .set_or_update("default".to_string(), profile1)
            .unwrap();

        // Updating with same identity should succeed
        assert!(config
            .set_or_update("default".to_string(), profile2.clone())
            .is_ok());
        assert_eq!(config.get("default"), Some(&profile2));
    }

    #[test]
    fn test_profiles_config_set_or_update_different_identity() {
        let mut config = ProfilesConfig::new();
        let profile1 = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: None,
            user_name: None,
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };
        let profile2 = Profile {
            team_id: "T789".to_string(),
            user_id: "U012".to_string(),
            team_name: None,
            user_name: None,
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };

        config
            .set_or_update("default".to_string(), profile1)
            .unwrap();

        // Trying to use same name with different identity should fail
        let result = config.set_or_update("default".to_string(), profile2);
        assert!(result.is_err());
        match result {
            Err(ProfileError::DuplicateName(_)) => {}
            _ => panic!("Expected DuplicateName error"),
        }
    }

    #[test]
    fn test_profiles_config_set_or_update_same_identity_different_name() {
        let mut config = ProfilesConfig::new();
        let profile1 = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Test Team".to_string()),
            user_name: Some("Test User".to_string()),
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };
        let profile2 = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Updated Team".to_string()),
            user_name: Some("Updated User".to_string()),
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };

        config.set_or_update("old".to_string(), profile1).unwrap();

        // Adding same identity with different name should update the old entry
        assert!(config
            .set_or_update("new".to_string(), profile2.clone())
            .is_ok());

        // Old name should still have the updated profile
        assert_eq!(config.get("old"), Some(&profile2));
        // New name should not exist
        assert_eq!(config.get("new"), None);
    }

    #[test]
    fn test_backward_compatibility_profile_without_client_id() {
        // Test that old profiles.json without client_id can be deserialized
        let json = r#"{
            "version": 1,
            "profiles": {
                "default": {
                    "team_id": "T123",
                    "user_id": "U456",
                    "team_name": "Test Team",
                    "user_name": "Test User"
                }
            }
        }"#;

        let config: ProfilesConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.profiles.len(), 1);

        let profile = config.get("default").unwrap();
        assert_eq!(profile.team_id, "T123");
        assert_eq!(profile.user_id, "U456");
        assert_eq!(profile.client_id, None);
    }

    #[test]
    fn test_profile_with_client_id_serialization() {
        // Test that new profiles with client_id serialize correctly
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Test Team".to_string()),
            user_name: Some("Test User".to_string()),
            client_id: Some("client-123".to_string()),
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile, deserialized);
        assert_eq!(deserialized.client_id, Some("client-123".to_string()));
    }

    #[test]
    fn test_profile_without_client_id_omits_field() {
        // Test that profiles without client_id don't include the field in JSON
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Test Team".to_string()),
            user_name: Some("Test User".to_string()),
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };

        let json = serde_json::to_string(&profile).unwrap();
        // The JSON should not contain "client_id" field due to skip_serializing_if
        assert!(!json.contains("client_id"));
    }

    #[test]
    fn test_profile_with_oauth_config_serialization() {
        // Test that profiles with full OAuth config serialize correctly
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Test Team".to_string()),
            user_name: Some("Test User".to_string()),
            client_id: Some("client-123".to_string()),
            redirect_uri: Some("http://127.0.0.1:8765/callback".to_string()),
            scopes: Some(vec!["chat:write".to_string(), "users:read".to_string()]),
            bot_scopes: None,
            user_scopes: None,
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile, deserialized);
        assert_eq!(deserialized.client_id, Some("client-123".to_string()));
        assert_eq!(
            deserialized.redirect_uri,
            Some("http://127.0.0.1:8765/callback".to_string())
        );
        assert_eq!(
            deserialized.scopes,
            Some(vec!["chat:write".to_string(), "users:read".to_string()])
        );
    }

    #[test]
    fn test_profile_without_oauth_config_omits_fields() {
        // Test that profiles without OAuth config don't include the fields in JSON
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Test Team".to_string()),
            user_name: Some("Test User".to_string()),
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };

        let json = serde_json::to_string(&profile).unwrap();
        // The JSON should not contain OAuth config fields due to skip_serializing_if
        assert!(!json.contains("client_id"));
        assert!(!json.contains("redirect_uri"));
        assert!(!json.contains("scopes"));
    }

    #[test]
    fn test_set_or_update_placeholder_to_real() {
        // Test that a profile with PLACEHOLDER values can be updated with real values
        let mut config = ProfilesConfig::new();

        // First, set a profile with PLACEHOLDER (e.g., from oauth config set)
        let placeholder_profile = Profile {
            team_id: "PLACEHOLDER".to_string(),
            user_id: "PLACEHOLDER".to_string(),
            team_name: None,
            user_name: None,
            client_id: Some("client-123".to_string()),
            redirect_uri: Some("http://localhost:8765/callback".to_string()),
            scopes: Some(vec!["chat:write".to_string()]),
            bot_scopes: None,
            user_scopes: None,
        };

        config
            .set_or_update("work".to_string(), placeholder_profile)
            .unwrap();

        // Then, update with real values (e.g., from login)
        let real_profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Real Team".to_string()),
            user_name: Some("Real User".to_string()),
            client_id: Some("client-123".to_string()),
            redirect_uri: Some("http://localhost:8765/callback".to_string()),
            scopes: Some(vec!["chat:write".to_string()]),
            bot_scopes: None,
            user_scopes: None,
        };

        // This should succeed and update the profile
        assert!(config
            .set_or_update("work".to_string(), real_profile.clone())
            .is_ok());

        // Verify the profile was updated with real values
        let updated = config.get("work").unwrap();
        assert_eq!(updated.team_id, "T123");
        assert_eq!(updated.user_id, "U456");
    }

    #[test]
    fn test_set_or_update_real_to_placeholder_keeps_real() {
        // Test that trying to update a real profile with PLACEHOLDER doesn't downgrade it
        let mut config = ProfilesConfig::new();

        // First, set a real profile
        let real_profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Real Team".to_string()),
            user_name: Some("Real User".to_string()),
            client_id: Some("client-123".to_string()),
            redirect_uri: Some("http://localhost:8765/callback".to_string()),
            scopes: Some(vec!["chat:write".to_string()]),
            bot_scopes: None,
            user_scopes: None,
        };

        config
            .set_or_update("work".to_string(), real_profile.clone())
            .unwrap();

        // Then, try to update with PLACEHOLDER values
        let placeholder_profile = Profile {
            team_id: "PLACEHOLDER".to_string(),
            user_id: "PLACEHOLDER".to_string(),
            team_name: None,
            user_name: None,
            client_id: Some("client-456".to_string()),
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };

        // This should succeed but keep the real values
        assert!(config
            .set_or_update("work".to_string(), placeholder_profile)
            .is_ok());

        // Verify the profile still has real values
        let updated = config.get("work").unwrap();
        assert_eq!(updated.team_id, "T123");
        assert_eq!(updated.user_id, "U456");
    }

    #[test]
    fn test_backward_compatibility_scopes_to_bot_scopes() {
        // Test that old profiles with scopes field can be read as bot_scopes
        let json = r#"{
            "version": 1,
            "profiles": {
                "default": {
                    "team_id": "T123",
                    "user_id": "U456",
                    "team_name": "Test Team",
                    "user_name": "Test User",
                    "scopes": ["chat:write", "users:read"]
                }
            }
        }"#;

        let config: ProfilesConfig = serde_json::from_str(json).unwrap();
        let profile = config.get("default").unwrap();

        // Old scopes field should be migrated to bot_scopes via get_bot_scopes()
        assert_eq!(
            profile.get_bot_scopes(),
            Some(vec!["chat:write".to_string(), "users:read".to_string()])
        );
        assert_eq!(profile.get_user_scopes(), None);
    }

    #[test]
    fn test_new_profile_with_bot_and_user_scopes() {
        // Test new profiles with separate bot_scopes and user_scopes
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Test Team".to_string()),
            user_name: Some("Test User".to_string()),
            client_id: Some("client-123".to_string()),
            redirect_uri: Some("http://localhost:8765/callback".to_string()),
            scopes: None,
            bot_scopes: Some(vec!["chat:write".to_string()]),
            user_scopes: Some(vec!["users:read".to_string()]),
        };

        assert_eq!(
            profile.get_bot_scopes(),
            Some(vec!["chat:write".to_string()])
        );
        assert_eq!(
            profile.get_user_scopes(),
            Some(vec!["users:read".to_string()])
        );
    }

    #[test]
    fn test_set_or_update_placeholder_no_conflict_with_other_profiles() {
        // Test that PLACEHOLDER profiles don't conflict with other real profiles
        let mut config = ProfilesConfig::new();

        // Add a real profile
        let real_profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: Some("Real Team".to_string()),
            user_name: None,
            client_id: None,
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };
        config
            .set_or_update("existing".to_string(), real_profile)
            .unwrap();

        // Add a PLACEHOLDER profile with different name
        let placeholder_profile = Profile {
            team_id: "PLACEHOLDER".to_string(),
            user_id: "PLACEHOLDER".to_string(),
            team_name: None,
            user_name: None,
            client_id: Some("client-789".to_string()),
            redirect_uri: None,
            scopes: None,
            bot_scopes: None,
            user_scopes: None,
        };

        // This should succeed without conflicts
        assert!(config
            .set_or_update("new".to_string(), placeholder_profile)
            .is_ok());

        // Both profiles should exist
        assert!(config.get("existing").is_some());
        assert!(config.get("new").is_some());
    }
}

#[cfg(test)]
mod backward_compat_tests {
    use super::*;

    #[test]
    fn test_get_bot_scopes_from_legacy_scopes() {
        // Test that scopes field is treated as bot_scopes for backward compatibility
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: None,
            user_name: None,
            client_id: None,
            redirect_uri: None,
            scopes: Some(vec!["chat:write".to_string(), "users:read".to_string()]),
            bot_scopes: None,
            user_scopes: None,
        };

        let bot_scopes = profile.get_bot_scopes();
        assert_eq!(
            bot_scopes,
            Some(vec!["chat:write".to_string(), "users:read".to_string()])
        );
    }

    #[test]
    fn test_get_bot_scopes_prefers_bot_scopes_over_scopes() {
        // Test that bot_scopes takes precedence over scopes
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: None,
            user_name: None,
            client_id: None,
            redirect_uri: None,
            scopes: Some(vec!["old:scope".to_string()]),
            bot_scopes: Some(vec!["new:scope".to_string()]),
            user_scopes: None,
        };

        let bot_scopes = profile.get_bot_scopes();
        assert_eq!(bot_scopes, Some(vec!["new:scope".to_string()]));
    }

    #[test]
    fn test_get_user_scopes_returns_none_for_legacy() {
        // Test that user_scopes returns None for legacy profiles
        let profile = Profile {
            team_id: "T123".to_string(),
            user_id: "U456".to_string(),
            team_name: None,
            user_name: None,
            client_id: None,
            redirect_uri: None,
            scopes: Some(vec!["chat:write".to_string()]),
            bot_scopes: None,
            user_scopes: None,
        };

        let user_scopes = profile.get_user_scopes();
        assert_eq!(user_scopes, None);
    }

    #[test]
    fn test_deserialize_old_profile_format() {
        // Test that old profile.json without bot_scopes/user_scopes can be deserialized
        let json = r#"{
            "team_id": "T123",
            "user_id": "U456",
            "team_name": "Test Team",
            "user_name": "Test User",
            "scopes": ["chat:write", "users:read"]
        }"#;

        let profile: Profile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.team_id, "T123");
        assert_eq!(profile.user_id, "U456");
        assert_eq!(
            profile.scopes,
            Some(vec!["chat:write".to_string(), "users:read".to_string()])
        );
        assert_eq!(profile.bot_scopes, None);
        assert_eq!(profile.user_scopes, None);

        // Verify backward compatibility: scopes should be accessible as bot_scopes
        let bot_scopes = profile.get_bot_scopes();
        assert_eq!(
            bot_scopes,
            Some(vec!["chat:write".to_string(), "users:read".to_string()])
        );
    }
}
