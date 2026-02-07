//! Doctor command for diagnostics
//!
//! Provides diagnostic information about the CLI environment:
//! - Profile configuration path
//! - Token store backend and path
//! - Token availability (bot/user)
//! - Scope hints for common permission issues

use serde::{Deserialize, Serialize};

use crate::profile::{create_token_store, default_config_path, load_config, make_token_key};

/// Diagnostic output structure
#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticInfo {
    /// Resolved profiles.json path
    pub config_path: String,
    /// Token store information
    pub token_store: TokenStoreInfo,
    /// Token availability status
    pub tokens: TokenStatus,
    /// Scope hints for common issues
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scope_hints: Vec<String>,
}

/// Token store backend information
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenStoreInfo {
    /// Backend type (e.g., "file", "keyring")
    pub backend: String,
    /// Resolved storage path
    pub path: String,
}

/// Token availability status
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenStatus {
    /// Whether bot token exists (value not shown)
    pub bot_token_exists: bool,
    /// Whether user token exists (value not shown)
    pub user_token_exists: bool,
}

/// Run doctor diagnostics
///
/// # Arguments
/// * `profile_name` - Optional profile name (defaults to "default")
/// * `json_output` - Whether to output JSON format
pub fn doctor(profile_name: Option<String>, json_output: bool) -> Result<(), String> {
    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());

    // Get config path
    let config_path =
        default_config_path().map_err(|e| format!("Failed to get config path: {}", e))?;

    // Check if config exists
    if !config_path.exists() {
        if json_output {
            let info = DiagnosticInfo {
                config_path: config_path.display().to_string(),
                token_store: TokenStoreInfo {
                    backend: "file".to_string(),
                    path: get_token_store_path()?,
                },
                tokens: TokenStatus {
                    bot_token_exists: false,
                    user_token_exists: false,
                },
                scope_hints: vec![
                    "No profiles configured. Run 'auth login' to authenticate.".to_string()
                ],
            };
            println!("{}", serde_json::to_string_pretty(&info).unwrap());
        } else {
            println!("Doctor Diagnostics");
            println!("==================");
            println!();
            println!("Config Path: {}", config_path.display());
            println!("Status: No profiles configured");
            println!();
            println!("Hint: Run 'auth login' to authenticate.");
        }
        return Ok(());
    }

    // Load config
    let config = load_config(&config_path).map_err(|e| format!("Failed to load config: {}", e))?;

    // Get profile
    let profile = config.get(&profile_name).ok_or_else(|| {
        format!(
            "Profile '{}' not found. Available profiles: {:?}",
            profile_name,
            config.list_names()
        )
    })?;

    // Check token store
    let token_store =
        create_token_store().map_err(|e| format!("Failed to create token store: {}", e))?;

    let bot_key = make_token_key(&profile.team_id, &profile.user_id);
    let user_key = format!("{}_user", bot_key);

    let bot_token_exists = token_store.exists(&bot_key);
    let user_token_exists = token_store.exists(&user_key);

    // Generate scope hints
    let mut scope_hints = Vec::new();
    if !bot_token_exists && !user_token_exists {
        scope_hints.push("No tokens found. Run 'auth login' to authenticate.".to_string());
    }

    let token_store_path = get_token_store_path()?;

    if json_output {
        let info = DiagnosticInfo {
            config_path: config_path.display().to_string(),
            token_store: TokenStoreInfo {
                backend: "file".to_string(),
                path: token_store_path,
            },
            tokens: TokenStatus {
                bot_token_exists,
                user_token_exists,
            },
            scope_hints,
        };
        println!("{}", serde_json::to_string_pretty(&info).unwrap());
    } else {
        println!("Doctor Diagnostics");
        println!("==================");
        println!();
        println!("Profile: {}", profile_name);
        println!("Config Path: {}", config_path.display());
        println!();
        println!("Token Store:");
        println!("  Backend: file");
        println!("  Path: {}", token_store_path);
        println!();
        println!("Token Status:");
        println!(
            "  Bot Token: {}",
            if bot_token_exists {
                "✓ exists"
            } else {
                "✗ not found"
            }
        );
        println!(
            "  User Token: {}",
            if user_token_exists {
                "✓ exists"
            } else {
                "✗ not found"
            }
        );

        if !scope_hints.is_empty() {
            println!();
            println!("Hints:");
            for hint in scope_hints {
                println!("  • {}", hint);
            }
        }
    }

    Ok(())
}

/// Get token store path
fn get_token_store_path() -> Result<String, String> {
    use crate::profile::FileTokenStore;

    FileTokenStore::default_path()
        .map(|p| p.display().to_string())
        .map_err(|e| format!("Failed to get token store path: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{InMemoryTokenStore, Profile, ProfilesConfig};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_diagnostic_info_serialization() {
        let info = DiagnosticInfo {
            config_path: "/home/user/.config/slack-rs/profiles.json".to_string(),
            token_store: TokenStoreInfo {
                backend: "file".to_string(),
                path: "/home/user/.local/share/slack-rs/tokens.json".to_string(),
            },
            tokens: TokenStatus {
                bot_token_exists: true,
                user_token_exists: false,
            },
            scope_hints: vec![],
        };

        let json = serde_json::to_string_pretty(&info).unwrap();
        assert!(json.contains("config_path"));
        assert!(json.contains("token_store"));
        assert!(json.contains("tokens"));
    }

    #[test]
    fn test_diagnostic_info_with_hints() {
        let info = DiagnosticInfo {
            config_path: "/home/user/.config/slack-rs/profiles.json".to_string(),
            token_store: TokenStoreInfo {
                backend: "file".to_string(),
                path: "/home/user/.local/share/slack-rs/tokens.json".to_string(),
            },
            tokens: TokenStatus {
                bot_token_exists: false,
                user_token_exists: false,
            },
            scope_hints: vec!["No tokens found. Run 'auth login' to authenticate.".to_string()],
        };

        let json = serde_json::to_string_pretty(&info).unwrap();
        assert!(json.contains("scope_hints"));
        assert!(json.contains("No tokens found"));
    }
}
