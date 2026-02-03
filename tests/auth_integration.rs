//! Integration tests for auth commands

use slack_rs::profile::{make_token_key, TokenStore};

#[test]
fn test_auth_status_no_profile() {
    // This tests the status command when no profile exists
    let result = slack_rs::auth::status(Some("nonexistent".to_string()));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_auth_list_empty() {
    // This tests the list command
    // Note: This might show existing profiles if run on a system with profiles
    let result = slack_rs::auth::list();
    assert!(result.is_ok());
}

#[test]
fn test_auth_rename() {
    // Note: This test is limited because we can't easily create test profiles
    // without modifying the actual config directory
    let result = slack_rs::auth::rename("nonexistent".to_string(), "new".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_auth_logout_nonexistent() {
    let result = slack_rs::auth::logout(Some("nonexistent".to_string()));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// Test that demonstrates token storage integration
#[test]
fn test_token_storage_integration() {
    use slack_rs::profile::InMemoryTokenStore;

    let store = InMemoryTokenStore::new();
    let key = make_token_key("T123", "U456");

    // Set token
    assert!(store.set(&key, "test_token").is_ok());

    // Get token
    assert_eq!(store.get(&key).unwrap(), "test_token");

    // Check existence
    assert!(store.exists(&key));

    // Delete token
    assert!(store.delete(&key).is_ok());
    assert!(!store.exists(&key));
}

// Test profile and token integration
#[test]
fn test_profile_with_token_storage() {
    use slack_rs::profile::{InMemoryTokenStore, Profile, ProfilesConfig};

    let mut config = ProfilesConfig::new();
    let profile = Profile {
        team_id: "T123ABC".to_string(),
        user_id: "U456DEF".to_string(),
        team_name: Some("Test Team".to_string()),
        user_name: Some("Test User".to_string()),
    };

    // Add profile
    assert!(config
        .set_or_update("test".to_string(), profile.clone())
        .is_ok());

    // Store token
    let store = InMemoryTokenStore::new();
    let key = make_token_key(&profile.team_id, &profile.user_id);
    assert!(store.set(&key, "xoxb-test-token").is_ok());

    // Verify profile exists
    assert!(config.get("test").is_some());

    // Verify token exists
    assert!(store.exists(&key));
    assert_eq!(store.get(&key).unwrap(), "xoxb-test-token");
}
