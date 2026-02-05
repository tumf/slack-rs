//! OAuth configuration management commands

use crate::oauth::OAuthError;
use crate::profile::{
    default_config_path, delete_oauth_client_secret, get_oauth_client_secret, load_config,
    save_config, store_oauth_client_secret, FileTokenStore, Profile, ProfilesConfig, TokenType,
};

/// Set OAuth configuration for a profile
///
/// # Arguments
/// * `profile_name` - Profile name
/// * `client_id` - OAuth client ID
/// * `redirect_uri` - OAuth redirect URI
/// * `scopes` - OAuth scopes (comma-separated, or 'all' for comprehensive preset)
pub fn oauth_set(
    profile_name: String,
    client_id: String,
    redirect_uri: String,
    scopes: String,
) -> Result<(), OAuthError> {
    let config_path = default_config_path()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to get config path: {}", e)))?;

    let mut config = load_config(&config_path).unwrap_or_else(|_| ProfilesConfig::new());

    // Prompt for client secret (secure input)
    let client_secret = rpassword::prompt_password("Enter OAuth client secret: ")
        .map_err(|e| OAuthError::ConfigError(format!("Failed to read password: {}", e)))?;

    if client_secret.trim().is_empty() {
        return Err(OAuthError::ConfigError(
            "Client secret cannot be empty".to_string(),
        ));
    }

    // Parse scopes
    let scopes_vec: Vec<String> = scopes
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if scopes_vec.is_empty() {
        return Err(OAuthError::ConfigError(
            "At least one scope is required".to_string(),
        ));
    }

    // Expand preset scopes (e.g., "all") and deduplicate
    let scopes_vec = crate::oauth::expand_scopes(&scopes_vec);

    // Get or create profile
    let profile = if let Some(existing) = config.get(&profile_name) {
        // Update existing profile with OAuth config
        Profile {
            team_id: existing.team_id.clone(),
            user_id: existing.user_id.clone(),
            team_name: existing.team_name.clone(),
            user_name: existing.user_name.clone(),
            client_id: Some(client_id.clone()),
            redirect_uri: Some(redirect_uri.clone()),
            scopes: Some(scopes_vec.clone()),
            bot_scopes: None,  // TODO: Will be populated in task 2
            user_scopes: None, // TODO: Will be populated in task 2
            default_token_type: existing.default_token_type,
        }
    } else {
        // Create placeholder profile (will be filled in during login)
        // We use placeholder values since OAuth config can be set before login
        Profile {
            team_id: "PLACEHOLDER".to_string(),
            user_id: "PLACEHOLDER".to_string(),
            team_name: None,
            user_name: None,
            client_id: Some(client_id.clone()),
            redirect_uri: Some(redirect_uri.clone()),
            scopes: Some(scopes_vec.clone()),
            bot_scopes: None,  // TODO: Will be populated in task 2
            user_scopes: None, // TODO: Will be populated in task 2
            default_token_type: None,
        }
    };

    config.set(profile_name.clone(), profile);

    save_config(&config_path, &config)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to save config: {}", e)))?;

    // Save client secret to file store
    let token_store = FileTokenStore::new()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to create token store: {}", e)))?;
    store_oauth_client_secret(&token_store, &profile_name, &client_secret)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to save client secret: {}", e)))?;

    println!("✓ OAuth configuration saved for profile '{}'", profile_name);
    println!("  Client ID: {}", client_id);
    println!("  Redirect URI: {}", redirect_uri);
    println!("  Scopes: {}", scopes_vec.join(", "));
    println!("  Client secret: (saved securely in file store)");

    Ok(())
}

/// Show OAuth configuration for a profile
///
/// # Arguments
/// * `profile_name` - Profile name
pub fn oauth_show(profile_name: String) -> Result<(), OAuthError> {
    let config_path = default_config_path()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to get config path: {}", e)))?;

    let config = load_config(&config_path)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to load config: {}", e)))?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| OAuthError::ConfigError(format!("Profile '{}' not found", profile_name)))?;

    println!("OAuth configuration for profile '{}':", profile_name);

    if let Some(client_id) = &profile.client_id {
        println!("  Client ID: {}", client_id);
    } else {
        println!("  Client ID: (not set)");
    }

    if let Some(redirect_uri) = &profile.redirect_uri {
        println!("  Redirect URI: {}", redirect_uri);
    } else {
        println!("  Redirect URI: (not set)");
    }

    if let Some(scopes) = &profile.scopes {
        println!("  Scopes: {}", scopes.join(", "));
    } else {
        println!("  Scopes: (not set)");
    }

    // Check if client secret exists in file store
    let token_store = FileTokenStore::new()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to create token store: {}", e)))?;
    let has_secret = get_oauth_client_secret(&token_store, &profile_name).is_ok();
    println!(
        "  Client secret: {}",
        if has_secret {
            "(saved in file store)"
        } else {
            "(not set)"
        }
    );

    Ok(())
}

/// Delete OAuth configuration for a profile
///
/// # Arguments
/// * `profile_name` - Profile name
pub fn oauth_delete(profile_name: String) -> Result<(), OAuthError> {
    let config_path = default_config_path()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to get config path: {}", e)))?;

    let mut config = load_config(&config_path)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to load config: {}", e)))?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| OAuthError::ConfigError(format!("Profile '{}' not found", profile_name)))?;

    // Clear OAuth config from profile
    let updated_profile = Profile {
        team_id: profile.team_id.clone(),
        user_id: profile.user_id.clone(),
        team_name: profile.team_name.clone(),
        user_name: profile.user_name.clone(),
        client_id: None,
        redirect_uri: None,
        scopes: None,
        bot_scopes: None,
        user_scopes: None,
        default_token_type: profile.default_token_type,
    };

    config.set(profile_name.clone(), updated_profile);

    save_config(&config_path, &config)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to save config: {}", e)))?;

    // Delete client secret from file store
    if let Ok(token_store) = FileTokenStore::new() {
        let _ = delete_oauth_client_secret(&token_store, &profile_name); // Ignore error if not found
    }

    println!(
        "✓ OAuth configuration deleted for profile '{}'",
        profile_name
    );

    Ok(())
}

/// Set default token type for a profile
///
/// # Arguments
/// * `profile_name` - Profile name
/// * `token_type` - Default token type (bot or user)
pub fn set_default_token_type(
    profile_name: String,
    token_type: TokenType,
) -> Result<(), OAuthError> {
    let config_path = default_config_path()
        .map_err(|e| OAuthError::ConfigError(format!("Failed to get config path: {}", e)))?;

    let mut config = load_config(&config_path).unwrap_or_else(|_| ProfilesConfig::new());

    // Get existing profile or return error
    let profile = config
        .get(&profile_name)
        .ok_or_else(|| OAuthError::ConfigError(format!("Profile '{}' not found", profile_name)))?
        .clone();

    // Update profile with new default token type
    let updated_profile = Profile {
        team_id: profile.team_id,
        user_id: profile.user_id,
        team_name: profile.team_name,
        user_name: profile.user_name,
        client_id: profile.client_id,
        redirect_uri: profile.redirect_uri,
        scopes: profile.scopes,
        bot_scopes: profile.bot_scopes,
        user_scopes: profile.user_scopes,
        default_token_type: Some(token_type),
    };

    config.set(profile_name.clone(), updated_profile);

    save_config(&config_path, &config)
        .map_err(|e| OAuthError::ConfigError(format!("Failed to save config: {}", e)))?;

    println!(
        "✓ Default token type set to '{}' for profile '{}'",
        token_type, profile_name
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_show_profile_not_found() {
        let result = oauth_show("nonexistent".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_oauth_delete_profile_not_found() {
        let result = oauth_delete("nonexistent".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
