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

    // Verify expected fields are present (camelCase)
    assert!(json.contains("configPath"));
    assert!(json.contains("tokenStore"));
    assert!(json.contains("botTokenExists"));
    assert!(json.contains("userTokenExists"));
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

    // Verify required fields (camelCase)
    assert!(parsed.get("configPath").is_some());
    assert!(parsed.get("tokenStore").is_some());
    assert!(parsed.get("tokens").is_some());

    // Verify tokenStore structure (camelCase)
    let token_store = parsed.get("tokenStore").unwrap();
    assert!(token_store.get("backend").is_some());
    assert!(token_store.get("path").is_some());

    // Verify tokens structure (camelCase)
    let tokens = parsed.get("tokens").unwrap();
    assert!(tokens.get("botTokenExists").is_some());
    assert!(tokens.get("userTokenExists").is_some());

    // Verify scopeHints is present when not empty (camelCase)
    assert!(parsed.get("scopeHints").is_some());
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

    // Empty scopeHints should not appear in JSON due to skip_serializing_if (camelCase)
    assert!(!json.contains("scopeHints"));
    assert!(!json.contains("scope_hints"));
}

#[test]
fn test_token_status_only_contains_existence_flags() {
    let status = slack_rs::commands::doctor::TokenStatus {
        bot_token_exists: true,
        user_token_exists: false,
    };

    let json = serde_json::to_string(&status).unwrap();

    // Verify it only contains boolean flags, never token values (camelCase)
    assert!(json.contains("botTokenExists"));
    assert!(json.contains("userTokenExists"));
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
        "configPath": "/test/profiles.json",
        "tokenStore": {
            "backend": "file",
            "path": "/test/tokens.json"
        },
        "tokens": {
            "botTokenExists": true,
            "userTokenExists": false
        },
        "scopeHints": ["Hint 1", "Hint 2"]
    }"#;

    let info: slack_rs::commands::doctor::DiagnosticInfo = serde_json::from_str(json).unwrap();

    assert_eq!(info.config_path, "/test/profiles.json");
    assert_eq!(info.token_store.backend, "file");
    assert_eq!(info.token_store.path, "/test/tokens.json");
    assert!(info.tokens.bot_token_exists);
    assert!(!info.tokens.user_token_exists);
    assert_eq!(info.scope_hints.len(), 2);
}

#[test]
fn test_doctor_help_output() {
    use std::process::Command;

    let output = Command::new("cargo")
        .args(["run", "--", "doctor", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify help output contains expected sections
    assert!(
        stdout.contains("Doctor diagnostics command"),
        "Help should contain command description"
    );
    assert!(
        stdout.contains("USAGE:"),
        "Help should contain usage section"
    );
    assert!(
        stdout.contains("OPTIONS:"),
        "Help should contain options section"
    );
    assert!(
        stdout.contains("--profile"),
        "Help should mention --profile option"
    );
    assert!(
        stdout.contains("--json"),
        "Help should mention --json option"
    );
    assert!(
        stdout.contains("EXAMPLES:"),
        "Help should contain examples section"
    );

    // Verify it doesn't run diagnostics (no diagnostic output)
    assert!(
        !stdout.contains("Doctor Diagnostics"),
        "Help should not run diagnostics"
    );
    assert!(
        !stdout.contains("Config Path:"),
        "Help should not show config path"
    );
}

#[test]
fn test_doctor_help_short_flag() {
    use std::process::Command;

    let output = Command::new("cargo")
        .args(["run", "--", "doctor", "-h"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify help output with -h flag
    assert!(
        stdout.contains("Doctor diagnostics command"),
        "Help with -h should contain command description"
    );
    assert!(
        stdout.contains("USAGE:"),
        "Help with -h should contain usage section"
    );
}
