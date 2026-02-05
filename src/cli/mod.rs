//! CLI command routing and handlers

mod context;
mod handlers;

pub use context::CliContext;
pub use handlers::{handle_export_command, handle_import_command, run_api_call, run_auth_login};

use crate::api::{ApiClient, CommandResponse};
use crate::commands;
use crate::commands::ConversationSelector;
use crate::profile::{
    create_token_store, default_config_path, load_config, make_token_key, resolve_profile_full,
    TokenType,
};
use serde_json::Value;

/// Get API client for a profile with optional token type selection
///
/// # Arguments
/// * `profile_name` - Optional profile name (defaults to "default")
/// * `token_type` - Optional token type (bot/user). If None, uses profile default or bot fallback
///
/// # Token Resolution Priority
/// 1. CLI flag token_type parameter (if provided)
/// 2. Profile's default_token_type (if set)
/// 3. Try user token first, fall back to bot token
pub async fn get_api_client_with_token_type(
    profile_name: Option<String>,
    token_type: Option<TokenType>,
) -> Result<ApiClient, String> {
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
pub async fn wrap_with_envelope(
    response: Value,
    method: &str,
    command: &str,
    profile_name: Option<String>,
) -> Result<CommandResponse, String> {
    let profile_name_str = profile_name.unwrap_or_else(|| "default".to_string());
    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let profile = resolve_profile_full(&config_path, &profile_name_str)
        .map_err(|e| format!("Failed to resolve profile '{}': {}", profile_name_str, e))?;

    Ok(CommandResponse::new(
        response,
        Some(profile_name_str),
        profile.team_id,
        profile.user_id,
        method.to_string(),
        command.to_string(),
    ))
}

/// Get option value from args (e.g., --key=value)
pub fn get_option(args: &[String], prefix: &str) -> Option<String> {
    args.iter()
        .find(|arg| arg.starts_with(prefix))
        .and_then(|arg| arg.strip_prefix(prefix))
        .map(|s| s.to_string())
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
    let profile = get_option(args, "--profile=");
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(profile.clone(), token_type).await?;
    let response = commands::search(&client, query, count, page, sort, sort_dir)
        .await
        .map_err(|e| e.to_string())?;

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped =
            wrap_with_envelope(response_value, "search.messages", "search", profile).await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

/// Get all options with a specific prefix from args
pub fn get_all_options(args: &[String], prefix: &str) -> Vec<String> {
    args.iter()
        .filter(|arg| arg.starts_with(prefix))
        .filter_map(|arg| arg.strip_prefix(prefix))
        .map(|s| s.to_string())
        .collect()
}

pub async fn run_conv_list(args: &[String]) -> Result<(), String> {
    let types = get_option(args, "--types=");
    let limit = get_option(args, "--limit=").and_then(|s| s.parse().ok());
    let profile = get_option(args, "--profile=");
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

    let client = get_api_client_with_token_type(profile.clone(), token_type).await?;
    let mut response = commands::conv_list(&client, types, limit)
        .await
        .map_err(|e| e.to_string())?;

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
        let wrapped =
            wrap_with_envelope(response_value, "conversations.list", "conv list", profile).await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_conv_select(args: &[String]) -> Result<(), String> {
    let types = get_option(args, "--types=");
    let limit = get_option(args, "--limit=").and_then(|s| s.parse().ok());
    let profile = get_option(args, "--profile=");
    let token_type = parse_token_type(args)?;
    let filter_strings = get_all_options(args, "--filter=");

    // Parse filters
    let filters: Result<Vec<_>, _> = filter_strings
        .iter()
        .map(|s| commands::ConversationFilter::parse(s))
        .collect();
    let filters = filters.map_err(|e| e.to_string())?;

    let client = get_api_client_with_token_type(profile, token_type).await?;
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

pub async fn run_conv_history(args: &[String]) -> Result<(), String> {
    let interactive = has_flag(args, "--interactive");

    let channel = if interactive {
        // Use conv_select logic to get channel
        let types = get_option(args, "--types=");
        let profile = get_option(args, "--profile=");
        let filter_strings = get_all_options(args, "--filter=");

        // Parse filters
        let filters: Result<Vec<_>, _> = filter_strings
            .iter()
            .map(|s| commands::ConversationFilter::parse(s))
            .collect();
        let filters = filters.map_err(|e| e.to_string())?;

        let token_type_inner = parse_token_type(args)?;
        let client = get_api_client_with_token_type(profile.clone(), token_type_inner).await?;
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
    let profile = get_option(args, "--profile=");
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(profile.clone(), token_type).await?;
    let response = commands::conv_history(&client, channel, limit, oldest, latest)
        .await
        .map_err(|e| e.to_string())?;

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped = wrap_with_envelope(
            response_value,
            "conversations.history",
            "conv history",
            profile,
        )
        .await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_users_info(args: &[String]) -> Result<(), String> {
    let user = args[3].clone();
    let profile = get_option(args, "--profile=");
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(profile.clone(), token_type).await?;
    let response = commands::users_info(&client, user)
        .await
        .map_err(|e| e.to_string())?;

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped =
            wrap_with_envelope(response_value, "users.info", "users info", profile).await?;
        serde_json::to_string_pretty(&wrapped).unwrap()
    };

    println!("{}", output);
    Ok(())
}

pub async fn run_users_cache_update(args: &[String]) -> Result<(), String> {
    let profile_name = get_option(args, "--profile=").unwrap_or_else(|| "default".to_string());
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
    let profile_name = get_option(args, "--profile=").unwrap_or_else(|| "default".to_string());
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
    let profile = get_option(args, "--profile=");
    let token_type = parse_token_type(args)?;

    // Validate: --reply-broadcast requires --thread-ts
    if reply_broadcast && thread_ts.is_none() {
        return Err("Error: --reply-broadcast requires --thread-ts".to_string());
    }

    let raw = has_flag(args, "--raw");
    let client = get_api_client_with_token_type(profile.clone(), token_type).await?;
    let response = commands::msg_post(&client, channel, text, thread_ts, reply_broadcast)
        .await
        .map_err(|e| e.to_string())?;

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped =
            wrap_with_envelope(response_value, "chat.postMessage", "msg post", profile).await?;
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
    let profile = get_option(args, "--profile=");
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(profile.clone(), token_type).await?;
    let response = commands::msg_update(&client, channel, ts, text, yes, non_interactive)
        .await
        .map_err(|e| e.to_string())?;

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped =
            wrap_with_envelope(response_value, "chat.update", "msg update", profile).await?;
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
    let profile = get_option(args, "--profile=");
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(profile.clone(), token_type).await?;
    let response = commands::msg_delete(&client, channel, ts, yes, non_interactive)
        .await
        .map_err(|e| e.to_string())?;

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped =
            wrap_with_envelope(response_value, "chat.delete", "msg delete", profile).await?;
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
    let profile = get_option(args, "--profile=");
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(profile.clone(), token_type).await?;
    let response = commands::react_add(&client, channel, ts, emoji)
        .await
        .map_err(|e| e.to_string())?;

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped =
            wrap_with_envelope(response_value, "reactions.add", "react add", profile).await?;
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
    let profile = get_option(args, "--profile=");
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(profile.clone(), token_type).await?;
    let response = commands::react_remove(&client, channel, ts, emoji, yes, non_interactive)
        .await
        .map_err(|e| e.to_string())?;

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped =
            wrap_with_envelope(response_value, "reactions.remove", "react remove", profile).await?;
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
    let profile = get_option(args, "--profile=");
    let token_type = parse_token_type(args)?;
    let raw = has_flag(args, "--raw");

    let client = get_api_client_with_token_type(profile.clone(), token_type).await?;
    let response = commands::file_upload(&client, file_path, channels, title, comment)
        .await
        .map_err(|e| e.to_string())?;

    // Output with or without envelope
    let output = if raw {
        serde_json::to_string_pretty(&response).unwrap()
    } else {
        let response_value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
        let wrapped =
            wrap_with_envelope(response_value, "files.upload", "file upload", profile).await?;
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
    println!("    Filters: name:<glob>, is_member:true|false, is_private:true|false");
    println!("    Formats: json (default), jsonl, table, tsv");
    println!("    Sort keys: name, created, num_members");
    println!("    Sort direction: asc (default), desc");
    println!("    Note: --raw is only valid with --format json");
    println!(
        "  {} conv select [--types=TYPE] [--filter=KEY:VALUE]... [--profile=NAME]",
        prog
    );
    println!("    Interactively select a conversation and output its channel ID");
    println!(
        "  {} conv history <channel> [--limit=N] [--oldest=TS] [--latest=TS] [--profile=NAME] [--token-type=bot|user]",
        prog
    );
    println!(
        "  {} conv history --interactive [--types=TYPE] [--filter=KEY:VALUE]... [--limit=N] [--profile=NAME]",
        prog
    );
    println!("    Select channel interactively before fetching history");
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
}

pub fn print_file_usage(prog: &str) {
    println!("File command usage:");
    println!(
        "  {} file upload <path> [--channel=ID] [--channels=IDs] [--title=TITLE] [--comment=TEXT] [--profile=NAME] [--token-type=bot|user]",
        prog
    );
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
}
