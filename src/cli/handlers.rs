//! CLI command handlers
//!
//! This module contains handler functions for CLI commands that were extracted from main.rs
//! to improve code organization and maintainability.

use crate::api::{execute_api_call, ApiCallArgs, ApiCallContext, ApiCallResponse, ApiClient};
use crate::auth;
use crate::debug;
use crate::oauth;
use crate::profile::{
    create_token_store, default_config_path, make_token_key, resolve_profile_full, TokenType,
};

/// Parsed login arguments structure
#[derive(Debug, Clone, PartialEq)]
pub struct LoginArgs {
    pub profile_name: Option<String>,
    pub client_id: Option<String>,
    pub bot_scopes: Option<Vec<String>>,
    pub user_scopes: Option<Vec<String>>,
    pub tunnel_mode: TunnelMode,
}

/// Tunnel mode for login
#[derive(Debug, Clone, PartialEq)]
pub enum TunnelMode {
    None,
    Cloudflared(Option<String>),
    Ngrok(Option<String>),
}

impl TunnelMode {
    /// Check if tunnel mode is enabled
    pub fn is_enabled(&self) -> bool {
        !matches!(self, TunnelMode::None)
    }

    /// Check if cloudflared is enabled
    pub fn is_cloudflared(&self) -> bool {
        matches!(self, TunnelMode::Cloudflared(_))
    }

    /// Check if ngrok is enabled
    #[allow(dead_code)]
    pub fn is_ngrok(&self) -> bool {
        matches!(self, TunnelMode::Ngrok(_))
    }
}

/// Parse login command arguments
///
/// This function extracts and validates arguments for the `auth login` command.
/// It enforces mutual exclusivity between --cloudflared and --ngrok flags.
///
/// # Arguments
/// * `args` - Raw command line arguments after "auth login"
///
/// # Returns
/// * `Ok(LoginArgs)` - Successfully parsed and validated arguments
/// * `Err(String)` - Parse error with descriptive message
///
/// # Validation Rules
/// 1. --cloudflared and --ngrok are mutually exclusive
/// 2. Unknown options are rejected
/// 3. Scope inputs are normalized (comma-separated, whitespace-trimmed)
pub fn parse_login_args(args: &[String]) -> Result<LoginArgs, String> {
    let mut profile_name: Option<String> = None;
    let mut client_id: Option<String> = None;
    let mut cloudflared_path: Option<String> = None;
    let mut ngrok_path: Option<String> = None;
    let mut bot_scopes: Option<Vec<String>> = None;
    let mut user_scopes: Option<Vec<String>> = None;

    let mut i = 0;
    while i < args.len() {
        if args[i].starts_with("--") {
            match args[i].as_str() {
                "--client-id" => {
                    i += 1;
                    if i < args.len() {
                        client_id = Some(args[i].clone());
                    } else {
                        return Err("--client-id requires a value".to_string());
                    }
                }
                "--cloudflared" => {
                    // Check if next arg is a value (not starting with --) or end of args
                    if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                        i += 1;
                        cloudflared_path = Some(args[i].clone());
                    } else {
                        // Use default "cloudflared" (PATH resolution)
                        cloudflared_path = Some("cloudflared".to_string());
                    }
                }
                "--ngrok" => {
                    // Check if next arg is a value (not starting with --) or end of args
                    if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                        i += 1;
                        ngrok_path = Some(args[i].clone());
                    } else {
                        // Use default "ngrok" (PATH resolution)
                        ngrok_path = Some("ngrok".to_string());
                    }
                }
                "--bot-scopes" => {
                    i += 1;
                    if i < args.len() {
                        let scopes_input: Vec<String> =
                            args[i].split(',').map(|s| s.trim().to_string()).collect();
                        // Expand 'all' presets with bot context (true)
                        bot_scopes = Some(oauth::expand_scopes_with_context(&scopes_input, true));
                    } else {
                        return Err("--bot-scopes requires a value".to_string());
                    }
                }
                "--user-scopes" => {
                    i += 1;
                    if i < args.len() {
                        let scopes_input: Vec<String> =
                            args[i].split(',').map(|s| s.trim().to_string()).collect();
                        // Expand 'all' presets with user context (false)
                        user_scopes = Some(oauth::expand_scopes_with_context(&scopes_input, false));
                    } else {
                        return Err("--user-scopes requires a value".to_string());
                    }
                }
                _ => {
                    return Err(format!("Unknown option: {}", args[i]));
                }
            }
        } else if profile_name.is_none() {
            profile_name = Some(args[i].clone());
        } else {
            return Err(format!("Unexpected argument: {}", args[i]));
        }
        i += 1;
    }

    // Check for conflicting options
    if cloudflared_path.is_some() && ngrok_path.is_some() {
        return Err("Cannot specify both --cloudflared and --ngrok at the same time".to_string());
    }

    // Determine tunnel mode
    let tunnel_mode = if let Some(path) = cloudflared_path {
        TunnelMode::Cloudflared(Some(path))
    } else if let Some(path) = ngrok_path {
        TunnelMode::Ngrok(Some(path))
    } else {
        TunnelMode::None
    };

    Ok(LoginArgs {
        profile_name,
        client_id,
        bot_scopes,
        user_scopes,
        tunnel_mode,
    })
}

/// Run the auth login command with argument parsing
pub async fn run_auth_login(args: &[String], non_interactive: bool) -> Result<(), String> {
    // Parse arguments
    let parsed_args = parse_login_args(args)?;

    // Use default redirect_uri
    let redirect_uri = "http://127.0.0.1:8765/callback".to_string();

    // Keep base_url from environment for testing purposes only
    let base_url = std::env::var("SLACK_OAUTH_BASE_URL").ok();

    // If cloudflared or ngrok is specified, use extended login flow
    if parsed_args.tunnel_mode.is_enabled() {
        // Collect missing parameters in non-interactive mode
        if non_interactive {
            let mut missing = Vec::new();
            if parsed_args.client_id.is_none() {
                missing.push("--client-id");
            }
            if parsed_args.bot_scopes.is_none() {
                missing.push("--bot-scopes");
            }
            if parsed_args.user_scopes.is_none() {
                missing.push("--user-scopes");
            }
            if !missing.is_empty() {
                return Err(format!(
                    "Missing required parameters in non-interactive mode: {}\n\
                    Provide them via CLI flags:\n\
                    Example: slack-rs auth login --cloudflared --client-id <id> --bot-scopes <scopes> --user-scopes <scopes>",
                    missing.join(", ")
                ));
            }
        }

        // Prompt for client_id if not provided (only in interactive mode)
        let client_id = if let Some(id) = parsed_args.client_id {
            id
        } else if non_interactive {
            return Err(
                "Client ID is required in non-interactive mode. Use --client-id flag.".to_string(),
            );
        } else {
            use std::io::{self, Write};
            print!("Enter Slack Client ID: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        };

        // Use default scopes if not provided
        let bot_scopes = parsed_args.bot_scopes.unwrap_or_else(oauth::bot_all_scopes);
        let user_scopes = parsed_args
            .user_scopes
            .unwrap_or_else(oauth::user_all_scopes);

        if debug::enabled() {
            debug::log("Preparing to call login_with_credentials_extended");
            debug::log(format!("bot_scopes_count={}", bot_scopes.len()));
            debug::log(format!("user_scopes_count={}", user_scopes.len()));
        }

        // Prompt for client_secret (only in interactive mode)
        let client_secret = if non_interactive {
            return Err("Client secret cannot be provided in non-interactive mode with --cloudflared/--ngrok. Use the standard login flow (without --cloudflared/--ngrok) to save credentials first.".to_string());
        } else {
            auth::prompt_for_client_secret()
                .map_err(|e| format!("Failed to read client secret: {}", e))?
        };

        // Call extended login with cloudflared support
        auth::login_with_credentials_extended(
            client_id,
            client_secret,
            bot_scopes,
            user_scopes,
            parsed_args.profile_name,
            parsed_args.tunnel_mode.is_cloudflared(),
        )
        .await
        .map_err(|e| e.to_string())
    } else {
        // Call standard login with credentials
        // This will prompt for client_secret and other missing OAuth config
        auth::login_with_credentials(
            parsed_args.client_id,
            parsed_args.profile_name,
            redirect_uri,
            vec![], // Legacy scopes parameter (unused)
            parsed_args.bot_scopes,
            parsed_args.user_scopes,
            base_url,
            non_interactive,
        )
        .await
        .map_err(|e| e.to_string())
    }
}

/// Check if we should show private channel guidance
fn should_show_private_channel_guidance(
    api_args: &ApiCallArgs,
    token_type: &str,
    response: &ApiCallResponse,
) -> bool {
    // Only show guidance for conversations.list with private_channel type and bot token
    if api_args.method != "conversations.list" || token_type != "bot" {
        return false;
    }

    // Check if types parameter includes private_channel
    if let Some(types) = api_args.params.get("types") {
        if !types.contains("private_channel") {
            return false;
        }
    } else {
        return false;
    }

    // Check if response has empty channels array
    if let Some(channels) = response.response.get("channels") {
        if let Some(channels_array) = channels.as_array() {
            return channels_array.is_empty();
        }
    }

    false
}

/// Infer the default token type based on token store existence
/// Returns User if a user token exists, otherwise Bot
fn infer_default_token_type(
    token_store: &dyn crate::profile::TokenStore,
    team_id: &str,
    user_id: &str,
) -> TokenType {
    let user_token_key = format!("{}:{}:user", team_id, user_id);
    if token_store.exists(&user_token_key) {
        TokenType::User
    } else {
        TokenType::Bot
    }
}

/// Run the api call command
pub async fn run_api_call(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments
    let api_args = ApiCallArgs::parse(&args)?;

    // Resolve profile name using common helper (--profile > SLACK_PROFILE > "default")
    let profile_name = crate::cli::resolve_profile_name(&args);

    // Get config path
    let config_path = default_config_path()?;

    // Resolve profile to get full profile details
    let profile = resolve_profile_full(&config_path, &profile_name)
        .map_err(|e| format!("Failed to resolve profile '{}': {}", profile_name, e))?;

    // Create context from resolved profile
    let context = ApiCallContext {
        profile_name: Some(profile_name.clone()),
        team_id: profile.team_id.clone(),
        user_id: profile.user_id.clone(),
    };

    // Create token store to check token existence for inference
    let token_store =
        create_token_store().map_err(|e| format!("Failed to create token store: {}", e))?;

    // Infer default token type based on user token existence
    let inferred_default =
        infer_default_token_type(&*token_store, &profile.team_id, &profile.user_id);

    // Resolve token type: CLI flag > profile default > inferred default (user if exists, else bot)
    let resolved_token_type = TokenType::resolve(
        api_args.token_type,
        profile.default_token_type,
        inferred_default,
    );

    // Create token key from team_id, user_id, and token type
    // User token key format: {team_id}:{user_id}:user (matches auth/commands.rs storage format)
    let token_key_bot = make_token_key(&profile.team_id, &profile.user_id);
    let token_key_user = format!("{}:{}:user", profile.team_id, profile.user_id);

    // Select the appropriate token key based on resolved token type
    let token_key = match resolved_token_type {
        TokenType::Bot => token_key_bot.clone(),
        TokenType::User => token_key_user.clone(),
    };

    // Retrieve token from token store
    // Try token store first, fall back to environment variable only for the requested token type

    // Determine if the token type was explicitly requested via CLI flag OR default_token_type
    // If either is set, we should NOT fallback to a different token type
    let explicit_request = api_args.token_type.is_some() || profile.default_token_type.is_some();

    let token = match token_store.get(&token_key) {
        Ok(t) => t,
        Err(_) => {
            // If token not found in store, check environment variable
            if let Ok(env_token) = std::env::var("SLACK_TOKEN") {
                env_token
            } else if explicit_request {
                // If token type was explicitly requested (via --token-type or default_token_type), fail without fallback
                return Err(format!(
                    "No {} token found for profile '{}' ({}:{}). Explicitly requested token type not available. Set SLACK_TOKEN environment variable or run 'slack login' to obtain a {} token.",
                    resolved_token_type, profile_name, profile.team_id, profile.user_id, resolved_token_type
                ).into());
            } else {
                // If no token type preference was specified at all, try bot token as fallback
                if resolved_token_type == TokenType::User {
                    if let Ok(bot_token) = token_store.get(&token_key_bot) {
                        eprintln!(
                            "Warning: User token not found, falling back to bot token for profile '{}'",
                            profile_name
                        );
                        bot_token
                    } else {
                        return Err(format!(
                            "No {} token found for profile '{}' ({}:{}). Set SLACK_TOKEN environment variable or run 'slack login' to obtain a token.",
                            resolved_token_type, profile_name, profile.team_id, profile.user_id
                        ).into());
                    }
                } else {
                    return Err(format!(
                        "No {} token found for profile '{}' ({}:{}). Set SLACK_TOKEN environment variable or run 'slack login' to obtain a token.",
                        resolved_token_type, profile_name, profile.team_id, profile.user_id
                    ).into());
                }
            }
        }
    };

    // Get debug level from args
    let debug_level = debug::get_debug_level(&args);

    // Log debug information if --debug or --trace flag is present
    let token_store_backend = if std::env::var("SLACK_TOKEN").is_ok() {
        "environment"
    } else {
        "file"
    };

    let endpoint = format!("https://slack.com/api/{}", api_args.method);

    debug::log_api_context(
        debug_level,
        Some(&profile_name),
        token_store_backend,
        resolved_token_type.as_str(),
        &api_args.method,
        &endpoint,
    );

    // Create API client
    let client = ApiClient::new();

    // Execute API call with token type information and command name
    let response = execute_api_call(
        &client,
        &api_args,
        &token,
        &context,
        resolved_token_type.as_str(),
        "api call",
    )
    .await?;

    // Log error code if present
    debug::log_error_code(debug_level, &response.response);

    // Display error guidance if response contains a known error
    crate::api::display_error_guidance(&response);

    // Check if we should show guidance for private_channel with bot token
    if should_show_private_channel_guidance(&api_args, resolved_token_type.as_str(), &response) {
        eprintln!();
        eprintln!("Note: The conversation list for private channels is empty.");
        eprintln!("Bot tokens can only see private channels where the bot is a member.");
        eprintln!("To list all private channels, use a User Token with appropriate scopes.");
        eprintln!("Run: slackcli auth login (with user_scopes) or use --token-type user");
        eprintln!();
    }

    // Print response as JSON
    // If --raw flag is set or SLACKRS_OUTPUT=raw, output only the Slack API response without envelope
    // Note: api_args.raw already accounts for both --raw flag and SLACKRS_OUTPUT env via should_output_raw()
    let json = if api_args.raw {
        serde_json::to_string_pretty(&response.response)?
    } else {
        serde_json::to_string_pretty(&response)?
    };
    println!("{}", json);

    Ok(())
}

/// Common arguments shared between export and import commands
struct ExportImportArgs {
    passphrase_env: Option<String>,
    yes: bool,
    lang: Option<String>,
}

impl ExportImportArgs {
    /// Parse common arguments from command line args
    /// Returns (ExportImportArgs, remaining_unparsed_args)
    fn parse(args: &[String]) -> (Self, Vec<(usize, String)>) {
        let mut passphrase_env: Option<String> = None;
        let mut yes = false;
        let mut lang: Option<String> = None;
        let mut remaining = Vec::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--passphrase-env" => {
                    i += 1;
                    if i < args.len() {
                        passphrase_env = Some(args[i].clone());
                    }
                }
                "--passphrase-prompt" => {
                    // Ignore this flag - we always prompt if --passphrase-env is not set
                }
                "--yes" => {
                    yes = true;
                }
                "--lang" => {
                    i += 1;
                    if i < args.len() {
                        lang = Some(args[i].clone());
                    }
                }
                _ => {
                    // Not a common argument, save for specific parsing
                    remaining.push((i, args[i].clone()));
                }
            }
            i += 1;
        }

        (
            Self {
                passphrase_env,
                yes,
                lang,
            },
            remaining,
        )
    }

    /// Get Messages based on language setting
    fn get_messages(&self) -> auth::Messages {
        if let Some(ref lang_code) = self.lang {
            if let Some(language) = auth::Language::from_code(lang_code) {
                auth::Messages::new(language)
            } else {
                auth::Messages::default()
            }
        } else {
            auth::Messages::default()
        }
    }

    /// Get passphrase from environment variable or prompt
    fn get_passphrase(&self, messages: &auth::Messages) -> Result<String, String> {
        if let Some(ref env_var) = self.passphrase_env {
            match std::env::var(env_var) {
                Ok(val) => Ok(val),
                Err(_) => {
                    // Fallback to prompt if environment variable is not set
                    eprintln!(
                        "Warning: Environment variable {} not found, prompting for passphrase",
                        env_var
                    );
                    rpassword::prompt_password(messages.get("prompt.passphrase"))
                        .map_err(|e| format!("Error reading passphrase: {}", e))
                }
            }
        } else {
            // Fallback to prompt mode
            rpassword::prompt_password(messages.get("prompt.passphrase"))
                .map_err(|e| format!("Error reading passphrase: {}", e))
        }
    }
}

/// Handle auth export command
pub async fn handle_export_command(args: &[String]) {
    // Check for help flags first
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        super::help::print_export_help();
        return;
    }

    // Parse common arguments
    let (common_args, remaining) = ExportImportArgs::parse(args);

    // Parse export-specific arguments
    let mut profile_name: Option<String> = None;
    let mut all = false;
    let mut output_path: Option<String> = None;

    for (idx, arg) in remaining {
        match arg.as_str() {
            "--profile" => {
                // Next arg should be the profile name
                if idx + 1 < args.len() {
                    profile_name = Some(args[idx + 1].clone());
                }
            }
            "--all" => {
                all = true;
            }
            "--out" => {
                // Next arg should be the output path
                if idx + 1 < args.len() {
                    output_path = Some(args[idx + 1].clone());
                }
            }
            _ => {
                // Check if this is a value for a previous flag
                if idx > 0 {
                    let prev = &args[idx - 1];
                    if prev == "--profile"
                        || prev == "--out"
                        || prev == "--passphrase-env"
                        || prev == "--lang"
                    {
                        // This is a value, not an unknown option
                        continue;
                    }
                }
                eprintln!("Unknown option: {}", arg);
                std::process::exit(1);
            }
        }
    }

    // Get i18n messages
    let messages = common_args.get_messages();

    // Show warning and validate --yes
    if !common_args.yes {
        eprintln!("{}", messages.get("warn.export_sensitive"));
        eprintln!("Error: --yes flag is required to confirm this dangerous operation");
        std::process::exit(1);
    }

    // Validate required options
    let output = match output_path {
        Some(path) => path,
        None => {
            eprintln!("Error: --out <file> is required");
            std::process::exit(1);
        }
    };

    // Get passphrase
    let passphrase = match common_args.get_passphrase(&messages) {
        Ok(pass) => pass,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let options = auth::ExportOptions {
        profile_name,
        all,
        output_path: output,
        passphrase,
        yes: common_args.yes,
    };

    let token_store = create_token_store().expect("Failed to create token store");
    match auth::export_profiles(&*token_store, &options) {
        Ok(_) => {
            println!("{}", messages.get("success.export"));
        }
        Err(e) => {
            eprintln!("Export failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle auth import command
pub async fn handle_import_command(args: &[String]) {
    // Check for help flags first
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        super::help::print_import_help();
        return;
    }

    // Parse common arguments
    let (common_args, remaining) = ExportImportArgs::parse(args);

    // Parse import-specific arguments
    let mut input_path: Option<String> = None;
    let mut force = false;
    let mut dry_run = false;
    let mut json = false;

    for (idx, arg) in remaining {
        match arg.as_str() {
            "--in" => {
                // Next arg should be the input path
                if idx + 1 < args.len() {
                    input_path = Some(args[idx + 1].clone());
                }
            }
            "--force" => {
                force = true;
            }
            "--dry-run" => {
                dry_run = true;
            }
            "--json" => {
                json = true;
            }
            _ => {
                // Check if this is a value for a previous flag
                if idx > 0 {
                    let prev = &args[idx - 1];
                    if prev == "--in" || prev == "--passphrase-env" || prev == "--lang" {
                        // This is a value, not an unknown option
                        continue;
                    }
                }
                eprintln!("Unknown option: {}", arg);
                std::process::exit(1);
            }
        }
    }

    // Get i18n messages
    let messages = common_args.get_messages();

    // Validate required options
    let input = match input_path {
        Some(path) => path,
        None => {
            eprintln!("Error: --in <file> is required");
            std::process::exit(1);
        }
    };

    // Get passphrase
    let passphrase = match common_args.get_passphrase(&messages) {
        Ok(pass) => pass,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let options = auth::ImportOptions {
        input_path: input,
        passphrase,
        yes: common_args.yes,
        force,
        dry_run,
        json,
    };

    let token_store = create_token_store().expect("Failed to create token store");
    match auth::import_profiles(&*token_store, &options) {
        Ok(result) => {
            if json {
                // Output JSON format
                match serde_json::to_string_pretty(&result) {
                    Ok(json_output) => {
                        println!("{}", json_output);
                    }
                    Err(e) => {
                        eprintln!("Failed to serialize result to JSON: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Output text format
                if result.dry_run {
                    println!("Dry-run mode: no changes were written.");
                    println!();
                }

                println!("Import Summary:");
                println!("  Total: {}", result.summary.total);
                println!("  Updated: {}", result.summary.updated);
                println!("  Skipped: {}", result.summary.skipped);
                println!("  Overwritten: {}", result.summary.overwritten);
                println!();
                println!("Profile Details:");
                for profile_result in &result.profiles {
                    println!(
                        "  {} - {} ({})",
                        profile_result.profile_name, profile_result.action, profile_result.reason
                    );
                }
                println!();

                if result.dry_run {
                    println!("Dry-run complete. Re-run without --dry-run to apply changes.");
                } else {
                    println!("{}", messages.get("success.import"));
                }
            }
        }
        Err(e) => {
            eprintln!("Import failed: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::call::ApiCallMeta;
    use crate::profile::{InMemoryTokenStore, TokenStore};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_parse_login_args_empty() {
        let args = vec![];
        let result = parse_login_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.profile_name, None);
        assert_eq!(parsed.client_id, None);
        assert_eq!(parsed.bot_scopes, None);
        assert_eq!(parsed.user_scopes, None);
        assert_eq!(parsed.tunnel_mode, TunnelMode::None);
    }

    #[test]
    fn test_parse_login_args_profile_only() {
        let args = vec!["my-profile".to_string()];
        let result = parse_login_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.profile_name, Some("my-profile".to_string()));
        assert_eq!(parsed.tunnel_mode, TunnelMode::None);
    }

    #[test]
    fn test_parse_login_args_with_client_id() {
        let args = vec!["--client-id".to_string(), "123.456".to_string()];
        let result = parse_login_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.client_id, Some("123.456".to_string()));
    }

    #[test]
    fn test_parse_login_args_cloudflared_default() {
        let args = vec!["--cloudflared".to_string()];
        let result = parse_login_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(matches!(
            parsed.tunnel_mode,
            TunnelMode::Cloudflared(Some(_))
        ));
        if let TunnelMode::Cloudflared(Some(path)) = parsed.tunnel_mode {
            assert_eq!(path, "cloudflared");
        }
    }

    #[test]
    fn test_parse_login_args_cloudflared_with_path() {
        let args = vec![
            "--cloudflared".to_string(),
            "/usr/bin/cloudflared".to_string(),
        ];
        let result = parse_login_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        if let TunnelMode::Cloudflared(Some(path)) = parsed.tunnel_mode {
            assert_eq!(path, "/usr/bin/cloudflared");
        } else {
            panic!("Expected Cloudflared tunnel mode");
        }
    }

    #[test]
    fn test_parse_login_args_ngrok_default() {
        let args = vec!["--ngrok".to_string()];
        let result = parse_login_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(matches!(parsed.tunnel_mode, TunnelMode::Ngrok(Some(_))));
        if let TunnelMode::Ngrok(Some(path)) = parsed.tunnel_mode {
            assert_eq!(path, "ngrok");
        }
    }

    #[test]
    fn test_parse_login_args_cloudflared_ngrok_mutual_exclusion() {
        let args = vec!["--cloudflared".to_string(), "--ngrok".to_string()];
        let result = parse_login_args(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot specify both --cloudflared and --ngrok"));
    }

    #[test]
    fn test_parse_login_args_bot_scopes() {
        let args = vec![
            "--bot-scopes".to_string(),
            "chat:write,users:read".to_string(),
        ];
        let result = parse_login_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.bot_scopes.is_some());
        let scopes = parsed.bot_scopes.unwrap();
        assert!(scopes.contains(&"chat:write".to_string()));
        assert!(scopes.contains(&"users:read".to_string()));
    }

    #[test]
    fn test_parse_login_args_user_scopes() {
        let args = vec![
            "--user-scopes".to_string(),
            "search:read,users:read".to_string(),
        ];
        let result = parse_login_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.user_scopes.is_some());
    }

    #[test]
    fn test_parse_login_args_all_parameters() {
        let args = vec![
            "work".to_string(),
            "--client-id".to_string(),
            "123.456".to_string(),
            "--bot-scopes".to_string(),
            "chat:write".to_string(),
            "--user-scopes".to_string(),
            "users:read".to_string(),
            "--cloudflared".to_string(),
        ];
        let result = parse_login_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.profile_name, Some("work".to_string()));
        assert_eq!(parsed.client_id, Some("123.456".to_string()));
        assert!(parsed.bot_scopes.is_some());
        assert!(parsed.user_scopes.is_some());
        assert!(parsed.tunnel_mode.is_cloudflared());
    }

    #[test]
    fn test_parse_login_args_unknown_option() {
        let args = vec!["--unknown-flag".to_string()];
        let result = parse_login_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown option"));
    }

    #[test]
    fn test_parse_login_args_unexpected_positional() {
        let args = vec!["profile1".to_string(), "profile2".to_string()];
        let result = parse_login_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unexpected argument"));
    }

    #[test]
    fn test_parse_login_args_client_id_missing_value() {
        let args = vec!["--client-id".to_string()];
        let result = parse_login_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--client-id requires a value"));
    }

    #[test]
    fn test_parse_login_args_bot_scopes_missing_value() {
        let args = vec!["--bot-scopes".to_string()];
        let result = parse_login_args(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("--bot-scopes requires a value"));
    }

    #[test]
    fn test_tunnel_mode_none() {
        let mode = TunnelMode::None;
        assert!(!mode.is_enabled());
        assert!(!mode.is_cloudflared());
        assert!(!mode.is_ngrok());
    }

    #[test]
    fn test_tunnel_mode_cloudflared() {
        let mode = TunnelMode::Cloudflared(Some("cloudflared".to_string()));
        assert!(mode.is_enabled());
        assert!(mode.is_cloudflared());
        assert!(!mode.is_ngrok());
    }

    #[test]
    fn test_tunnel_mode_ngrok() {
        let mode = TunnelMode::Ngrok(Some("ngrok".to_string()));
        assert!(mode.is_enabled());
        assert!(!mode.is_cloudflared());
        assert!(mode.is_ngrok());
    }

    #[test]
    fn test_should_show_private_channel_guidance_empty_response() {
        let mut params = HashMap::new();
        params.insert("types".to_string(), "private_channel".to_string());

        let args = ApiCallArgs {
            method: "conversations.list".to_string(),
            params,
            use_json: false,
            use_get: false,
            token_type: None,
            raw: false,
        };

        let response = ApiCallResponse {
            response: json!({
                "ok": true,
                "channels": []
            }),
            meta: ApiCallMeta {
                profile_name: Some("default".to_string()),
                team_id: "T123".to_string(),
                user_id: "U123".to_string(),
                method: "conversations.list".to_string(),
                command: "api call".to_string(),
                token_type: "bot".to_string(),
            },
        };

        // Should show guidance when bot token returns empty private channels
        assert!(should_show_private_channel_guidance(
            &args, "bot", &response
        ));
    }

    #[test]
    fn test_should_show_private_channel_guidance_non_empty_response() {
        let mut params = HashMap::new();
        params.insert("types".to_string(), "private_channel".to_string());

        let args = ApiCallArgs {
            method: "conversations.list".to_string(),
            params,
            use_json: false,
            use_get: false,
            token_type: None,
            raw: false,
        };

        let response = ApiCallResponse {
            response: json!({
                "ok": true,
                "channels": [
                    {"id": "C123", "name": "private-channel"}
                ]
            }),
            meta: ApiCallMeta {
                profile_name: Some("default".to_string()),
                team_id: "T123".to_string(),
                user_id: "U123".to_string(),
                method: "conversations.list".to_string(),
                command: "api call".to_string(),
                token_type: "bot".to_string(),
            },
        };

        // Should not show guidance when channels are returned
        assert!(!should_show_private_channel_guidance(
            &args, "bot", &response
        ));
    }

    #[test]
    fn test_should_show_private_channel_guidance_user_token() {
        let mut params = HashMap::new();
        params.insert("types".to_string(), "private_channel".to_string());

        let args = ApiCallArgs {
            method: "conversations.list".to_string(),
            params,
            use_json: false,
            use_get: false,
            token_type: None,
            raw: false,
        };

        let response = ApiCallResponse {
            response: json!({
                "ok": true,
                "channels": []
            }),
            meta: ApiCallMeta {
                profile_name: Some("default".to_string()),
                team_id: "T123".to_string(),
                user_id: "U123".to_string(),
                method: "conversations.list".to_string(),
                command: "api call".to_string(),
                token_type: "user".to_string(),
            },
        };

        // Should not show guidance when using user token
        assert!(!should_show_private_channel_guidance(
            &args, "user", &response
        ));
    }

    #[test]
    fn test_infer_default_token_type_with_user_token() {
        let token_store = InMemoryTokenStore::new();
        let team_id = "T123";
        let user_id = "U456";

        // Set a user token
        token_store
            .set(
                &format!("{}:{}:user", team_id, user_id),
                "xoxp-test-user-token",
            )
            .unwrap();

        // Should infer User when user token exists
        let inferred = infer_default_token_type(&token_store, team_id, user_id);
        assert_eq!(inferred, TokenType::User);
    }

    #[test]
    fn test_infer_default_token_type_without_user_token() {
        let token_store = InMemoryTokenStore::new();
        let team_id = "T123";
        let user_id = "U456";

        // Set only a bot token
        token_store
            .set(&format!("{}:{}", team_id, user_id), "xoxb-test-bot-token")
            .unwrap();

        // Should infer Bot when user token does not exist
        let inferred = infer_default_token_type(&token_store, team_id, user_id);
        assert_eq!(inferred, TokenType::Bot);
    }

    #[test]
    fn test_infer_default_token_type_with_both_tokens() {
        let token_store = InMemoryTokenStore::new();
        let team_id = "T123";
        let user_id = "U456";

        // Set both tokens
        token_store
            .set(&format!("{}:{}", team_id, user_id), "xoxb-test-bot-token")
            .unwrap();
        token_store
            .set(
                &format!("{}:{}:user", team_id, user_id),
                "xoxp-test-user-token",
            )
            .unwrap();

        // Should infer User when user token exists (even if bot token also exists)
        let inferred = infer_default_token_type(&token_store, team_id, user_id);
        assert_eq!(inferred, TokenType::User);
    }

    #[test]
    fn test_infer_default_token_type_with_no_tokens() {
        let token_store = InMemoryTokenStore::new();
        let team_id = "T123";
        let user_id = "U456";

        // No tokens set

        // Should infer Bot when no tokens exist
        let inferred = infer_default_token_type(&token_store, team_id, user_id);
        assert_eq!(inferred, TokenType::Bot);
    }
}
