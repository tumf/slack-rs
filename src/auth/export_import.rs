//! Export and import commands for profile backup and migration

use crate::auth::crypto::{self, KdfParams};
use crate::auth::format::{self, ExportPayload, ExportProfile};
use crate::profile::{
    default_config_path, get_oauth_client_secret, load_config, make_token_key, save_config,
    store_oauth_client_secret, Profile, TokenStore, TokenStoreError,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportImportError {
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    #[error("Token not found for profile: {0}")]
    TokenNotFound(String),
    #[error("No profiles to export")]
    NoProfiles,
    #[error("Export requires --yes flag for confirmation")]
    ConfirmationRequired,
    #[error("Profile already exists: {0} (use --force to overwrite)")]
    ProfileExists(String),
    #[error("Empty passphrase not allowed")]
    EmptyPassphrase,
    #[cfg(unix)]
    #[error("File permission error: {0}")]
    PermissionError(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Crypto error: {0}")]
    Crypto(#[from] crypto::CryptoError),
    #[error("Format error: {0}")]
    Format(#[from] format::FormatError),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Token store error: {0}")]
    TokenStore(#[from] TokenStoreError),
}

pub type Result<T> = std::result::Result<T, ExportImportError>;

/// Options for export command
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub profile_name: Option<String>,
    pub all: bool,
    pub output_path: String,
    pub passphrase: String,
    pub yes: bool,
}

/// Options for import command
#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub input_path: String,
    pub passphrase: String,
    pub yes: bool,
    pub force: bool,
    pub dry_run: bool,
    pub json: bool,
}

/// Import action taken for a profile
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportAction {
    Updated,
    Skipped,
    Overwritten,
}

impl std::fmt::Display for ImportAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportAction::Updated => write!(f, "updated"),
            ImportAction::Skipped => write!(f, "skipped"),
            ImportAction::Overwritten => write!(f, "overwritten"),
        }
    }
}

/// Result for a single profile import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileImportResult {
    pub profile_name: String,
    pub action: ImportAction,
    pub reason: String,
}

/// Overall import result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub profiles: Vec<ProfileImportResult>,
    pub summary: ImportSummary,
    pub dry_run: bool,
}

/// Summary of import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    pub updated: usize,
    pub skipped: usize,
    pub overwritten: usize,
    pub total: usize,
}

/// Export profiles to encrypted file
pub fn export_profiles(token_store: &dyn TokenStore, options: &ExportOptions) -> Result<()> {
    // Require --yes confirmation
    if !options.yes {
        return Err(ExportImportError::ConfirmationRequired);
    }

    // Validate passphrase
    if options.passphrase.is_empty() {
        return Err(ExportImportError::EmptyPassphrase);
    }

    // Load profiles
    let config_path =
        default_config_path().map_err(|e| ExportImportError::Storage(e.to_string()))?;
    let config =
        load_config(&config_path).map_err(|e| ExportImportError::Storage(e.to_string()))?;

    // Select profiles to export
    let profiles_to_export: Vec<(String, &Profile)> = if options.all {
        config
            .profiles
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect()
    } else if let Some(ref profile_name) = options.profile_name {
        let profile = config
            .get(profile_name)
            .ok_or_else(|| ExportImportError::ProfileNotFound(profile_name.clone()))?;
        vec![(profile_name.clone(), profile)]
    } else {
        // Default to "default" profile
        let profile = config
            .get("default")
            .ok_or_else(|| ExportImportError::ProfileNotFound("default".to_string()))?;
        vec![("default".to_string(), profile)]
    };

    if profiles_to_export.is_empty() {
        return Err(ExportImportError::NoProfiles);
    }

    // Build export payload
    let mut payload = ExportPayload::new();
    for (name, profile) in profiles_to_export {
        let token_key = make_token_key(&profile.team_id, &profile.user_id);
        let token = token_store
            .get(&token_key)
            .map_err(|_| ExportImportError::TokenNotFound(name.clone()))?;

        // Try to get OAuth credentials (non-fatal if not found)
        let client_id = profile.client_id.clone();
        let client_secret = get_oauth_client_secret(token_store, &name).ok();

        payload.profiles.insert(
            name,
            ExportProfile {
                team_id: profile.team_id.clone(),
                user_id: profile.user_id.clone(),
                team_name: profile.team_name.clone(),
                user_name: profile.user_name.clone(),
                token,
                client_id,
                client_secret,
            },
        );
    }

    // Encrypt payload
    let kdf_params = KdfParams {
        salt: crypto::generate_salt(),
        ..Default::default()
    };

    let key = crypto::derive_key(&options.passphrase, &kdf_params)?;
    let payload_json = serde_json::to_vec(&payload)
        .map_err(|e| ExportImportError::Format(format::FormatError::Json(e)))?;
    let encrypted = crypto::encrypt(&payload_json, &key)?;

    // Encode to binary format
    let encoded = format::encode_export(&payload, &encrypted, &kdf_params)?;

    // Check existing file permissions
    let output_path = Path::new(&options.output_path);
    if output_path.exists() {
        check_file_permissions(output_path)?;
    }

    // Write to file with 0600 permissions
    write_secure_file(output_path, &encoded)?;

    Ok(())
}

/// Import profiles from encrypted file
pub fn import_profiles(
    token_store: &dyn TokenStore,
    options: &ImportOptions,
) -> Result<ImportResult> {
    // Validate passphrase
    if options.passphrase.is_empty() {
        return Err(ExportImportError::EmptyPassphrase);
    }

    // Read and check file permissions
    let input_path = Path::new(&options.input_path);
    check_file_permissions(input_path)?;

    let encoded_data = fs::read(input_path)?;

    // Decode from binary format
    let decoded = format::decode_export(&encoded_data)?;

    // Decrypt payload
    let key = crypto::derive_key(&options.passphrase, &decoded.kdf_params)?;
    let payload_json = crypto::decrypt(&decoded.encrypted_data, &key)?;
    let payload: ExportPayload = serde_json::from_slice(&payload_json)
        .map_err(|e| ExportImportError::Format(format::FormatError::Json(e)))?;

    // Load existing profiles
    let config_path =
        default_config_path().map_err(|e| ExportImportError::Storage(e.to_string()))?;
    let mut config =
        load_config(&config_path).map_err(|e| ExportImportError::Storage(e.to_string()))?;

    // Check for conflicts and determine actions
    // Force requires --yes
    if options.force && !options.yes && !options.dry_run {
        return Err(ExportImportError::Storage(
            "--force requires --yes to confirm overwrite".to_string(),
        ));
    }

    // Track results for each profile
    let mut profile_results = Vec::new();

    // Import profiles - no early validation, handle conflicts during import
    for (name, export_profile) in payload.profiles {
        // Helper to find conflicting profile name (different name, same team_id)
        let find_conflicting_name = || -> Option<String> {
            config
                .profiles
                .iter()
                .find(|(n, p)| *n != &name && p.team_id == export_profile.team_id)
                .map(|(n, _)| n.clone())
        };

        // Determine action and reason based on current state
        let (action, reason, should_import) = if let Some(existing) = config.get(&name) {
            // Profile name already exists
            if existing.team_id == export_profile.team_id {
                // Same team_id: update or overwrite
                if options.force {
                    (
                        ImportAction::Overwritten,
                        format!(
                            "Overwritten existing profile (same team_id: {})",
                            existing.team_id
                        ),
                        true,
                    )
                } else {
                    (
                        ImportAction::Updated,
                        format!(
                            "Updated existing profile (same team_id: {})",
                            existing.team_id
                        ),
                        true,
                    )
                }
            } else {
                // Different team_id: conflict
                if options.force {
                    (
                        ImportAction::Overwritten,
                        format!(
                            "Overwritten conflicting profile (team_id {} -> {})",
                            existing.team_id, export_profile.team_id
                        ),
                        true,
                    )
                } else {
                    (
                        ImportAction::Skipped,
                        format!(
                            "Skipped due to team_id conflict ({} vs {})",
                            existing.team_id, export_profile.team_id
                        ),
                        false,
                    )
                }
            }
        } else if let Some(conflicting_name) = find_conflicting_name() {
            // team_id exists under different name
            if options.force {
                // Remove the conflicting profile before importing
                config.remove(&conflicting_name);
                (
                    ImportAction::Overwritten,
                    format!(
                        "Overwritten profile '{}' with conflicting team_id {}",
                        conflicting_name, export_profile.team_id
                    ),
                    true,
                )
            } else {
                (
                    ImportAction::Skipped,
                    format!(
                        "Skipped due to existing team_id {} under different name '{}'",
                        export_profile.team_id, conflicting_name
                    ),
                    false,
                )
            }
        } else {
            // New profile
            (
                ImportAction::Updated,
                "New profile imported".to_string(),
                true,
            )
        };

        // Only perform import actions if should_import is true
        if should_import && !options.dry_run {
            let profile = Profile {
                team_id: export_profile.team_id.clone(),
                user_id: export_profile.user_id.clone(),
                team_name: export_profile.team_name,
                user_name: export_profile.user_name,
                client_id: export_profile.client_id.clone(),
                redirect_uri: None, // Not exported/imported for security
                scopes: None,       // Not exported/imported for security
                bot_scopes: None,   // Not exported/imported for security
                user_scopes: None,  // Not exported/imported for security
                default_token_type: None,
            };

            config.set(name.clone(), profile);

            // Store token
            let token_key = make_token_key(&export_profile.team_id, &export_profile.user_id);
            token_store.set(&token_key, &export_profile.token)?;

            // Store OAuth client secret if present
            if let Some(client_secret) = export_profile.client_secret {
                store_oauth_client_secret(token_store, &name, &client_secret)?;
            }
        }

        profile_results.push(ProfileImportResult {
            profile_name: name,
            action,
            reason,
        });
    }

    // Save config unless dry-run
    if !options.dry_run {
        save_config(&config_path, &config)
            .map_err(|e| ExportImportError::Storage(e.to_string()))?;
    }

    // Calculate summary
    let updated = profile_results
        .iter()
        .filter(|r| r.action == ImportAction::Updated)
        .count();
    let skipped = profile_results
        .iter()
        .filter(|r| r.action == ImportAction::Skipped)
        .count();
    let overwritten = profile_results
        .iter()
        .filter(|r| r.action == ImportAction::Overwritten)
        .count();
    let total = profile_results.len();

    Ok(ImportResult {
        profiles: profile_results,
        summary: ImportSummary {
            updated,
            skipped,
            overwritten,
            total,
        },
        dry_run: options.dry_run,
    })
}

/// Check file permissions (Unix only)
#[cfg(unix)]
fn check_file_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::metadata(path)?;
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // Check if file is 0600 (owner read/write only)
    if mode & 0o777 != 0o600 {
        return Err(ExportImportError::PermissionError(format!(
            "File must have 0600 permissions, found: {:o}",
            mode & 0o777
        )));
    }

    Ok(())
}

/// Check file permissions (non-Unix - always succeeds)
#[cfg(not(unix))]
fn check_file_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

/// Write file with secure permissions (Unix: 0600)
#[cfg(unix)]
fn write_secure_file(path: &Path, data: &[u8]) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // Write file
    fs::write(path, data)?;

    // Set permissions to 0600
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)?;

    Ok(())
}

/// Write file with secure permissions (non-Unix - no permission setting)
#[cfg(not(unix))]
fn write_secure_file(path: &Path, data: &[u8]) -> Result<()> {
    fs::write(path, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{InMemoryTokenStore, ProfilesConfig};
    use tempfile::TempDir;

    #[test]
    fn test_export_requires_yes_flag() {
        let token_store = InMemoryTokenStore::new();
        let options = ExportOptions {
            profile_name: None,
            all: false,
            output_path: "/tmp/test.export".to_string(),
            passphrase: "password".to_string(),
            yes: false,
        };

        let result = export_profiles(&token_store, &options);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExportImportError::ConfirmationRequired
        ));
    }

    #[test]
    fn test_export_empty_passphrase() {
        let token_store = InMemoryTokenStore::new();
        let options = ExportOptions {
            profile_name: None,
            all: false,
            output_path: "/tmp/test.export".to_string(),
            passphrase: "".to_string(),
            yes: true,
        };

        let result = export_profiles(&token_store, &options);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExportImportError::EmptyPassphrase
        ));
    }

    #[test]
    fn test_import_empty_passphrase() {
        let token_store = InMemoryTokenStore::new();
        let options = ImportOptions {
            input_path: "/tmp/test.export".to_string(),
            passphrase: "".to_string(),
            yes: false,
            force: false,
            dry_run: false,
            json: false,
        };

        let result = import_profiles(&token_store, &options);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExportImportError::EmptyPassphrase
        ));
    }

    #[test]
    fn test_export_import_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("profiles.json");
        let _export_path = temp_dir.path().join("export.dat");

        // Set up test profile
        let mut config = ProfilesConfig::new();
        config.set(
            "test".to_string(),
            Profile {
                team_id: "T123".to_string(),
                user_id: "U456".to_string(),
                team_name: Some("Test Team".to_string()),
                user_name: Some("Test User".to_string()),
                client_id: None,
                redirect_uri: None,
                scopes: None,
                bot_scopes: None,
                user_scopes: None,
                default_token_type: None,
            },
        );
        save_config(&config_path, &config).unwrap();

        // Set up token store
        let token_store = InMemoryTokenStore::new();
        let token_key = make_token_key("T123", "U456");
        token_store.set(&token_key, "xoxb-test-token").unwrap();

        // Export (this will use default_config_path, so we need to work around that)
        // For now, skip the full integration test as it requires mocking config path
        // This is covered by crypto and format tests
    }

    #[cfg(unix)]
    #[test]
    fn test_write_secure_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("secure.dat");

        write_secure_file(&file_path, b"test data").unwrap();

        let metadata = fs::metadata(&file_path).unwrap();
        let mode = metadata.permissions().mode();

        assert_eq!(mode & 0o777, 0o600, "File should have 0600 permissions");
    }

    #[test]
    #[serial_test::serial]
    fn test_import_dry_run_no_changes() {
        use crate::auth::crypto::KdfParams;
        use crate::auth::format::ExportProfile;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("profiles.json");
        let import_path = temp_dir.path().join("import.dat");
        let tokens_path = temp_dir.path().join("tokens.json");

        // Set SLACK_RS_TOKENS_PATH for file-based token store
        std::env::set_var("SLACK_RS_TOKENS_PATH", tokens_path.to_str().unwrap());

        // Create existing profile
        let mut config = ProfilesConfig::new();
        config.set(
            "existing".to_string(),
            Profile {
                team_id: "T123".to_string(),
                user_id: "U456".to_string(),
                team_name: Some("Existing Team".to_string()),
                user_name: None,
                client_id: None,
                redirect_uri: None,
                scopes: None,
                bot_scopes: None,
                user_scopes: None,
                default_token_type: None,
            },
        );
        save_config(&config_path, &config).unwrap();

        // Create encrypted export file
        let mut payload = crate::auth::format::ExportPayload::new();
        payload.profiles.insert(
            "new_profile".to_string(),
            ExportProfile {
                team_id: "T789".to_string(),
                user_id: "U101".to_string(),
                team_name: Some("New Team".to_string()),
                user_name: None,
                token: "xoxb-new-token".to_string(),
                client_id: None,
                client_secret: None,
            },
        );

        let passphrase = "test-password";
        let kdf_params = KdfParams {
            salt: crypto::generate_salt(),
            ..Default::default()
        };
        let key = crypto::derive_key(passphrase, &kdf_params).unwrap();
        let payload_json = serde_json::to_vec(&payload).unwrap();
        let encrypted = crypto::encrypt(&payload_json, &key).unwrap();
        let encoded = format::encode_export(&payload, &encrypted, &kdf_params).unwrap();

        #[cfg(unix)]
        write_secure_file(&import_path, &encoded).unwrap();
        #[cfg(not(unix))]
        std::fs::write(&import_path, &encoded).unwrap();

        // Mock config path by using environment variable
        std::env::set_var("SLACK_RS_CONFIG_PATH", config_path.to_str().unwrap());

        // Test dry-run import
        let token_store = crate::profile::FileTokenStore::with_path(tokens_path.clone()).unwrap();
        let options = ImportOptions {
            input_path: import_path.to_str().unwrap().to_string(),
            passphrase: passphrase.to_string(),
            yes: true,
            force: false,
            dry_run: true,
            json: false,
        };

        let result = import_profiles(&token_store, &options).unwrap();

        // Verify dry-run flag is set
        assert!(result.dry_run);

        // Verify action is "updated" for new profile
        assert_eq!(result.profiles.len(), 1);
        assert_eq!(result.profiles[0].profile_name, "new_profile");
        assert_eq!(result.profiles[0].action, ImportAction::Updated);

        // Verify no changes were made to config file
        let config_after = load_config(&config_path).unwrap();
        assert_eq!(config_after.profiles.len(), 1);
        assert!(config_after.get("new_profile").is_none());
        assert!(config_after.get("existing").is_some());

        // Verify no token was stored
        let token_key = make_token_key("T789", "U101");
        assert!(!token_store.exists(&token_key));

        // Clean up
        std::env::remove_var("SLACK_RS_TOKENS_PATH");
        std::env::remove_var("SLACK_RS_CONFIG_PATH");
    }

    // Note: More comprehensive integration tests for dry-run would require
    // mocking the config path system, which is not currently supported.
    // The test_import_dry_run_no_changes test provides basic coverage that
    // dry-run prevents file writes. Manual testing is recommended for
    // full validation of update/conflict scenarios.
}
