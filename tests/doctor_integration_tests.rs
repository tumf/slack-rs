use slack_rs::profile::{save_config, Profile, ProfilesConfig};
use std::env;
use std::fs;
use tempfile::TempDir;

/// Helper to set up a test environment with profile and tokens
fn setup_test_env() -> (TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let token_dir = temp_dir.path().join("tokens");

    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&token_dir).unwrap();

    // Set environment variables to use temp directories
    let config_path = config_dir.join("profiles.json");
    let token_path = token_dir.join("tokens.json");

    env::set_var("XDG_CONFIG_HOME", config_dir.to_str().unwrap());
    env::set_var("SLACK_RS_TOKENS_PATH", token_path.to_str().unwrap());

    // Create a test profile
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
    config.set("test_profile".to_string(), profile);
    save_config(&config_path, &config).unwrap();

    // Create token store with dummy tokens
    let token_store = slack_rs::profile::create_token_store().unwrap();
    let bot_key = slack_rs::profile::make_token_key("T123ABC", "U456DEF");
    let user_key = format!("{}_user", bot_key);

    // Store tokens with realistic-looking values
    token_store
        .set(
            &bot_key,
            "bot_test_token_placeholder",
        )
        .unwrap();
    token_store
        .set(
            &user_key,
            "user_test_token_placeholder",
        )
        .unwrap();

    (temp_dir, config_path.display().to_string())
}

#[test]
fn test_doctor_output_does_not_contain_token_values() {
    let (_temp_dir, _config_path) = setup_test_env();

    // Verify that the diagnostic structures can't hold token values by design
    let info = slack_rs::commands::doctor::DiagnosticInfo {
        config_path: "/test/path".to_string(),
        token_store: slack_rs::commands::doctor::TokenStoreInfo {
            backend: "file".to_string(),
            path: "/test/tokens.json".to_string(),
        },
        tokens: slack_rs::commands::doctor::TokenStatus {
            bot_token_exists: true,
            user_token_exists: true,
        },
        scope_hints: vec![],
    };

    let json = serde_json::to_string(&info).unwrap();

    // Verify no token patterns appear in output
    assert!(!json.contains("xoxb-"), "Output contains bot token pattern");
    assert!(
        !json.contains("xoxp-"),
        "Output contains user token pattern"
    );

    // Verify expected fields are present
    assert!(json.contains("config_path"));
    assert!(json.contains("token_store"));
    assert!(json.contains("bot_token_exists"));
    assert!(json.contains("user_token_exists"));
}

#[test]
fn test_doctor_json_output_schema() {
    let info = slack_rs::commands::doctor::DiagnosticInfo {
        config_path: "/home/user/.config/slack-rs/profiles.json".to_string(),
        token_store: slack_rs::commands::doctor::TokenStoreInfo {
            backend: "file".to_string(),
            path: "/home/user/.local/share/slack-rs/tokens.json".to_string(),
        },
        tokens: slack_rs::commands::doctor::TokenStatus {
            bot_token_exists: true,
            user_token_exists: false,
        },
        scope_hints: vec!["Test hint".to_string()],
    };

    let json = serde_json::to_string_pretty(&info).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify required fields
    assert!(parsed.get("config_path").is_some());
    assert!(parsed.get("token_store").is_some());
    assert!(parsed.get("tokens").is_some());

    // Verify token_store structure
    let token_store = parsed.get("token_store").unwrap();
    assert!(token_store.get("backend").is_some());
    assert!(token_store.get("path").is_some());

    // Verify tokens structure
    let tokens = parsed.get("tokens").unwrap();
    assert!(tokens.get("bot_token_exists").is_some());
    assert!(tokens.get("user_token_exists").is_some());

    // Verify scope_hints is present when not empty
    assert!(parsed.get("scope_hints").is_some());
}

#[test]
fn test_doctor_json_output_omits_empty_scope_hints() {
    let info = slack_rs::commands::doctor::DiagnosticInfo {
        config_path: "/test/path".to_string(),
        token_store: slack_rs::commands::doctor::TokenStoreInfo {
            backend: "file".to_string(),
            path: "/test/tokens.json".to_string(),
        },
        tokens: slack_rs::commands::doctor::TokenStatus {
            bot_token_exists: true,
            user_token_exists: true,
        },
        scope_hints: vec![],
    };

    let json = serde_json::to_string(&info).unwrap();

    // Empty scope_hints should not appear in JSON due to skip_serializing_if
    assert!(!json.contains("scope_hints"));
}

#[test]
fn test_token_status_only_contains_existence_flags() {
    let status = slack_rs::commands::doctor::TokenStatus {
        bot_token_exists: true,
        user_token_exists: false,
    };

    let json = serde_json::to_string(&status).unwrap();

    // Verify it only contains boolean flags, never token values
    assert!(json.contains("bot_token_exists"));
    assert!(json.contains("user_token_exists"));
    assert!(json.contains("true"));
    assert!(json.contains("false"));

    // Verify no token-like patterns
    assert!(!json.contains("xoxb"));
    assert!(!json.contains("xoxp"));
    assert!(!json.contains("token\":\""));
}

#[test]
fn test_diagnostic_info_deserialization() {
    let json = r#"{
        "config_path": "/test/profiles.json",
        "token_store": {
            "backend": "file",
            "path": "/test/tokens.json"
        },
        "tokens": {
            "bot_token_exists": true,
            "user_token_exists": false
        },
        "scope_hints": ["Hint 1", "Hint 2"]
    }"#;

    let info: slack_rs::commands::doctor::DiagnosticInfo = serde_json::from_str(json).unwrap();

    assert_eq!(info.config_path, "/test/profiles.json");
    assert_eq!(info.token_store.backend, "file");
    assert_eq!(info.token_store.path, "/test/tokens.json");
    assert_eq!(info.tokens.bot_token_exists, true);
    assert_eq!(info.tokens.user_token_exists, false);
    assert_eq!(info.scope_hints.len(), 2);
}
