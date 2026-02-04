//! Slack App Manifest generation
//!
//! This module provides functionality to generate Slack App Manifest YAML files
//! from OAuth configuration and scope information.

use serde::{Deserialize, Serialize};

/// Slack App Manifest structure
#[derive(Debug, Serialize, Deserialize)]
pub struct AppManifest {
    pub _metadata: Metadata,
    pub display_information: DisplayInformation,
    pub features: Features,
    pub oauth_config: OAuthConfig,
    pub settings: Settings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub major_version: u32,
    pub minor_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayInformation {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    pub bot_user: BotUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BotUser {
    pub display_name: String,
    pub always_online: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub redirect_urls: Vec<String>,
    pub scopes: Scopes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scopes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub org_deploy_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_mode_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_rotation_enabled: Option<bool>,
}

/// Generate Slack App Manifest YAML from OAuth configuration
///
/// # Arguments
/// * `_client_id` - OAuth client ID (not currently used in manifest generation)
/// * `bot_scopes` - Bot OAuth scopes
/// * `user_scopes` - User OAuth scopes
/// * `redirect_uri` - OAuth redirect URI
/// * `use_cloudflared` - Whether cloudflared tunnel is used (affects redirect_urls)
/// * `use_ngrok` - Whether ngrok tunnel is used (affects redirect_urls)
/// * `profile_name` - Profile name (used for bot display name)
///
/// # Returns
/// YAML string representation of the Slack App Manifest
pub fn generate_manifest(
    _client_id: &str,
    bot_scopes: &[String],
    user_scopes: &[String],
    redirect_uri: &str,
    use_cloudflared: bool,
    use_ngrok: bool,
    profile_name: &str,
) -> Result<String, String> {
    // Determine redirect URLs based on whether cloudflared or ngrok is used
    let redirect_urls = if use_cloudflared {
        vec![
            "https://*.trycloudflare.com/callback".to_string(),
            redirect_uri.to_string(),
        ]
    } else if use_ngrok {
        vec![
            "https://*.ngrok-free.app/callback".to_string(),
            redirect_uri.to_string(),
        ]
    } else {
        vec![redirect_uri.to_string()]
    };

    let manifest = AppManifest {
        _metadata: Metadata {
            major_version: 1,
            minor_version: 1,
        },
        display_information: DisplayInformation {
            name: format!("slack-rs ({})", profile_name),
            description: Some(format!(
                "Slack CLI application for profile '{}'",
                profile_name
            )),
            background_color: Some("#2c2d30".to_string()),
        },
        features: Features {
            bot_user: BotUser {
                display_name: format!("slack-rs-{}", profile_name),
                always_online: false,
            },
        },
        oauth_config: OAuthConfig {
            redirect_urls,
            scopes: Scopes {
                bot: if bot_scopes.is_empty() {
                    None
                } else {
                    Some(bot_scopes.to_vec())
                },
                user: if user_scopes.is_empty() {
                    None
                } else {
                    Some(user_scopes.to_vec())
                },
            },
        },
        settings: Settings {
            org_deploy_enabled: false,
            socket_mode_enabled: Some(false),
            token_rotation_enabled: Some(false),
        },
    };

    serde_yaml::to_string(&manifest).map_err(|e| format!("Failed to serialize manifest: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_manifest_with_bot_scopes_only() {
        let bot_scopes = vec!["chat:write".to_string(), "users:read".to_string()];
        let user_scopes = vec![];
        let result = generate_manifest(
            "test-client-id",
            &bot_scopes,
            &user_scopes,
            "http://localhost:8765/callback",
            false,
            false,
            "default",
        );

        assert!(result.is_ok());
        let yaml = result.unwrap();
        assert!(yaml.contains("chat:write"));
        assert!(yaml.contains("users:read"));
        assert!(yaml.contains("http://localhost:8765/callback"));
        assert!(yaml.contains("slack-rs (default)"));
        assert!(yaml.contains("bot:"));
        // Verify the structure - bot scopes should be under "scopes:" section
        assert!(yaml.contains("scopes:"));
    }

    #[test]
    fn test_generate_manifest_with_cloudflared() {
        let bot_scopes = vec!["chat:write".to_string()];
        let user_scopes = vec!["search:read".to_string()];
        let result = generate_manifest(
            "test-client-id",
            &bot_scopes,
            &user_scopes,
            "http://localhost:8765/callback",
            true,
            false,
            "work",
        );

        assert!(result.is_ok());
        let yaml = result.unwrap();
        assert!(yaml.contains("https://*.trycloudflare.com/callback"));
        assert!(yaml.contains("http://localhost:8765/callback"));
        assert!(yaml.contains("chat:write"));
        assert!(yaml.contains("search:read"));
    }

    #[test]
    fn test_generate_manifest_with_user_scopes() {
        let bot_scopes = vec!["chat:write".to_string()];
        let user_scopes = vec!["users:read".to_string(), "search:read".to_string()];
        let result = generate_manifest(
            "test-client-id",
            &bot_scopes,
            &user_scopes,
            "http://localhost:8765/callback",
            false,
            false,
            "personal",
        );

        assert!(result.is_ok());
        let yaml = result.unwrap();
        assert!(yaml.contains("chat:write"));
        assert!(yaml.contains("users:read"));
        assert!(yaml.contains("search:read"));
        assert!(yaml.contains("bot:"));
        assert!(yaml.contains("user:"));
    }

    #[test]
    fn test_generate_manifest_empty_scopes() {
        let bot_scopes = vec![];
        let user_scopes = vec![];
        let result = generate_manifest(
            "test-client-id",
            &bot_scopes,
            &user_scopes,
            "http://localhost:8765/callback",
            false,
            false,
            "empty",
        );

        // Should still generate a valid manifest even with empty scopes
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_manifest_with_ngrok() {
        let bot_scopes = vec!["chat:write".to_string()];
        let user_scopes = vec!["search:read".to_string()];
        let result = generate_manifest(
            "test-client-id",
            &bot_scopes,
            &user_scopes,
            "http://localhost:8765/callback",
            false,
            true,
            "ngrok-test",
        );

        assert!(result.is_ok());
        let yaml = result.unwrap();
        assert!(yaml.contains("https://*.ngrok-free.app/callback"));
        assert!(yaml.contains("http://localhost:8765/callback"));
        assert!(yaml.contains("chat:write"));
        assert!(yaml.contains("search:read"));
    }
}
