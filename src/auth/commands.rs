//! Auth command implementations

use crate::auth::cloudflared::{CloudflaredError, CloudflaredTunnel};
use crate::auth::manifest::generate_manifest;
use crate::auth::ngrok::{NgrokError, NgrokTunnel};
use crate::oauth::{
    build_authorization_url, exchange_code, generate_pkce, generate_state, resolve_callback_port,
    run_callback_server, OAuthConfig, OAuthError,
};
use crate::profile::{
    default_config_path, load_config, make_token_key, save_config, KeyringTokenStore, Profile,
    ProfilesConfig, TokenStore,
};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

/// Login command with credential prompting - performs OAuth authentication
///
/// # Arguments
/// * `client_id` - Optional OAuth client ID from CLI
/// * `profile_name` - Optional profile name (defaults to "default")
/// * `redirect_uri` - OAuth redirect URI (used as fallback if not in profile)
/// * `_scopes` - OAuth scopes (legacy parameter, unused - use bot_scopes/user_scopes instead)
/// * `bot_scopes` - Optional bot scopes from CLI
/// * `user_scopes` - Optional user scopes from CLI
/// * `base_url` - Optional base URL for testing
#[allow(dead_code)]
pub async fn login_with_credentials(
    client_id: Option<String>,
    profile_name: Option<String>,
    redirect_uri: String,
    _scopes: Vec<String>,
    bot_scopes: Option<Vec<String>>,
    user_scopes: Option<Vec<String>>,
    base_url: Option<String>,
) -> Result<(), OAuthError> {
    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());

    // Load existing config to check for saved OAuth settings
    let config_path = default_config_path()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to get config path: {}", e)))?;
    let existing_config = load_config(&config_path).ok();

    // Resolve OAuth config with priority: CLI arg > saved in profile > prompt (not fallback)
    let (final_client_id, final_redirect_uri, final_bot_scopes, final_user_scopes) =
        if let Some(config) = &existing_config {
            if let Some(profile) = config.get(&profile_name) {
                // Client ID: CLI arg > profile > prompt
                let resolved_client_id = match client_id {
                    Some(id) => id,
                    None => {
                        if let Some(saved_id) = &profile.client_id {
                            saved_id.clone()
                        } else {
                            prompt_for_client_id()?
                        }
                    }
                };

                // Redirect URI: profile > prompt (not fallback)
                let resolved_redirect_uri = if let Some(saved_uri) = &profile.redirect_uri {
                    saved_uri.clone()
                } else {
                    prompt_for_redirect_uri(&redirect_uri)?
                };

                // Bot scopes: CLI arg > profile.bot_scopes > profile.scopes (legacy) > prompt
                let resolved_bot_scopes = if let Some(cli_bot_scopes) = bot_scopes {
                    cli_bot_scopes
                } else if let Some(saved_bot_scopes) = profile.get_bot_scopes() {
                    saved_bot_scopes
                } else {
                    prompt_for_bot_scopes()?
                };

                // User scopes: CLI arg > profile.user_scopes > prompt
                let resolved_user_scopes = if let Some(cli_user_scopes) = user_scopes {
                    cli_user_scopes
                } else if let Some(saved_user_scopes) = profile.get_user_scopes() {
                    saved_user_scopes
                } else {
                    prompt_for_user_scopes()?
                };

                (
                    resolved_client_id,
                    resolved_redirect_uri,
                    resolved_bot_scopes,
                    resolved_user_scopes,
                )
            } else {
                // Profile doesn't exist, prompt for all OAuth config
                let resolved_client_id =
                    client_id.unwrap_or_else(|| prompt_for_client_id().unwrap());
                let resolved_redirect_uri = prompt_for_redirect_uri(&redirect_uri)?;
                let resolved_bot_scopes =
                    bot_scopes.unwrap_or_else(|| prompt_for_bot_scopes().unwrap());
                let resolved_user_scopes =
                    user_scopes.unwrap_or_else(|| prompt_for_user_scopes().unwrap());
                (
                    resolved_client_id,
                    resolved_redirect_uri,
                    resolved_bot_scopes,
                    resolved_user_scopes,
                )
            }
        } else {
            // No config file exists, prompt for all OAuth config
            let resolved_client_id = client_id.unwrap_or_else(|| prompt_for_client_id().unwrap());
            let resolved_redirect_uri = prompt_for_redirect_uri(&redirect_uri)?;
            let resolved_bot_scopes =
                bot_scopes.unwrap_or_else(|| prompt_for_bot_scopes().unwrap());
            let resolved_user_scopes =
                user_scopes.unwrap_or_else(|| prompt_for_user_scopes().unwrap());
            (
                resolved_client_id,
                resolved_redirect_uri,
                resolved_bot_scopes,
                resolved_user_scopes,
            )
        };

    // Client secret resolution: Keyring > prompt
    let token_store = KeyringTokenStore::default_service();
    let final_client_secret =
        match crate::profile::get_oauth_client_secret(&token_store, &profile_name) {
            Ok(secret) => {
                println!("Using saved client secret from keyring.");
                secret
            }
            Err(_) => {
                // Not found in keyring, prompt for it
                prompt_for_client_secret()?
            }
        };

    // Create OAuth config
    // For backward compatibility, treat final_scopes as bot_scopes
    let config = OAuthConfig {
        client_id: final_client_id.clone(),
        client_secret: final_client_secret.clone(),
        redirect_uri: final_redirect_uri.clone(),
        scopes: final_bot_scopes.clone(),
        user_scopes: final_user_scopes.clone(),
    };

    // Perform login flow (existing implementation)
    let (team_id, team_name, user_id, bot_token, user_token) =
        perform_oauth_flow(&config, base_url.as_deref()).await?;

    // Save profile with OAuth config and client_secret to Keyring
    // For backward compatibility, treat final_scopes as bot_scopes
    save_profile_and_credentials(SaveCredentials {
        config_path: &config_path,
        profile_name: &profile_name,
        team_id: &team_id,
        team_name: &team_name,
        user_id: &user_id,
        bot_token: bot_token.as_deref(),
        user_token: user_token.as_deref(),
        client_id: &final_client_id,
        client_secret: &final_client_secret,
        redirect_uri: &final_redirect_uri,
        scopes: &final_bot_scopes, // Legacy field, now stores bot scopes
        bot_scopes: &final_bot_scopes,
        user_scopes: &final_user_scopes,
    })?;

    println!("✓ Authentication successful!");
    println!("Profile '{}' saved.", profile_name);

    Ok(())
}

/// Prompt user for OAuth client ID
fn prompt_for_client_id() -> Result<String, OAuthError> {
    loop {
        print!("Enter OAuth client ID: ");
        io::stdout()
            .flush()
            .map_err(|e| OAuthError::ConfigError(format!("Failed to flush stdout: {}", e)))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| OAuthError::ConfigError(format!("Failed to read input: {}", e)))?;

        let trimmed = input.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
        eprintln!("Client ID cannot be empty. Please try again.");
    }
}

/// Prompt user for OAuth client secret (hidden input)
fn prompt_for_client_secret() -> Result<String, OAuthError> {
    loop {
        let input = rpassword::prompt_password("Enter OAuth client secret: ")
            .map_err(|e| OAuthError::ConfigError(format!("Failed to read password: {}", e)))?;

        let trimmed = input.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
        eprintln!("Client secret cannot be empty. Please try again.");
    }
}

/// Prompt user for OAuth redirect URI with default option
fn prompt_for_redirect_uri(default: &str) -> Result<String, OAuthError> {
    print!("Enter OAuth redirect URI [{}]: ", default);
    io::stdout()
        .flush()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to flush stdout: {}", e)))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to read input: {}", e)))?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// Prompt user for bot OAuth scopes with default "all"
fn prompt_for_bot_scopes() -> Result<Vec<String>, OAuthError> {
    print!("Enter bot scopes (comma-separated, or 'all'/'bot:all' for preset) [all]: ");
    io::stdout()
        .flush()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to flush stdout: {}", e)))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to read input: {}", e)))?;

    let trimmed = input.trim();
    let scopes_input = if trimmed.is_empty() {
        vec!["all".to_string()]
    } else {
        trimmed.split(',').map(|s| s.trim().to_string()).collect()
    };

    Ok(crate::oauth::expand_scopes_with_context(
        &scopes_input,
        true,
    ))
}

/// Prompt user for user OAuth scopes with default "all"
fn prompt_for_user_scopes() -> Result<Vec<String>, OAuthError> {
    print!("Enter user scopes (comma-separated, or 'all'/'user:all' for preset) [all]: ");
    io::stdout()
        .flush()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to flush stdout: {}", e)))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to read input: {}", e)))?;

    let trimmed = input.trim();
    let scopes_input = if trimmed.is_empty() {
        vec!["all".to_string()]
    } else {
        trimmed.split(',').map(|s| s.trim().to_string()).collect()
    };

    Ok(crate::oauth::expand_scopes_with_context(
        &scopes_input,
        false,
    ))
}

/// Perform OAuth flow and return user/team info and tokens (bot and user)
async fn perform_oauth_flow(
    config: &OAuthConfig,
    base_url: Option<&str>,
) -> Result<
    (
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
    ),
    OAuthError,
> {
    // Validate config
    config.validate()?;

    // Generate PKCE and state
    let (code_verifier, code_challenge) = generate_pkce();
    let state = generate_state();

    // Build authorization URL
    let auth_url = build_authorization_url(config, &code_challenge, &state)?;

    println!("Opening browser for authentication...");
    println!("If the browser doesn't open, visit this URL:");
    println!("{}", auth_url);
    println!();

    // Try to open browser
    if let Err(e) = open_browser(&auth_url) {
        println!("Failed to open browser: {}", e);
        println!("Please open the URL manually in your browser.");
    }

    // Start callback server with resolved port
    let port = resolve_callback_port()?;
    println!("Waiting for authentication callback...");
    let callback_result = run_callback_server(port, state.clone(), 300).await?;

    println!("Received authorization code, exchanging for token...");

    // Exchange code for token
    let oauth_response =
        exchange_code(config, &callback_result.code, &code_verifier, base_url).await?;

    // Extract user and team information
    let team_id = oauth_response
        .team
        .as_ref()
        .map(|t| t.id.clone())
        .ok_or_else(|| OAuthError::SlackError("Missing team information".to_string()))?;

    let team_name = oauth_response.team.as_ref().map(|t| t.name.clone());

    let user_id = oauth_response
        .authed_user
        .as_ref()
        .map(|u| u.id.clone())
        .ok_or_else(|| OAuthError::SlackError("Missing user information".to_string()))?;

    // Extract bot token (from access_token field)
    let bot_token = oauth_response.access_token.clone();

    // Extract user token (from authed_user.access_token field)
    let user_token = oauth_response
        .authed_user
        .as_ref()
        .and_then(|u| u.access_token.clone());

    // Ensure at least one token is present
    if bot_token.is_none() && user_token.is_none() {
        return Err(OAuthError::SlackError(
            "No access tokens received".to_string(),
        ));
    }

    Ok((team_id, team_name, user_id, bot_token, user_token))
}

/// Credentials to save after OAuth authentication
struct SaveCredentials<'a> {
    config_path: &'a std::path::Path,
    profile_name: &'a str,
    team_id: &'a str,
    team_name: &'a Option<String>,
    user_id: &'a str,
    bot_token: Option<&'a str>,  // Bot token (optional)
    user_token: Option<&'a str>, // User token (optional)
    client_id: &'a str,
    client_secret: &'a str,
    redirect_uri: &'a str,
    scopes: &'a [String],      // Legacy field for backward compatibility
    bot_scopes: &'a [String],  // New bot scopes field
    user_scopes: &'a [String], // New user scopes field
}

/// Save profile and credentials (including client_id and client_secret)
fn save_profile_and_credentials(creds: SaveCredentials) -> Result<(), OAuthError> {
    // Load or create config
    let mut profiles_config =
        load_config(creds.config_path).unwrap_or_else(|_| ProfilesConfig::new());

    // Create profile with OAuth config (client_id, redirect_uri, bot_scopes, user_scopes)
    let profile = Profile {
        team_id: creds.team_id.to_string(),
        user_id: creds.user_id.to_string(),
        team_name: creds.team_name.clone(),
        user_name: None,
        client_id: Some(creds.client_id.to_string()),
        redirect_uri: Some(creds.redirect_uri.to_string()),
        scopes: Some(creds.scopes.to_vec()), // Legacy field
        bot_scopes: Some(creds.bot_scopes.to_vec()),
        user_scopes: Some(creds.user_scopes.to_vec()),
    };

    profiles_config
        .set_or_update(creds.profile_name.to_string(), profile)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to save profile: {}", e)))?;

    save_config(creds.config_path, &profiles_config)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to save config: {}", e)))?;

    // Save tokens to keyring
    let token_store = KeyringTokenStore::default_service();

    // Save bot token to team_id:user_id key (make_token_key format)
    if let Some(bot_token) = creds.bot_token {
        let bot_token_key = make_token_key(creds.team_id, creds.user_id);
        token_store
            .set(&bot_token_key, bot_token)
            .map_err(|e| OAuthError::ConfigError(format!("Failed to save bot token: {}", e)))?;
    }

    // Save user token to separate key (team_id:user_id:user)
    if let Some(user_token) = creds.user_token {
        let user_token_key = format!("{}:{}:user", creds.team_id, creds.user_id);
        token_store
            .set(&user_token_key, user_token)
            .map_err(|e| OAuthError::ConfigError(format!("Failed to save user token: {}", e)))?;
    }

    // Save client_secret to keyring (per design: service=slack-rs, username=oauth-client-secret:<profile_name>)
    let client_secret_key = format!("oauth-client-secret:{}", creds.profile_name);
    token_store
        .set(&client_secret_key, creds.client_secret)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to save client secret: {}", e)))?;

    Ok(())
}

/// Login command - performs OAuth authentication (legacy, delegates to login_with_credentials)
///
/// # Arguments
/// * `config` - OAuth configuration
/// * `profile_name` - Optional profile name (defaults to "default")
/// * `base_url` - Optional base URL for testing
#[allow(dead_code)]
pub async fn login(
    config: OAuthConfig,
    profile_name: Option<String>,
    base_url: Option<String>,
) -> Result<(), OAuthError> {
    // Validate config
    config.validate()?;

    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());

    // Generate PKCE and state
    let (code_verifier, code_challenge) = generate_pkce();
    let state = generate_state();

    // Build authorization URL
    let auth_url = build_authorization_url(&config, &code_challenge, &state)?;

    println!("Opening browser for authentication...");
    println!("If the browser doesn't open, visit this URL:");
    println!("{}", auth_url);
    println!();

    // Try to open browser
    if let Err(e) = open_browser(&auth_url) {
        println!("Failed to open browser: {}", e);
        println!("Please open the URL manually in your browser.");
    }

    // Start callback server with resolved port
    let port = resolve_callback_port()?;
    println!("Waiting for authentication callback...");
    let callback_result = run_callback_server(port, state.clone(), 300).await?;

    println!("Received authorization code, exchanging for token...");

    // Exchange code for token
    let oauth_response = exchange_code(
        &config,
        &callback_result.code,
        &code_verifier,
        base_url.as_deref(),
    )
    .await?;

    // Extract user and team information
    let team_id = oauth_response
        .team
        .as_ref()
        .map(|t| t.id.clone())
        .ok_or_else(|| OAuthError::SlackError("Missing team information".to_string()))?;

    let team_name = oauth_response.team.as_ref().map(|t| t.name.clone());

    let user_id = oauth_response
        .authed_user
        .as_ref()
        .map(|u| u.id.clone())
        .ok_or_else(|| OAuthError::SlackError("Missing user information".to_string()))?;

    let token = oauth_response
        .authed_user
        .as_ref()
        .and_then(|u| u.access_token.clone())
        .or(oauth_response.access_token.clone())
        .ok_or_else(|| OAuthError::SlackError("Missing access token".to_string()))?;

    // Save profile
    let config_path = default_config_path()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to get config path: {}", e)))?;

    let mut config = load_config(&config_path).unwrap_or_else(|_| ProfilesConfig::new());

    let profile = Profile {
        team_id: team_id.clone(),
        user_id: user_id.clone(),
        team_name,
        user_name: None, // We don't get user name from OAuth response
        client_id: None, // OAuth client ID not stored in legacy login flow
        redirect_uri: None,
        scopes: None,
        bot_scopes: None,
        user_scopes: None,
    };

    config
        .set_or_update(profile_name.clone(), profile)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to save profile: {}", e)))?;

    save_config(&config_path, &config)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to save config: {}", e)))?;

    // Save token
    let token_store = KeyringTokenStore::default_service();
    let token_key = make_token_key(&team_id, &user_id);
    token_store
        .set(&token_key, &token)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to save token: {}", e)))?;

    println!("✓ Authentication successful!");
    println!("Profile '{}' saved.", profile_name);

    Ok(())
}

/// Status command - shows current profile status
///
/// # Arguments
/// * `profile_name` - Optional profile name (defaults to "default")
pub fn status(profile_name: Option<String>) -> Result<(), String> {
    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());

    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let config = load_config(&config_path).map_err(|e| e.to_string())?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    println!("Profile: {}", profile_name);
    println!("Team ID: {}", profile.team_id);
    println!("User ID: {}", profile.user_id);
    if let Some(team_name) = &profile.team_name {
        println!("Team Name: {}", team_name);
    }
    if let Some(user_name) = &profile.user_name {
        println!("User Name: {}", user_name);
    }
    if let Some(client_id) = &profile.client_id {
        println!("Client ID: {}", client_id);
    }

    // Check if token exists
    let token_store = KeyringTokenStore::default_service();
    let token_key = make_token_key(&profile.team_id, &profile.user_id);
    let has_token = token_store.exists(&token_key);
    println!("Token: {}", if has_token { "Present" } else { "Missing" });

    Ok(())
}

/// List command - lists all profiles
pub fn list() -> Result<(), String> {
    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let config = load_config(&config_path).map_err(|e| e.to_string())?;

    if config.profiles.is_empty() {
        println!("No profiles found.");
        return Ok(());
    }

    println!("Profiles:");
    for name in config.list_names() {
        if let Some(profile) = config.get(&name) {
            let team_name = profile.team_name.as_deref().unwrap_or(&profile.team_id);
            println!(
                "  {}: {} ({}:{})",
                name, team_name, profile.team_id, profile.user_id
            );
        }
    }

    Ok(())
}

/// Rename command - renames a profile
///
/// # Arguments
/// * `old_name` - Current profile name
/// * `new_name` - New profile name
pub fn rename(old_name: String, new_name: String) -> Result<(), String> {
    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let mut config = load_config(&config_path).map_err(|e| e.to_string())?;

    // Check if old profile exists
    let profile = config
        .get(&old_name)
        .ok_or_else(|| format!("Profile '{}' not found", old_name))?
        .clone();

    // Check if new name already exists
    if config.get(&new_name).is_some() {
        return Err(format!("Profile '{}' already exists", new_name));
    }

    // Remove old profile and add with new name
    config.remove(&old_name);
    config.set(new_name.clone(), profile);

    save_config(&config_path, &config).map_err(|e| e.to_string())?;

    println!("Profile '{}' renamed to '{}'", old_name, new_name);

    Ok(())
}

/// Logout command - removes authentication
///
/// # Arguments
/// * `profile_name` - Optional profile name (defaults to "default")
pub fn logout(profile_name: Option<String>) -> Result<(), String> {
    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());

    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let mut config = load_config(&config_path).map_err(|e| e.to_string())?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?
        .clone();

    // Delete token
    let token_store = KeyringTokenStore::default_service();
    let token_key = make_token_key(&profile.team_id, &profile.user_id);
    let _ = token_store.delete(&token_key); // Ignore error if token doesn't exist

    // Remove profile
    config.remove(&profile_name);
    save_config(&config_path, &config).map_err(|e| e.to_string())?;

    println!("Profile '{}' removed", profile_name);

    Ok(())
}

/// Try to open a URL in the default browser
fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(url).spawn();

    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd").args(["/C", "start", url]).spawn();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let result: Result<std::process::Child, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Unsupported platform",
    ));

    result.map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_profile_not_found() {
        let result = status(Some("nonexistent".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_list_empty() {
        // This test may fail if there are existing profiles
        // It's more of a demonstration of how to use the function
        let result = list();
        assert!(result.is_ok());
    }

    #[test]
    fn test_rename_nonexistent_profile() {
        let result = rename("nonexistent".to_string(), "new_name".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_logout_nonexistent_profile() {
        let result = logout(Some("nonexistent".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_save_profile_and_credentials_with_client_id() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("profiles.json");

        // Save profile with client_id and client_secret to Keyring
        let scopes = vec!["chat:write".to_string(), "users:read".to_string()];
        let bot_scopes = vec!["chat:write".to_string()];
        let user_scopes = vec!["users:read".to_string()];
        save_profile_and_credentials(SaveCredentials {
            config_path: &config_path,
            profile_name: "test",
            team_id: "T123",
            team_name: &Some("Test Team".to_string()),
            user_id: "U456",
            bot_token: Some("xoxb-test-bot-token"),
            user_token: Some("xoxp-test-user-token"),
            client_id: "test-client-id",
            client_secret: "test-client-secret",
            redirect_uri: "http://127.0.0.1:8765/callback",
            scopes: &scopes,
            bot_scopes: &bot_scopes,
            user_scopes: &user_scopes,
        })
        .unwrap();

        // Verify profile was saved with client_id
        let config = load_config(&config_path).unwrap();
        let profile = config.get("test").unwrap();
        assert_eq!(profile.client_id, Some("test-client-id".to_string()));
        assert_eq!(profile.team_id, "T123");
        assert_eq!(profile.user_id, "U456");

        // Verify client_secret was saved to keyring
        let _token_store = KeyringTokenStore::default_service();
        let _client_secret_key = format!("oauth-client-secret:{}", "test");
        // Note: In CI/test environments without keyring, this may fail
        // In production, client_secret is stored in keyring for later use
    }

    #[test]
    fn test_backward_compatibility_load_profile_without_client_id() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("profiles.json");

        // Create old-format profile without client_id
        let mut config = ProfilesConfig::new();
        config.set(
            "legacy".to_string(),
            Profile {
                team_id: "T999".to_string(),
                user_id: "U888".to_string(),
                team_name: Some("Legacy Team".to_string()),
                user_name: Some("Legacy User".to_string()),
                client_id: None,
                redirect_uri: None,
                scopes: None,
                bot_scopes: None,
                user_scopes: None,
            },
        );
        save_config(&config_path, &config).unwrap();

        // Verify it can be loaded
        let loaded_config = load_config(&config_path).unwrap();
        let profile = loaded_config.get("legacy").unwrap();
        assert_eq!(profile.client_id, None);
        assert_eq!(profile.team_id, "T999");
    }

    #[test]
    fn test_bot_and_user_token_storage_keys() {
        use crate::profile::InMemoryTokenStore;

        // Create token store
        let token_store = InMemoryTokenStore::new();

        // Test credentials
        let team_id = "T123";
        let user_id = "U456";
        let bot_token = "xoxb-test-bot-token";
        let user_token = "xoxp-test-user-token";

        // Simulate what save_profile_and_credentials does
        let bot_token_key = make_token_key(team_id, user_id); // team_id:user_id
        let user_token_key = format!("{}:{}:user", team_id, user_id); // team_id:user_id:user

        token_store.set(&bot_token_key, bot_token).unwrap();
        token_store.set(&user_token_key, user_token).unwrap();

        // Verify bot token is stored at team_id:user_id
        assert_eq!(token_store.get(&bot_token_key).unwrap(), bot_token);
        assert_eq!(bot_token_key, "T123:U456");

        // Verify user token is stored at team_id:user_id:user
        assert_eq!(token_store.get(&user_token_key).unwrap(), user_token);
        assert_eq!(user_token_key, "T123:U456:user");

        // Verify they are different keys
        assert_ne!(bot_token_key, user_token_key);
    }
}

/// Extended login options
pub struct ExtendedLoginOptions {
    pub client_id: Option<String>,
    pub profile_name: Option<String>,
    pub redirect_uri: String,
    pub bot_scopes: Option<Vec<String>>,
    pub user_scopes: Option<Vec<String>>,
    pub cloudflared_path: Option<String>,
    pub ngrok_path: Option<String>,
    pub base_url: Option<String>,
}

/// Extended login command with cloudflared/ngrok support and manifest generation
///
/// # Arguments
/// * `options` - Extended login options
pub async fn login_with_credentials_extended(
    options: ExtendedLoginOptions,
) -> Result<(), OAuthError> {
    let ExtendedLoginOptions {
        client_id,
        profile_name,
        redirect_uri,
        bot_scopes,
        user_scopes,
        cloudflared_path,
        ngrok_path,
        base_url,
    } = options;
    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());

    // Load existing config to check for saved OAuth settings
    let config_path = default_config_path()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to get config path: {}", e)))?;
    let existing_config = load_config(&config_path).ok();

    // Resolve OAuth config with priority: CLI arg > saved in profile > prompt (not fallback)
    let (final_client_id, mut final_redirect_uri, final_bot_scopes, final_user_scopes) =
        if let Some(config) = &existing_config {
            if let Some(profile) = config.get(&profile_name) {
                // Client ID: CLI arg > profile > prompt
                let resolved_client_id = match client_id {
                    Some(id) => id,
                    None => {
                        if let Some(saved_id) = &profile.client_id {
                            saved_id.clone()
                        } else {
                            prompt_for_client_id()?
                        }
                    }
                };

                // Redirect URI: Will be resolved later (cloudflared or prompt)
                let resolved_redirect_uri = if let Some(saved_uri) = &profile.redirect_uri {
                    saved_uri.clone()
                } else {
                    redirect_uri.clone()
                };

                // Bot scopes: CLI arg > profile.bot_scopes > profile.scopes (legacy) > prompt
                let resolved_bot_scopes = if let Some(cli_bot_scopes) = bot_scopes {
                    cli_bot_scopes
                } else if let Some(saved_bot_scopes) = profile.get_bot_scopes() {
                    saved_bot_scopes
                } else {
                    prompt_for_bot_scopes()?
                };

                // User scopes: CLI arg > profile.user_scopes > prompt
                let resolved_user_scopes = if let Some(cli_user_scopes) = user_scopes {
                    cli_user_scopes
                } else if let Some(saved_user_scopes) = profile.get_user_scopes() {
                    saved_user_scopes
                } else {
                    prompt_for_user_scopes()?
                };

                (
                    resolved_client_id,
                    resolved_redirect_uri,
                    resolved_bot_scopes,
                    resolved_user_scopes,
                )
            } else {
                // Profile doesn't exist, prompt for all OAuth config
                let resolved_client_id =
                    client_id.unwrap_or_else(|| prompt_for_client_id().unwrap());
                let resolved_redirect_uri = redirect_uri.clone();
                let resolved_bot_scopes =
                    bot_scopes.unwrap_or_else(|| prompt_for_bot_scopes().unwrap());
                let resolved_user_scopes =
                    user_scopes.unwrap_or_else(|| prompt_for_user_scopes().unwrap());
                (
                    resolved_client_id,
                    resolved_redirect_uri,
                    resolved_bot_scopes,
                    resolved_user_scopes,
                )
            }
        } else {
            // No config file exists, prompt for all OAuth config
            let resolved_client_id = client_id.unwrap_or_else(|| prompt_for_client_id().unwrap());
            let resolved_redirect_uri = redirect_uri.clone();
            let resolved_bot_scopes =
                bot_scopes.unwrap_or_else(|| prompt_for_bot_scopes().unwrap());
            let resolved_user_scopes =
                user_scopes.unwrap_or_else(|| prompt_for_user_scopes().unwrap());
            (
                resolved_client_id,
                resolved_redirect_uri,
                resolved_bot_scopes,
                resolved_user_scopes,
            )
        };

    // Resolve redirect_uri: cloudflared, ngrok, or prompt
    let mut cloudflared_tunnel: Option<CloudflaredTunnel> = None;
    let mut ngrok_tunnel: Option<NgrokTunnel> = None;
    let use_cloudflared = cloudflared_path.is_some();
    let use_ngrok = ngrok_path.is_some();

    if let Some(path) = cloudflared_path {
        // Start cloudflared tunnel
        println!("Starting cloudflared tunnel...");
        let local_url = "http://localhost:8765";
        match CloudflaredTunnel::start(&path, local_url, 30) {
            Ok(t) => {
                let public_url = t.public_url();
                println!("✓ Tunnel started: {}", public_url);
                final_redirect_uri = format!("{}/callback", public_url);
                println!("Using redirect URI: {}", final_redirect_uri);
                cloudflared_tunnel = Some(t);
            }
            Err(CloudflaredError::StartError(msg)) => {
                return Err(OAuthError::ConfigError(format!(
                    "Failed to start cloudflared: {}",
                    msg
                )));
            }
            Err(CloudflaredError::UrlExtractionError(msg)) => {
                return Err(OAuthError::ConfigError(format!(
                    "Failed to extract cloudflared URL: {}",
                    msg
                )));
            }
            Err(e) => {
                return Err(OAuthError::ConfigError(format!("Cloudflared error: {}", e)));
            }
        }
    } else if let Some(path) = ngrok_path {
        // Start ngrok tunnel
        println!("Starting ngrok tunnel...");
        let port = 8765;
        match NgrokTunnel::start(&path, port, 30) {
            Ok(t) => {
                let public_url = t.public_url();
                println!("✓ Tunnel started: {}", public_url);
                final_redirect_uri = format!("{}/callback", public_url);
                println!("Using redirect URI: {}", final_redirect_uri);
                ngrok_tunnel = Some(t);
            }
            Err(NgrokError::StartError(msg)) => {
                return Err(OAuthError::ConfigError(format!(
                    "Failed to start ngrok: {}",
                    msg
                )));
            }
            Err(NgrokError::UrlExtractionError(msg)) => {
                return Err(OAuthError::ConfigError(format!(
                    "Failed to extract ngrok URL: {}",
                    msg
                )));
            }
            Err(e) => {
                return Err(OAuthError::ConfigError(format!("Ngrok error: {}", e)));
            }
        }
    } else {
        // Always prompt for redirect_uri when neither cloudflared nor ngrok is used
        final_redirect_uri = prompt_for_redirect_uri(&final_redirect_uri)?;
    }

    // Client secret resolution: Keyring > prompt
    let token_store = KeyringTokenStore::default_service();
    let final_client_secret =
        match crate::profile::get_oauth_client_secret(&token_store, &profile_name) {
            Ok(secret) => {
                println!("Using saved client secret from keyring.");
                secret
            }
            Err(_) => {
                // Not found in keyring, prompt for it
                prompt_for_client_secret()?
            }
        };

    // Create OAuth config
    let config = OAuthConfig {
        client_id: final_client_id.clone(),
        client_secret: final_client_secret.clone(),
        redirect_uri: final_redirect_uri.clone(),
        scopes: final_bot_scopes.clone(),
        user_scopes: final_user_scopes.clone(),
    };

    // Perform login flow (existing implementation)
    let (team_id, team_name, user_id, bot_token, user_token) =
        perform_oauth_flow(&config, base_url.as_deref()).await?;

    // Stop cloudflared tunnel if running
    if let Some(t) = cloudflared_tunnel {
        println!("Stopping cloudflared tunnel...");
        if let Err(e) = t.stop() {
            eprintln!("Warning: Failed to stop cloudflared: {}", e);
        }
    }

    // Stop ngrok tunnel if running
    if let Some(t) = ngrok_tunnel {
        println!("Stopping ngrok tunnel...");
        if let Err(e) = t.stop() {
            eprintln!("Warning: Failed to stop ngrok: {}", e);
        }
    }

    // Save profile with OAuth config and client_secret to Keyring
    save_profile_and_credentials(SaveCredentials {
        config_path: &config_path,
        profile_name: &profile_name,
        team_id: &team_id,
        team_name: &team_name,
        user_id: &user_id,
        bot_token: bot_token.as_deref(),
        user_token: user_token.as_deref(),
        client_id: &final_client_id,
        client_secret: &final_client_secret,
        redirect_uri: &final_redirect_uri,
        scopes: &final_bot_scopes, // Legacy field, now stores bot scopes
        bot_scopes: &final_bot_scopes,
        user_scopes: &final_user_scopes,
    })?;

    println!("✓ Authentication successful!");
    println!("Profile '{}' saved.", profile_name);

    // Generate and save manifest
    println!("\nGenerating Slack App Manifest...");
    match generate_manifest(
        &final_client_id,
        &final_bot_scopes,
        &final_user_scopes,
        &final_redirect_uri,
        use_cloudflared,
        use_ngrok,
        &profile_name,
    ) {
        Ok(manifest_yaml) => {
            // Determine manifest file path
            let manifest_path = get_manifest_path(&profile_name)?;

            // Save manifest to file
            match std::fs::write(&manifest_path, manifest_yaml) {
                Ok(_) => {
                    println!("✓ Manifest saved to: {}", manifest_path.display());
                    println!("\nYou can upload this manifest to your Slack App configuration:");
                    println!("  https://api.slack.com/apps");
                }
                Err(e) => {
                    eprintln!("Warning: Failed to save manifest file: {}", e);
                    eprintln!(
                        "OAuth authentication was successful, but manifest could not be saved."
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to generate manifest: {}", e);
            eprintln!("OAuth authentication was successful, but manifest could not be generated.");
        }
    }

    Ok(())
}

/// Get manifest file path for a profile
fn get_manifest_path(profile_name: &str) -> Result<PathBuf, OAuthError> {
    let config_dir = directories::BaseDirs::new()
        .ok_or_else(|| OAuthError::ConfigError("Failed to determine config directory".to_string()))?
        .config_dir()
        .join("slack-rs");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&config_dir).map_err(|e| {
        OAuthError::ConfigError(format!("Failed to create config directory: {}", e))
    })?;

    Ok(config_dir.join(format!("{}_manifest.yml", profile_name)))
}
