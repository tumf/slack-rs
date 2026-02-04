//! Integration tests for manifest generation

use slack_rs::auth::generate_manifest;

#[test]
fn test_manifest_generation_with_cloudflared() {
    let bot_scopes = vec!["chat:write".to_string(), "channels:read".to_string()];
    let user_scopes = vec!["search:read".to_string()];
    let redirect_uri = "http://localhost:8765/callback";
    let use_cloudflared = true;
    let use_ngrok = false;
    let profile_name = "test-profile";

    let result = generate_manifest(
        "test-client-id",
        &bot_scopes,
        &user_scopes,
        redirect_uri,
        use_cloudflared,
        use_ngrok,
        profile_name,
    );

    assert!(result.is_ok());
    let yaml = result.unwrap();

    // Verify cloudflared wildcard URL is included
    assert!(yaml.contains("https://*.trycloudflare.com/callback"));

    // Verify redirect_uri is also included
    assert!(yaml.contains(redirect_uri));

    // Verify bot scopes
    assert!(yaml.contains("chat:write"));
    assert!(yaml.contains("channels:read"));

    // Verify user scopes
    assert!(yaml.contains("search:read"));

    // Verify profile name in display name
    assert!(yaml.contains("test-profile"));
}

#[test]
fn test_manifest_generation_without_cloudflared() {
    let bot_scopes = vec!["chat:write".to_string()];
    let user_scopes = vec![];
    let redirect_uri = "https://example.com/callback";
    let use_cloudflared = false;
    let use_ngrok = false;
    let profile_name = "default";

    let result = generate_manifest(
        "test-client-id",
        &bot_scopes,
        &user_scopes,
        redirect_uri,
        use_cloudflared,
        use_ngrok,
        profile_name,
    );

    assert!(result.is_ok());
    let yaml = result.unwrap();

    // Verify cloudflared wildcard URL is NOT included
    assert!(!yaml.contains("https://*.trycloudflare.com/callback"));

    // Verify only the provided redirect_uri is included
    assert!(yaml.contains(redirect_uri));

    // Verify bot scopes
    assert!(yaml.contains("chat:write"));
}

#[test]
fn test_manifest_generation_bot_and_user_scopes() {
    let bot_scopes = vec![
        "chat:write".to_string(),
        "users:read".to_string(),
        "channels:history".to_string(),
    ];
    let user_scopes = vec!["search:read".to_string(), "files:read".to_string()];
    let redirect_uri = "http://localhost:8765/callback";
    let use_cloudflared = false;
    let use_ngrok = false;
    let profile_name = "work";

    let result = generate_manifest(
        "test-client-id",
        &bot_scopes,
        &user_scopes,
        redirect_uri,
        use_cloudflared,
        use_ngrok,
        profile_name,
    );

    assert!(result.is_ok());
    let yaml = result.unwrap();

    // Verify all bot scopes are present
    for scope in &bot_scopes {
        assert!(
            yaml.contains(scope),
            "Bot scope '{}' not found in manifest",
            scope
        );
    }

    // Verify all user scopes are present
    for scope in &user_scopes {
        assert!(
            yaml.contains(scope),
            "User scope '{}' not found in manifest",
            scope
        );
    }

    // Verify both bot and user sections exist
    assert!(yaml.contains("bot:"));
    assert!(yaml.contains("user:"));
}

#[test]
fn test_manifest_generation_with_ngrok() {
    let bot_scopes = vec!["chat:write".to_string(), "channels:read".to_string()];
    let user_scopes = vec!["search:read".to_string()];
    let redirect_uri = "http://localhost:8765/callback";
    let use_cloudflared = false;
    let use_ngrok = true;
    let profile_name = "ngrok-test";

    let result = generate_manifest(
        "test-client-id",
        &bot_scopes,
        &user_scopes,
        redirect_uri,
        use_cloudflared,
        use_ngrok,
        profile_name,
    );

    assert!(result.is_ok());
    let yaml = result.unwrap();

    // Verify ngrok wildcard URL is included
    assert!(yaml.contains("https://*.ngrok-free.app/callback"));

    // Verify redirect_uri is also included
    assert!(yaml.contains(redirect_uri));

    // Verify bot scopes
    assert!(yaml.contains("chat:write"));
    assert!(yaml.contains("channels:read"));

    // Verify user scopes
    assert!(yaml.contains("search:read"));

    // Verify profile name in display name
    assert!(yaml.contains("ngrok-test"));
}
