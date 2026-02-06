//! CLI command routing and handlers

mod context;
mod handlers;
pub mod introspection;

pub use context::CliContext;
pub use handlers::{handle_export_command, handle_import_command, run_api_call, run_auth_login};
pub use introspection::{
    generate_commands_list, generate_help, generate_schema, CommandDef, CommandsListResponse,
    HelpResponse, SchemaResponse,
};

use crate::api::{ApiClient, CommandResponse};
use crate::commands;
use crate::commands::ConversationSelector;
use crate::profile::{
    create_token_store, default_config_path, load_config, make_token_key, resolve_profile_full,
    TokenStore, TokenType,
};
use serde_json::Value;

/// Resolve token with priority: SLACK_TOKEN env > token store
///
/// # Arguments
/// * `slack_token_env` - Value of SLACK_TOKEN environment variable (None if unset)
/// * `token_store` - Token store to retrieve tokens from
/// * `token_key` - Key to use for token store lookup
/// * `fallback_token_key` - Optional fallback key (e.g., bot token when user token not found)
/// * `explicit_request` - Whether the token type was explicitly requested (via --token-type or default_token_type)
///
/// # Returns
/// * `Ok(token)` - Successfully resolved token
/// * `Err(message)` - Token resolution failed
///
/// # Token Resolution Priority
/// 1. SLACK_TOKEN environment variable (if set, bypasses token store)
/// 2. Token store with primary token_key
/// 3. Token store with fallback_token_key (only if not explicit_request)
/// 4. Error if no token found
#[allow(dead_code)]
pub fn resolve_token_for_wrapper(
    slack_token_env: Option<String>,
    token_store: &dyn TokenStore,
    token_key: &str,
    fallback_token_key: Option<&str>,
    explicit_request: bool,
) -> Result<String, String> {
    // Priority 1: SLACK_TOKEN environment variable
    if let Some(env_token) = slack_token_env {
        return Ok(env_token);
    }

    // Priority 2: Token store with primary key
    if let Ok(token) = token_store.get(token_key) {
        return Ok(token);
    }

    // Priority 3: Fallback token (only if not explicit_request)
    if !explicit_request {
        if let Some(fallback_key) = fallback_token_key {
            if let Ok(token) = token_store.get(fallback_key) {
                eprintln!("Warning: Primary token not found, falling back to alternative token");
                return Ok(token);
            }
        }
    }

    // Priority 4: Error
    if explicit_request {
        Err(
            "No token found for explicitly requested token type. Set SLACK_TOKEN environment variable or run 'slack login' to obtain a token.".to_string()
        )
    } else {
        Err(
            "No token found. Set SLACK_TOKEN environment variable or run 'slack login' to obtain a token.".to_string()
        )
    }
}

/// Get API client for a profile with optional token type selection
///
/// # Arguments
/// * `profile_name` - Optional profile name (defaults to "default")
/// * `token_type` - Optional token type (bot/user). If None, uses profile default or bot fallback
///
/// # Token Resolution Priority
/// 1. SLACK_TOKEN environment variable (if set, bypasses token store)
/// 2. CLI flag token_type parameter (if provided)
/// 3. Profile's default_token_type (if set)
/// 4. Try user token first, fall back to bot token
pub async fn get_api_client_with_token_type(
    profile_name: Option<String>,
    token_type: Option<TokenType>,
) -> Result<ApiClient, String> {
    // Check for SLACK_TOKEN environment variable first
    if let Ok(env_token) = std::env::var("SLACK_TOKEN") {
        return Ok(ApiClient::with_token(env_token));
    }

    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());
    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let config = load_config(&config_path).map_err(|e| e.to_string())?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    let token_store = create_token_store().map_err(|e| e.to_string())?;

    // Resolve token type: CLI flag > profile default > try user first with bot fallback
    let resolved_token_type = token_type.or(profile.default_token_type);

    let bot_token_key = make_token_key(&profile.team_id, &profile.user_id);
    let user_token_key = format!("{}:{}:user", profile.team_id, profile.user_id);

    let token = match resolved_token_type {
        Some(TokenType::Bot) => {
            // Explicitly requested bot token
            token_store
                .get(&bot_token_key)
                .map_err(|e| format!("Failed to get bot token: {}", e))?
        }
        Some(TokenType::User) => {
            // Explicitly requested user token
            token_store
                .get(&user_token_key)
                .map_err(|e| format!("Failed to get user token: {}", e))?
        }
        None => {
            // No explicit preference, try user token first (for APIs that require user scope)
            match token_store.get(&user_token_key) {
                Ok(user_token) => user_token,
                Err(_) => {
                    // Fall back to bot token
                    token_store
                        .get(&bot_token_key)
                        .map_err(|e| format!("Failed to get token: {}", e))?
                }
            }
        }
    };

    Ok(ApiClient::with_token(token))
}

/// Get API client for a profile (legacy function, maintains backward compatibility)
#[allow(dead_code)]
pub async fn get_api_client(profile_name: Option<String>) -> Result<ApiClient, String> {
    get_api_client_with_token_type(profile_name, None).await
}

/// Check if a flag exists in args
pub fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

/// Check if error message indicates non-interactive mode failure
pub fn is_non_interactive_error(error_msg: &str) -> bool {
    error_msg.contains("Non-interactive mode error")
        || error_msg.contains("Use --yes flag to confirm in non-interactive mode")
}

/// Wrap response with unified envelope including metadata
#[allow(dead_code)]
pub async fn wrap_with_envelope(
    response: Value,
    method: &str,
    command: &str,
    profile_name: Option<String>,
) -> Result<CommandResponse, String> {
    wrap_with_envelope_and_token_type(response, method, command, profile_name, None).await
}

/// Wrap response with unified envelope including metadata and explicit token type
pub async fn wrap_with_envelope_and_token_type(
    response: Value,
    method: &str,
    command: &str,
    profile_name: Option<String>,
    explicit_token_type: Option<TokenType>,
) -> Result<CommandResponse, String> {
    let profile_name_str = profile_name.unwrap_or_else(|| "default".to_string());
    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let profile = resolve_profile_full(&config_path, &profile_name_str)
        .map_err(|e| format!("Failed to resolve profile '{}': {}", profile_name_str, e))?;

    // Resolve token type for metadata
    let token_type_str = if let Some(explicit) = explicit_token_type {
        // If explicitly specified via --token-type, use that
        Some(explicit.to_string())
    } else if std::env::var("SLACK_TOKEN").is_ok() {
        // If using SLACK_TOKEN, use profile's default_token_type if set, otherwise "bot"
        Some(
            profile
                .default_token_type
                .map(|t| t.to_string())
                .unwrap_or_else(|| "bot".to_string()),
        )
    } else {
        // Resolve from token store (check which token exists)
        let token_store = create_token_store().map_err(|e| e.to_string())?;
        let bot_token_key = make_token_key(&profile.team_id, &profile.user_id);
        let user_token_key = format!("{}:{}:user", profile.team_id, profile.user_id);

        // Try to determine which token was used based on default_token_type
        let resolved_type = profile.default_token_type.or_else(|| {
            // If no default, check which token exists (try user first, then bot)
            if token_store.get(&user_token_key).is_ok() {
                Some(TokenType::User)
            } else if token_store.get(&bot_token_key).is_ok() {
                Some(TokenType::Bot)
            } else {
                None
            }
        });

        resolved_type.map(|t| t.to_string())
    };

    Ok(CommandResponse::with_token_type(
        response,
        Some(profile_name_str),
        profile.team_id,
        profile.user_id,
        method.to_string(),
        command.to_string(),
        token_type_str,
    ))
}

/// Resolve profile name with priority: --profile flag > SLACK_PROFILE env > "default"
///
/// This function implements the unified profile selection logic across all CLI commands.
/// It searches for `--profile` in any position within the args array, supporting both
/// `--profile=name` and `--profile name` formats.
///
/// # Arguments
/// * `args` - Command line arguments (including subcommands and flags)
///
/// # Returns
/// Profile name resolved according to priority rules
///
/// # Priority
/// 1. `--profile` flag from command line (either format)
/// 2. `SLACK_PROFILE` environment variable
/// 3. "default" as fallback
pub fn resolve_profile_name(args: &[String]) -> String {
    // Priority 1: Check for --profile flag in args
    if let Some(profile) = get_option(args, "--profile=") {
        return profile;
    }

    // Priority 2: Check SLACK_PROFILE environment variable
    if let Ok(profile) = std::env::var("SLACK_PROFILE") {
        return profile;
    }

    // Priority 3: Default to "default"
    "default".to_string()
}

/// Get option value from args
/// Supports both --key=value and --key value formats
/// When using space-separated format, value must not start with '-'
pub fn get_option(args: &[String], prefix: &str) -> Option<String> {
    // First try --key=value format
    if let Some(value) = args
        .iter()
        .find(|arg| arg.starts_with(prefix))
        .and_then(|arg| arg.strip_prefix(prefix))
        .map(|s| s.to_string())
    {
        return Some(value);
    }

    // Then try --key value format (space-separated)
    // Extract the flag name without the '=' suffix
    let flag = prefix.strip_suffix('=').unwrap_or(prefix);
    if let Some(pos) = args.iter().position(|arg| arg == flag) {
        if let Some(value) = args.get(pos + 1) {
            // Only treat as value if it doesn't start with '-'
            if !value.starts_with('-') {
                return Some(value.clone());
            }
        }
    }

    None
}

/// Parse token type from command line arguments
/// Supports both --token-type=VALUE and --token-type VALUE formats
pub fn parse_token_type(args: &[String]) -> Result<Option<TokenType>, String> {
    // First try --token-type=VALUE format
    if let Some(token_type_str) = get_option(args, "--token-type=") {
        return token_type_str
            .parse::<TokenType>()
            .map(Some)
            .map_err(|e| e.to_string());
    }

    // Then try --token-type VALUE format (space-separated)
    if let Some(pos) = args.iter().position(|arg| arg == "--token-type") {
        if let Some(value) = args.get(pos + 1) {
            return value
                .parse::<TokenType>()
                .map(Some)
                .map_err(|e| e.to_string());
        } else {
            return Err("--token-type requires a value (bot or user)".to_string());
        }
    }

    Ok(None)
}

pub async fn run_search(args: &[String]) -> Result<(), String> {
    let query = args[2].clone();
    let count = get_option(args, "--count=").and_then(|s| s.parse().ok());
    let page = get_option(args, "--page=").and_then(|s| s.parse().ok());
    let sort = get_option(args, "--sort=");
    let sort_dir = get_option(args, "--sort_dir=");
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let response = commands::search(&client, query, count, page, sort, sort_dir)
        .await
        .map_err(|e| e.to_string())?;

    // Display error guidance if response contains a known error
    crate::api::display_wrapper_error_guidance(&response);

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "search.messages",
            "search",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

/// Get all options with a specific prefix from args
/// Supports both --key=value and --key value formats (can be mixed)
/// When using space-separated format, value must not start with '-'
pub fn get_all_options(args: &[String], prefix: &str) -> Vec<String> {
    let mut results = Vec::new();

    // Collect --key=value format
    results.extend(
        args.iter()
            .filter(|arg| arg.starts_with(prefix))
            .filter_map(|arg| arg.strip_prefix(prefix))
            .map(|s| s.to_string()),
    );

    // Collect --key value format (space-separated)
    let flag = prefix.strip_suffix('=').unwrap_or(prefix);
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag {
            if let Some(value) = args.get(i + 1) {
                // Only treat as value if it doesn't start with '-'
                if !value.starts_with('-') {
                    results.push(value.clone());
                    i += 2; // Skip both flag and value
                    continue;
                }
            }
        }
        i += 1;
    }

    results
}

pub async fn run_conv_list(args: &[String]) -> Result<(), String> {
    // Check for --help flag before API call
    if has_flag(args, "--help") || has_flag(args, "-h") {
        print_conv_usage(&args[0]);
        return Ok(());
    }

    let types = get_option(args, "--types=");
    let limit = get_option(args, "--limit=").and_then(|s| s.parse().ok());
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let filter_strings = get_all_options(args, "--filter=");
    let raw = has_flag(args, "--raw");

    // Parse format option (default: json)
    let format = if let Some(fmt_str) = get_option(args, "--format=") {
        commands::OutputFormat::parse(&fmt_str)?
    } else {
        commands::OutputFormat::Json
    };

    // Validate --raw compatibility
    if raw && format != commands::OutputFormat::Json {
        return Err(format!(
            "--raw is only valid with --format json, but got --format {}",
            format
        ));
    }

    // Parse sort options
    let sort_key = if let Some(sort_str) = get_option(args, "--sort=") {
        Some(commands::SortKey::parse(&sort_str)?)
    } else {
        None
    };

    let sort_dir = if let Some(dir_str) = get_option(args, "--sort-dir=") {
        commands::SortDirection::parse(&dir_str)?
    } else {
        commands::SortDirection::default()
    };

    // Parse filters
    let filters: Result<Vec<_>, _> = filter_strings
        .iter()
        .map(|s| commands::ConversationFilter::parse(s))
        .collect();
    let filters = filters.map_err(|e| e.to_string())?;

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let mut response = commands::conv_list(&client, types, limit)
        .await
        .map_err(|e| e.to_string())?;

    // Display error guidance if response contains a known error
    crate::api::display_wrapper_error_guidance(&response);

    // Apply filters
    commands::apply_filters(&mut response, &filters);

    // Apply sorting if specified
    if let Some(key) = sort_key {
        commands::sort_conversations(&mut response, key, sort_dir);
    }

    // Format output: non-JSON formats bypass raw/envelope logic
    let output = if format != commands::OutputFormat::Json {
        commands::format_response(&response, format)?
    } else if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "conversations.list",
            "conv list",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_conv_select(args: &[String]) -> Result<(), String> {
    // Check for --help flag before API call
    if has_flag(args, "--help") || has_flag(args, "-h") {
        print_conv_usage(&args[0]);
        return Ok(());
    }

    let types = get_option(args, "--types=");
    let limit = get_option(args, "--limit=").and_then(|s| s.parse().ok());
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let filter_strings = get_all_options(args, "--filter=");

    // Parse filters
    let filters: Result<Vec<_>, _> = filter_strings
        .iter()
        .map(|s| commands::ConversationFilter::parse(s))
        .collect();
    let filters = filters.map_err(|e| e.to_string())?;

    let client = get_api_client_with_token_type(Some(profile_name), token_type).await?;
    let mut response = commands::conv_list(&client, types, limit)
        .await
        .map_err(|e| e.to_string())?;

    // Apply filters
    commands::apply_filters(&mut response, &filters);

    // Extract conversations and present selection
    let items = commands::extract_conversations(&response);
    let selector = commands::StdinSelector;
    let channel_id = selector.select(&items)?;

    println!("{}", channel_id);
    Ok(())
}

pub async fn run_conv_search(args: &[String]) -> Result<(), String> {
    // Check for --help flag before pattern extraction
    if has_flag(args, "--help") || has_flag(args, "-h") {
        print_conv_usage(&args[0]);
        return Ok(());
    }

    // Extract the search pattern (first non-flag argument after "search")
    let pattern = args
        .get(3)
        .filter(|arg| !arg.starts_with("--"))
        .ok_or_else(|| "Search pattern is required".to_string())?
        .clone();

    let types = get_option(args, "--types=");
    let limit = get_option(args, "--limit=").and_then(|s| s.parse().ok());
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");
    let select = has_flag(args, "--select");

    // Parse additional filters from --filter= flags
    let filter_strings = get_all_options(args, "--filter=");

    // Parse format option (default: json)
    let format = if let Some(fmt_str) = get_option(args, "--format=") {
        commands::OutputFormat::parse(&fmt_str)?
    } else {
        commands::OutputFormat::Json
    };

    // Validate --raw compatibility
    if raw && format != commands::OutputFormat::Json {
        return Err(format!(
            "--raw is only valid with --format json, but got --format {}",
            format
        ));
    }

    // Parse sort options
    let sort_key = if let Some(sort_str) = get_option(args, "--sort=") {
        Some(commands::SortKey::parse(&sort_str)?)
    } else {
        None
    };

    let sort_dir = if let Some(dir_str) = get_option(args, "--sort-dir=") {
        commands::SortDirection::parse(&dir_str)?
    } else {
        commands::SortDirection::default()
    };

    // Build filters: inject name:<pattern> filter + any additional filters
    let mut filters: Vec<commands::ConversationFilter> =
        vec![commands::ConversationFilter::Name(pattern)];

    // Parse and add additional filters
    for filter_str in filter_strings {
        filters.push(commands::ConversationFilter::parse(&filter_str).map_err(|e| e.to_string())?);
    }

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let mut response = commands::conv_list(&client, types, limit)
        .await
        .map_err(|e| e.to_string())?;

    // Apply filters
    commands::apply_filters(&mut response, &filters);

    // Apply sorting if specified
    if let Some(key) = sort_key {
        commands::sort_conversations(&mut response, key, sort_dir);
    }

    // If --select flag is present, use interactive selection
    if select {
        let items = commands::extract_conversations(&response);
        let selector = commands::StdinSelector;
        let channel_id = selector.select(&items)?;
        println!("{}", channel_id);
        return Ok(());
    }

    // Format output: non-JSON formats bypass raw/envelope logic
    let output = if format != commands::OutputFormat::Json {
        commands::format_response(&response, format)?
    } else if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "conversations.list",
            "conv search",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_conv_history(args: &[String]) -> Result<(), String> {
    // Check for --help flag before API call
    if has_flag(args, "--help") || has_flag(args, "-h") {
        print_conv_usage(&args[0]);
        return Ok(());
    }

    let interactive = has_flag(args, "--interactive");

    let channel = if interactive {
        // Use conv_select logic to get channel
        let types = get_option(args, "--types=");
        let profile_name_inner = resolve_profile_name(args);
        let filter_strings = get_all_options(args, "--filter=");

        // Parse filters
        let filters: Result<Vec<_>, _> = filter_strings
            .iter()
            .map(|s| commands::ConversationFilter::parse(s))
            .collect();
        let filters = filters.map_err(|e| e.to_string())?;

        let token_type_inner = parse_token_type(args)?;
        let client =
            get_api_client_with_token_type(Some(profile_name_inner), token_type_inner).await?;
        let mut response = commands::conv_list(&client, types, None)
            .await
            .map_err(|e| e.to_string())?;

        // Apply filters
        commands::apply_filters(&mut response, &filters);

        // Extract conversations and present selection
        let items = commands::extract_conversations(&response);
        let selector = commands::StdinSelector;
        selector.select(&items)?
    } else {
        if args.len() < 4 {
            return Err("Channel argument required when --interactive is not used".to_string());
        }
        args[3].clone()
    };

    let limit = get_option(args, "--limit=").and_then(|s| s.parse().ok());
    let oldest = get_option(args, "--oldest=");
    let latest = get_option(args, "--latest=");
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let response = commands::conv_history(&client, channel, limit, oldest, latest)
        .await
        .map_err(|e| e.to_string())?;

    // Display error guidance if response contains a known error
    crate::api::display_wrapper_error_guidance(&response);

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "conversations.history",
            "conv history",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_users_info(args: &[String]) -> Result<(), String> {
    let user = args[3].clone();
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let response = commands::users_info(&client, user)
        .await
        .map_err(|e| e.to_string())?;

    // Display error guidance if response contains a known error
    crate::api::display_wrapper_error_guidance(&response);

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "users.info",
            "users info",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_users_cache_update(args: &[String]) -> Result<(), String> {
    let profile_name = resolve_profile_name(args);
    let force = has_flag(args, "--force");
    let token_type = parse_token_type(args)?;

    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let config = load_config(&config_path).map_err(|e| e.to_string())?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;

    commands::update_cache(&client, profile.team_id.clone(), force)
        .await
        .map_err(|e| e.to_string())?;

    println!("Cache updated successfully for team {}", profile.team_id);
    Ok(())
}

pub async fn run_users_resolve_mentions(args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        return Err(
            "Usage: users resolve-mentions <text> [--profile=NAME] [--format=FORMAT]".to_string(),
        );
    }

    let text = args[3].clone();
    let profile_name = resolve_profile_name(args);
    let format_str = get_option(args, "--format=").unwrap_or_else(|| "display_name".to_string());

    let format = format_str.parse::<commands::MentionFormat>().map_err(|_| {
        format!(
            "Invalid format: {}. Use display_name, real_name, or username",
            format_str
        )
    })?;

    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let config = load_config(&config_path).map_err(|e| e.to_string())?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    let cache_path = commands::UsersCacheFile::default_path()?;
    let cache_file = commands::UsersCacheFile::load(&cache_path)?;

    let workspace_cache = cache_file.get_workspace(&profile.team_id).ok_or_else(|| {
        format!(
            "No cache found for team {}. Run 'users cache-update' first.",
            profile.team_id
        )
    })?;

    let result = commands::resolve_mentions(&text, workspace_cache, format);
    println!("{}", result);
    Ok(())
}

pub async fn run_msg_post(args: &[String]) -> Result<(), String> {
    if args.len() < 5 {
        return Err("Usage: msg post <channel> <text> [--thread-ts=TS] [--reply-broadcast] [--profile=NAME] [--token-type=bot|user]".to_string());
    }

    let channel = args[3].clone();
    let text = args[4].clone();
    let thread_ts = get_option(args, "--thread-ts=");
    let reply_broadcast = has_flag(args, "--reply-broadcast");
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;

    // Validate: --reply-broadcast requires --thread-ts
    if reply_broadcast && thread_ts.is_none() {
        return Err("Error: --reply-broadcast requires --thread-ts".to_string());
    }

    let raw = has_flag(args, "--raw");
    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let response = commands::msg_post(&client, channel, text, thread_ts, reply_broadcast)
        .await
        .map_err(|e| e.to_string())?;

    // Display error guidance if response contains a known error
    crate::api::display_wrapper_error_guidance(&response);

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "chat.postMessage",
            "msg post",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_msg_update(args: &[String], non_interactive: bool) -> Result<(), String> {
    if args.len() < 6 {
        return Err("Usage: msg update <channel> <ts> <text> [--yes] [--profile=NAME] [--token-type=bot|user]".to_string());
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let text = args[5].clone();
    let yes = has_flag(args, "--yes");
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let response = commands::msg_update(&client, channel, ts, text, yes, non_interactive)
        .await
        .map_err(|e| e.to_string())?;

    // Display error guidance if response contains a known error
    crate::api::display_wrapper_error_guidance(&response);

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "chat.update",
            "msg update",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_msg_delete(args: &[String], non_interactive: bool) -> Result<(), String> {
    if args.len() < 5 {
        return Err(
            "Usage: msg delete <channel> <ts> [--yes] [--profile=NAME] [--token-type=bot|user]"
                .to_string(),
        );
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let yes = has_flag(args, "--yes");
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let response = commands::msg_delete(&client, channel, ts, yes, non_interactive)
        .await
        .map_err(|e| e.to_string())?;

    // Display error guidance if response contains a known error
    crate::api::display_wrapper_error_guidance(&response);

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "chat.delete",
            "msg delete",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_react_add(args: &[String]) -> Result<(), String> {
    if args.len() < 6 {
        return Err(
            "Usage: react add <channel> <ts> <emoji> [--profile=NAME] [--token-type=bot|user]"
                .to_string(),
        );
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let emoji = args[5].clone();
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let response = commands::react_add(&client, channel, ts, emoji)
        .await
        .map_err(|e| e.to_string())?;

    // Display error guidance if response contains a known error
    crate::api::display_wrapper_error_guidance(&response);

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "reactions.add",
            "react add",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_react_remove(args: &[String], non_interactive: bool) -> Result<(), String> {
    if args.len() < 6 {
        return Err(
            "Usage: react remove <channel> <ts> <emoji> [--yes] [--profile=NAME] [--token-type=bot|user]".to_string(),
        );
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let emoji = args[5].clone();
    let yes = has_flag(args, "--yes");
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let response = commands::react_remove(&client, channel, ts, emoji, yes, non_interactive)
        .await
        .map_err(|e| e.to_string())?;

    // Display error guidance if response contains a known error
    crate::api::display_wrapper_error_guidance(&response);

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "reactions.remove",
            "react remove",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_file_upload(args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        return Err(
            "Usage: file upload <path> [--channel=ID] [--channels=IDs] [--title=TITLE] [--comment=TEXT] [--profile=NAME] [--token-type=bot|user]"
                .to_string(),
        );
    }

    let file_path = args[3].clone();

    // Support both --channel and --channels
    let channels = get_option(args, "--channel=").or_else(|| get_option(args, "--channels="));
    let title = get_option(args, "--title=");
    let comment = get_option(args, "--comment=");
    let profile_name = resolve_profile_name(args);
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(Some(profile_name.clone()), token_type).await?;
    let response = commands::file_upload(&client, file_path, channels, title, comment)
        .await
        .map_err(|e| e.to_string())?;

    // Display error guidance if response contains a known error
    crate::api::display_json_error_guidance(&response);

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope_and_token_type(
            response_value,
            "files.upload",
            "file upload",
            Some(profile_name),
            token_type,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub fn print_conv_usage(prog: &str) {
    println!("Conv command usage:");
    println!(
        "  {} conv list [--types=TYPE] [--limit=N] [--filter=KEY:VALUE]... [--format=FORMAT] [--sort=KEY] [--sort-dir=DIR] [--raw] [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!("    List conversations with optional filtering and sorting");
    println!("    Options accept both --option=value and --option value formats");
    println!("    Filters: name:<glob>, is_member:true|false, is_private:true|false");
    println!("      - name:<glob>: Filter by channel name (supports * and ? wildcards)");
    println!("      - is_member:true|false: Filter by membership status");
    println!("      - is_private:true|false: Filter by channel privacy");
    println!("    Formats: json (default), jsonl, table, tsv");
    println!("      - json: JSON format with envelope (use --raw for raw Slack API response)");
    println!("      - jsonl: JSON Lines format (one object per line)");
    println!("      - table: Human-readable table format");
    println!("      - tsv: Tab-separated values");
    println!("    Sort keys: name, created, num_members");
    println!("      - name: Sort by channel name");
    println!("      - created: Sort by creation timestamp");
    println!("      - num_members: Sort by member count");
    println!("    Sort direction: asc (default), desc");
    println!("    Note: --raw is only valid with --format json");
    println!();
    println!(
        "  {} conv search <pattern> [--select] [--types=TYPE] [--limit=N] [--filter=KEY:VALUE]... [--format=FORMAT] [--sort=KEY] [--sort-dir=DIR] [--raw] [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!("    Search conversations by name pattern (applies name:<pattern> filter)");
    println!("    Options accept both --option=value and --option value formats");
    println!("    --select: Interactively select from results and output channel ID only");
    println!();
    println!(
        "  {} conv select [--types=TYPE] [--filter=KEY:VALUE]... [--profile=NAME]",
        prog
    );
    println!("    Interactively select a conversation and output its channel ID");
    println!("    Options accept both --option=value and --option value formats");
    println!();
    println!(
        "  {} conv history <channel> [--limit=N] [--oldest=TS] [--latest=TS] [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!(
        "  {} conv history --interactive [--types=TYPE] [--filter=KEY:VALUE]... [--limit=N] [--profile=NAME]",
        prog
    );
    println!("    Select channel interactively before fetching history");
    println!("    Options accept both --option=value and --option value formats");
}

pub fn print_users_usage(prog: &str) {
    println!("Users command usage:");
    println!(
        "  {} users info <user_id> [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!(
        "  {} users cache-update [--profile=NAME] [--force] [--token-type=bot|user]",
        prog
    );
    println!("  {} users resolve-mentions <text> [--profile=NAME] [--format=display_name|real_name|username]", prog);
    println!("  Options accept both --option=value and --option value formats");
}

pub fn print_msg_usage(prog: &str) {
    println!("Msg command usage:");
    println!(
        "  {} msg post <channel> <text> [--thread-ts=TS] [--reply-broadcast] [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!("    Requires SLACKCLI_ALLOW_WRITE=true environment variable");
    println!(
        "  {} msg update <channel> <ts> <text> [--yes] [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!("    Requires SLACKCLI_ALLOW_WRITE=true environment variable");
    println!(
        "  {} msg delete <channel> <ts> [--yes] [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!("    Requires SLACKCLI_ALLOW_WRITE=true environment variable");
    println!("  Options accept both --option=value and --option value formats");
}

pub fn print_react_usage(prog: &str) {
    println!("React command usage:");
    println!(
        "  {} react add <channel> <ts> <emoji> [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!("    Requires SLACKCLI_ALLOW_WRITE=true environment variable");
    println!(
        "  {} react remove <channel> <ts> <emoji> [--yes] [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!("    Requires SLACKCLI_ALLOW_WRITE=true environment variable");
    println!("  Options accept both --option=value and --option value formats");
}

pub fn print_file_usage(prog: &str) {
    println!("File command usage:");
    println!(
        "  {} file upload <path> [--channel=ID] [--channels=IDs] [--title=TITLE] [--comment=TEXT] [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!("  Options accept both --option=value and --option value formats");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_token_type_equals_format() {
        let args = vec!["command".to_string(), "--token-type=user".to_string()];
        let result = parse_token_type(&args).unwrap();
        assert_eq!(result, Some(TokenType::User));
    }

    #[test]
    fn test_parse_token_type_space_separated() {
        let args = vec![
            "command".to_string(),
            "--token-type".to_string(),
            "bot".to_string(),
        ];
        let result = parse_token_type(&args).unwrap();
        assert_eq!(result, Some(TokenType::Bot));
    }

    #[test]
    fn test_parse_token_type_both_values() {
        // Test user with equals
        let args1 = vec!["--token-type=user".to_string()];
        assert_eq!(parse_token_type(&args1).unwrap(), Some(TokenType::User));

        // Test bot with equals
        let args2 = vec!["--token-type=bot".to_string()];
        assert_eq!(parse_token_type(&args2).unwrap(), Some(TokenType::Bot));

        // Test user with space
        let args3 = vec!["--token-type".to_string(), "user".to_string()];
        assert_eq!(parse_token_type(&args3).unwrap(), Some(TokenType::User));

        // Test bot with space
        let args4 = vec!["--token-type".to_string(), "bot".to_string()];
        assert_eq!(parse_token_type(&args4).unwrap(), Some(TokenType::Bot));
    }

    #[test]
    fn test_parse_token_type_missing() {
        let args = vec!["command".to_string()];
        let result = parse_token_type(&args).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_token_type_missing_value() {
        let args = vec!["--token-type".to_string()];
        let result = parse_token_type(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "--token-type requires a value (bot or user)"
        );
    }

    #[test]
    fn test_parse_token_type_invalid_value() {
        let args = vec!["--token-type=invalid".to_string()];
        let result = parse_token_type(&args);
        assert!(result.is_err());
    }

    // Mock token store for testing
    struct MockTokenStore {
        tokens: std::collections::HashMap<String, String>,
    }

    impl MockTokenStore {
        fn new() -> Self {
            Self {
                tokens: std::collections::HashMap::new(),
            }
        }

        fn with_token(mut self, key: &str, value: &str) -> Self {
            self.tokens.insert(key.to_string(), value.to_string());
            self
        }
    }

    impl TokenStore for MockTokenStore {
        fn get(&self, key: &str) -> crate::profile::token_store::Result<String> {
            use crate::profile::token_store::TokenStoreError;
            self.tokens
                .get(key)
                .cloned()
                .ok_or_else(|| TokenStoreError::NotFound(key.to_string()))
        }

        fn set(&self, _key: &str, _value: &str) -> crate::profile::token_store::Result<()> {
            unimplemented!("set not needed for tests")
        }

        fn delete(&self, _key: &str) -> crate::profile::token_store::Result<()> {
            unimplemented!("delete not needed for tests")
        }

        fn exists(&self, key: &str) -> bool {
            self.tokens.contains_key(key)
        }
    }

    #[test]
    fn test_resolve_token_prefers_env() {
        // SLACK_TOKEN should be preferred over token store
        let store = MockTokenStore::new().with_token("T123:U123", "xoxb-store-token");

        let result = resolve_token_for_wrapper(
            Some("xoxb-env-token".to_string()),
            &store,
            "T123:U123",
            None,
            false,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "xoxb-env-token");
    }

    #[test]
    fn test_resolve_token_uses_store() {
        // When SLACK_TOKEN is not set, use token store
        let store = MockTokenStore::new().with_token("T123:U123", "xoxb-store-token");

        let result = resolve_token_for_wrapper(None, &store, "T123:U123", None, false);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "xoxb-store-token");
    }

    #[test]
    fn test_resolve_token_explicit_request() {
        // When token type is explicitly requested, don't fallback
        let store = MockTokenStore::new().with_token("T123:U123", "xoxb-bot-token");

        let result = resolve_token_for_wrapper(
            None,
            &store,
            "T123:U123:user",  // User token key
            Some("T123:U123"), // Bot token fallback
            true,              // Explicit request
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("explicitly requested"));
    }

    #[test]
    fn test_resolve_token_fallback_when_not_explicit() {
        // When token type is not explicitly requested, allow fallback
        let store = MockTokenStore::new().with_token("T123:U123", "xoxb-bot-token");

        let result = resolve_token_for_wrapper(
            None,
            &store,
            "T123:U123:user",  // User token key (not found)
            Some("T123:U123"), // Bot token fallback
            false,             // Not explicit request
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "xoxb-bot-token");
    }

    #[test]
    fn test_resolve_token_env_overrides_explicit() {
        // SLACK_TOKEN should override even explicit token type requests
        let store = MockTokenStore::new()
            .with_token("T123:U123", "xoxb-bot-token")
            .with_token("T123:U123:user", "xoxp-user-token");

        let result = resolve_token_for_wrapper(
            Some("xoxb-env-token".to_string()),
            &store,
            "T123:U123:user",
            None,
            true, // Explicit request
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "xoxb-env-token");
    }

    // Tests for get_option with space-separated format
    #[test]
    fn test_get_option_equals_format() {
        let args = vec!["cmd".to_string(), "--filter=is_private:true".to_string()];
        assert_eq!(
            get_option(&args, "--filter="),
            Some("is_private:true".to_string())
        );
    }

    #[test]
    fn test_get_option_space_separated() {
        let args = vec![
            "cmd".to_string(),
            "--filter".to_string(),
            "is_private:true".to_string(),
        ];
        assert_eq!(
            get_option(&args, "--filter="),
            Some("is_private:true".to_string())
        );
    }

    #[test]
    fn test_get_option_space_separated_rejects_dash_value() {
        // Value starting with '-' should not be treated as value
        let args = vec![
            "cmd".to_string(),
            "--filter".to_string(),
            "--other".to_string(),
        ];
        assert_eq!(get_option(&args, "--filter="), None);
    }

    #[test]
    fn test_get_option_space_separated_missing_value() {
        let args = vec!["cmd".to_string(), "--filter".to_string()];
        assert_eq!(get_option(&args, "--filter="), None);
    }

    #[test]
    fn test_get_option_prefers_equals_format() {
        // When both formats exist, equals format should be returned first
        let args = vec![
            "--filter=value1".to_string(),
            "--filter".to_string(),
            "value2".to_string(),
        ];
        assert_eq!(get_option(&args, "--filter="), Some("value1".to_string()));
    }

    // Tests for get_all_options with mixed formats
    #[test]
    fn test_get_all_options_equals_format() {
        let args = vec![
            "cmd".to_string(),
            "--filter=is_private:true".to_string(),
            "--filter=is_member:true".to_string(),
        ];
        let result = get_all_options(&args, "--filter=");
        assert_eq!(result, vec!["is_private:true", "is_member:true"]);
    }

    #[test]
    fn test_get_all_options_space_separated() {
        let args = vec![
            "cmd".to_string(),
            "--filter".to_string(),
            "is_private:true".to_string(),
            "--filter".to_string(),
            "is_member:true".to_string(),
        ];
        let result = get_all_options(&args, "--filter=");
        assert_eq!(result, vec!["is_private:true", "is_member:true"]);
    }

    #[test]
    fn test_get_all_options_mixed_format() {
        let args = vec![
            "cmd".to_string(),
            "--filter=is_private:true".to_string(),
            "--filter".to_string(),
            "is_member:true".to_string(),
            "--filter=name:test".to_string(),
            "--filter".to_string(),
            "is_archived:false".to_string(),
        ];
        let result = get_all_options(&args, "--filter=");
        assert_eq!(
            result,
            vec![
                "is_private:true",
                "name:test",
                "is_member:true",
                "is_archived:false"
            ]
        );
    }

    #[test]
    fn test_get_all_options_rejects_dash_values() {
        let args = vec![
            "cmd".to_string(),
            "--filter=value1".to_string(),
            "--filter".to_string(),
            "--other".to_string(), // Should be ignored
            "--filter".to_string(),
            "value2".to_string(),
        ];
        let result = get_all_options(&args, "--filter=");
        assert_eq!(result, vec!["value1", "value2"]);
    }

    #[test]
    fn test_get_all_options_space_separated_at_end() {
        // --filter at the end without value should be ignored
        let args = vec![
            "cmd".to_string(),
            "--filter=value1".to_string(),
            "--filter".to_string(),
        ];
        let result = get_all_options(&args, "--filter=");
        assert_eq!(result, vec!["value1"]);
    }

    // Integration tests for conv commands with space-separated options
    #[test]
    fn test_conv_list_filter_space_separated() {
        // Test that filter parsing works with space-separated format
        let args = vec![
            "slack".to_string(),
            "conv".to_string(),
            "list".to_string(),
            "--filter".to_string(),
            "is_private:true".to_string(),
        ];
        let filters = get_all_options(&args, "--filter=");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0], "is_private:true");
    }

    #[test]
    fn test_conv_list_multiple_filters_mixed() {
        let args = vec![
            "slack".to_string(),
            "conv".to_string(),
            "list".to_string(),
            "--filter=is_private:true".to_string(),
            "--filter".to_string(),
            "is_member:true".to_string(),
        ];
        let filters = get_all_options(&args, "--filter=");
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0], "is_private:true");
        assert_eq!(filters[1], "is_member:true");
    }

    #[test]
    fn test_conv_search_options_space_separated() {
        let args = vec![
            "slack".to_string(),
            "conv".to_string(),
            "search".to_string(),
            "pattern".to_string(),
            "--format".to_string(),
            "table".to_string(),
            "--sort".to_string(),
            "name".to_string(),
        ];
        assert_eq!(get_option(&args, "--format="), Some("table".to_string()));
        assert_eq!(get_option(&args, "--sort="), Some("name".to_string()));
    }

    #[test]
    fn test_search_command_options_space_separated() {
        let args = vec![
            "slack".to_string(),
            "search".to_string(),
            "query".to_string(),
            "--count".to_string(),
            "10".to_string(),
            "--sort".to_string(),
            "timestamp".to_string(),
        ];
        assert_eq!(get_option(&args, "--count="), Some("10".to_string()));
        assert_eq!(get_option(&args, "--sort="), Some("timestamp".to_string()));
    }

    // Tests for resolve_profile_name function
    #[test]
    fn test_resolve_profile_name_with_equals_format() {
        let args = vec![
            "slack".to_string(),
            "api".to_string(),
            "call".to_string(),
            "--profile=myprofile".to_string(),
            "test.method".to_string(),
        ];
        assert_eq!(resolve_profile_name(&args), "myprofile");
    }

    #[test]
    fn test_resolve_profile_name_with_space_format() {
        let args = vec![
            "slack".to_string(),
            "api".to_string(),
            "call".to_string(),
            "--profile".to_string(),
            "myprofile".to_string(),
            "test.method".to_string(),
        ];
        assert_eq!(resolve_profile_name(&args), "myprofile");
    }

    #[test]
    fn test_resolve_profile_name_at_beginning() {
        let args = vec![
            "slack".to_string(),
            "--profile=myprofile".to_string(),
            "api".to_string(),
            "call".to_string(),
            "test.method".to_string(),
        ];
        assert_eq!(resolve_profile_name(&args), "myprofile");
    }

    #[test]
    fn test_resolve_profile_name_at_end() {
        let args = vec![
            "slack".to_string(),
            "api".to_string(),
            "call".to_string(),
            "test.method".to_string(),
            "--profile=myprofile".to_string(),
        ];
        assert_eq!(resolve_profile_name(&args), "myprofile");
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_profile_name_env_fallback() {
        // Set environment variable
        std::env::set_var("SLACK_PROFILE", "envprofile");

        let args = vec!["slack".to_string(), "api".to_string(), "call".to_string()];
        assert_eq!(resolve_profile_name(&args), "envprofile");

        // Clean up
        std::env::remove_var("SLACK_PROFILE");
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_profile_name_default_fallback() {
        // Ensure SLACK_PROFILE is not set
        std::env::remove_var("SLACK_PROFILE");

        let args = vec!["slack".to_string(), "api".to_string(), "call".to_string()];
        assert_eq!(resolve_profile_name(&args), "default");
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_profile_name_flag_overrides_env() {
        // Set environment variable
        std::env::set_var("SLACK_PROFILE", "envprofile");

        let args = vec![
            "slack".to_string(),
            "api".to_string(),
            "--profile=flagprofile".to_string(),
            "call".to_string(),
        ];
        assert_eq!(resolve_profile_name(&args), "flagprofile");

        // Clean up
        std::env::remove_var("SLACK_PROFILE");
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_profile_name_priority_all_sources() {
        // Set environment variable
        std::env::set_var("SLACK_PROFILE", "envprofile");

        // Test that --profile flag takes highest priority
        let args = vec![
            "--profile".to_string(),
            "flagprofile".to_string(),
            "slack".to_string(),
            "api".to_string(),
            "call".to_string(),
        ];
        assert_eq!(resolve_profile_name(&args), "flagprofile");

        // Clean up
        std::env::remove_var("SLACK_PROFILE");
    }

    #[test]
    fn test_resolve_profile_name_mixed_formats() {
        // Test that equals format is found even with space format present
        let args = vec![
            "slack".to_string(),
            "--profile=profile1".to_string(),
            "api".to_string(),
            "--profile".to_string(),
            "profile2".to_string(),
            "call".to_string(),
        ];
        // Should return profile1 as equals format is checked first
        assert_eq!(resolve_profile_name(&args), "profile1");
    }
}
