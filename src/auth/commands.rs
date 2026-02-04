//! Auth command implementations

use crate::oauth::{
    build_authorization_url, exchange_code, generate_pkce, generate_state, run_callback_server,
    OAuthConfig, OAuthError,
};
use crate::profile::{
    default_config_path, load_config, make_token_key, save_config, KeyringTokenStore, Profile,
    ProfilesConfig, TokenStore,
};
use std::process::Command;

/// Login command - performs OAuth authentication
///
/// # Arguments
/// * `config` - OAuth configuration
/// * `profile_name` - Optional profile name (defaults to "default")
/// * `base_url` - Optional base URL for testing
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

    // Start callback server
    println!("Waiting for authentication callback...");
    let callback_result = run_callback_server(3000, state.clone(), 300).await?;

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
    let result = Command::new("cmd").args(&["/C", "start", url]).spawn();

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
}
