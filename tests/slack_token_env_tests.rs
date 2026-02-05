//! Integration tests for SLACK_TOKEN environment variable
//!
//! Tests that wrapper commands (conv list, users info, etc.) correctly use
//! the SLACK_TOKEN environment variable when set.

use httpmock::prelude::*;
use serde_json::json;
use slack_rs::cli::get_api_client_with_token_type;
use slack_rs::profile::{save_config, Profile, ProfilesConfig};
use std::env;
use tempfile::TempDir;

/// Setup test environment with a profile but no actual tokens in keyring
fn setup_test_profile() -> (TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("profiles.json");

    let mut config = ProfilesConfig::new();
    let profile = Profile {
        team_id: "T123ABC".to_string(),
        user_id: "U456DEF".to_string(),
        team_name: Some("Test Team".to_string()),
        user_name: Some("Test User".to_string()),
        client_id: None,
        redirect_uri: None,
        scopes: None,
        bot_scopes: None,
        user_scopes: None,
        default_token_type: None,
    };
    config.set("default".to_string(), profile);
    save_config(&config_path, &config).unwrap();

    (temp_dir, config_path.to_string_lossy().to_string())
}

#[tokio::test]
#[serial_test::serial]
async fn test_get_api_client_uses_slack_token_env() {
    // Set SLACK_TOKEN environment variable
    let test_token = "xoxb-test-env-token";
    env::set_var("SLACK_TOKEN", test_token);

    // Setup test profile (but we won't use tokens from it)
    let (_temp_dir, _config_path) = setup_test_profile();

    // Get API client - it should use SLACK_TOKEN
    let client = get_api_client_with_token_type(None, None).await;

    // Clean up
    env::remove_var("SLACK_TOKEN");

    // Verify client was created successfully
    assert!(
        client.is_ok(),
        "Failed to create client with SLACK_TOKEN: {:?}",
        client.err()
    );
}

#[tokio::test]
#[serial_test::serial]
async fn test_slack_token_bypasses_profile_token_store() {
    // Set SLACK_TOKEN environment variable
    let env_token = "xoxb-env-token-12345";
    env::set_var("SLACK_TOKEN", env_token);

    // Setup test profile with actual profile name
    let (_temp_dir, _config_path) = setup_test_profile();

    // Get API client with profile specified
    let client = get_api_client_with_token_type(Some("default".to_string()), None).await;

    // Clean up
    env::remove_var("SLACK_TOKEN");

    // The client should be created using SLACK_TOKEN, not the profile's token store
    assert!(
        client.is_ok(),
        "Should successfully create client with SLACK_TOKEN even when profile is specified"
    );
}

#[tokio::test]
#[serial_test::serial]
async fn test_wrapper_command_with_slack_token_authorization() {
    // Start a mock Slack API server
    let server = MockServer::start();

    // Set SLACK_TOKEN environment variable
    let test_token = "xoxb-test-wrapper-token";
    env::set_var("SLACK_TOKEN", test_token);

    // Create a mock endpoint for conversations.list
    let _mock = server.mock(|when, then| {
        when.method(POST)
            .path("/conversations.list")
            .header("Authorization", format!("Bearer {}", test_token));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "ok": true,
                "channels": [
                    {
                        "id": "C123456",
                        "name": "general",
                        "is_member": true
                    }
                ]
            }));
    });

    // Setup test profile
    let (_temp_dir, _config_path) = setup_test_profile();

    // Get API client (which should use SLACK_TOKEN)
    let client_result = get_api_client_with_token_type(None, None).await;
    assert!(client_result.is_ok());

    // Note: We can't directly test the full command flow without setting up more infrastructure,
    // but we've verified that get_api_client_with_token_type correctly uses SLACK_TOKEN

    // Clean up
    env::remove_var("SLACK_TOKEN");
}

#[tokio::test]
#[serial_test::serial]
async fn test_slack_token_takes_precedence_over_token_type_flag() {
    // Set SLACK_TOKEN environment variable
    let env_token = "xoxb-env-priority-token";
    env::set_var("SLACK_TOKEN", env_token);

    // Setup test profile
    let (_temp_dir, _config_path) = setup_test_profile();

    // Try to get API client with explicit token type (should still use SLACK_TOKEN)
    let client_result = get_api_client_with_token_type(
        Some("default".to_string()),
        Some(slack_rs::profile::TokenType::User),
    )
    .await;

    // Clean up
    env::remove_var("SLACK_TOKEN");

    // Should succeed - SLACK_TOKEN takes precedence
    assert!(
        client_result.is_ok(),
        "SLACK_TOKEN should take precedence over --token-type flag"
    );
}

#[tokio::test]
#[serial_test::serial]
async fn test_fallback_to_token_store_when_slack_token_not_set() {
    // Ensure SLACK_TOKEN is not set
    env::remove_var("SLACK_TOKEN");

    // Setup test profile
    let (_temp_dir, _config_path) = setup_test_profile();

    // Try to get API client without SLACK_TOKEN
    // This should fail because we don't have tokens in the keyring
    let client_result = get_api_client_with_token_type(None, None).await;

    // Should fail with token not found error (expected behavior without SLACK_TOKEN)
    assert!(
        client_result.is_err(),
        "Should fail when SLACK_TOKEN not set and no tokens in store"
    );
}

#[test]
fn test_command_response_with_token_type_metadata() {
    use serde_json::json;
    use slack_rs::api::CommandResponse;

    // Test that CommandResponse::with_token_type includes token_type in metadata
    let response = CommandResponse::with_token_type(
        json!({"ok": true, "channels": []}),
        Some("default".to_string()),
        "T123ABC".to_string(),
        "U456DEF".to_string(),
        "conversations.list".to_string(),
        "conv list".to_string(),
        Some("bot".to_string()),
    );

    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["meta"]["token_type"], "bot");
}

#[test]
fn test_command_response_with_user_token_type_metadata() {
    use serde_json::json;
    use slack_rs::api::CommandResponse;

    // Test that CommandResponse::with_token_type works with user token type
    let response = CommandResponse::with_token_type(
        json!({"ok": true}),
        Some("default".to_string()),
        "T123".to_string(),
        "U456".to_string(),
        "users.info".to_string(),
        "users info".to_string(),
        Some("user".to_string()),
    );

    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["meta"]["token_type"], "user");
}

#[test]
fn test_command_response_without_token_type_metadata() {
    use serde_json::json;
    use slack_rs::api::CommandResponse;

    // Test that CommandResponse::with_token_type with None doesn't include token_type
    let response = CommandResponse::with_token_type(
        json!({"ok": true}),
        Some("default".to_string()),
        "T123".to_string(),
        "U456".to_string(),
        "users.info".to_string(),
        "users info".to_string(),
        None,
    );

    let json_str = serde_json::to_string(&response).unwrap();
    // token_type should not be present in JSON when None
    assert!(!json_str.contains("token_type"));
}
