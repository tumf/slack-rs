//! CLI command routing and handlers

use crate::api::ApiClient;
use crate::commands;
use crate::commands::ConversationSelector;
use crate::profile::{
    default_config_path, load_config, make_token_key, FileTokenStore, TokenStore,
};

/// Get API client for a profile with optional token type preference
/// Returns (ApiClient, is_user_token) where is_user_token indicates if a user token was used
pub async fn get_api_client_with_token_type(
    profile_name: Option<String>,
    prefer_user_token: bool,
) -> Result<(ApiClient, bool), String> {
    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());
    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let config = load_config(&config_path).map_err(|e| e.to_string())?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    let token_store = FileTokenStore::new().map_err(|e| e.to_string())?;

    let user_token_key = format!("{}:{}:user", profile.team_id, profile.user_id);
    let bot_token_key = make_token_key(&profile.team_id, &profile.user_id);

    let (token, is_user_token) = if prefer_user_token {
        // User token is required - do not fall back to bot token
        let token = token_store.get(&user_token_key).map_err(|_| {
            format!(
                "User token not found for profile '{}'.\n\
                    Private channels require a User Token with appropriate scopes.\n\
                    Run: slackcli auth login (with user_scopes)",
                profile_name
            )
        })?;
        (token, true)
    } else {
        // Try bot token first, fall back to user token
        match token_store.get(&bot_token_key) {
            Ok(bot_token) => (bot_token, false),
            Err(_) => {
                let user_token = token_store
                    .get(&user_token_key)
                    .map_err(|e| format!("Failed to get token: {}", e))?;
                (user_token, true)
            }
        }
    };

    Ok((ApiClient::with_token(token), is_user_token))
}

/// Get API client for a profile (default: prefer user token)
pub async fn get_api_client(profile_name: Option<String>) -> Result<ApiClient, String> {
    get_api_client_with_token_type(profile_name, true)
        .await
        .map(|(client, _)| client)
}

/// Check if a flag exists in args
pub fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

/// Get option value from args (e.g., --key=value)
pub fn get_option(args: &[String], prefix: &str) -> Option<String> {
    args.iter()
        .find(|arg| arg.starts_with(prefix))
        .and_then(|arg| arg.strip_prefix(prefix))
        .map(|s| s.to_string())
}

pub async fn run_search(args: &[String]) -> Result<(), String> {
    let query = args[2].clone();
    let count = get_option(args, "--count=").and_then(|s| s.parse().ok());
    let page = get_option(args, "--page=").and_then(|s| s.parse().ok());
    let sort = get_option(args, "--sort=");
    let sort_dir = get_option(args, "--sort_dir=");
    let profile = get_option(args, "--profile=");

    let client = get_api_client(profile).await?;
    let response = commands::search(&client, query, count, page, sort, sort_dir)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
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

/// Check if we should show guidance for empty private channel list
fn should_show_conv_list_guidance(
    types: &Option<String>,
    response: &crate::api::types::ApiResponse,
    is_user_token: bool,
) -> bool {
    // Only show guidance if using bot token (not user token)
    if is_user_token {
        return false;
    }

    // Only show guidance if types includes private_channel
    if let Some(types_str) = types {
        if !types_str.contains("private_channel") {
            return false;
        }
    } else {
        return false;
    }

    // Check if response has empty channels array
    if let Some(channels) = response.data.get("channels") {
        if let Some(channels_array) = channels.as_array() {
            return channels_array.is_empty();
        }
    }

    false
}

pub async fn run_conv_list(args: &[String]) -> Result<(), String> {
    let types = get_option(args, "--types=");
    let limit = get_option(args, "--limit=").and_then(|s| s.parse().ok());
    let profile = get_option(args, "--profile=");
    let filter_strings = get_all_options(args, "--filter=");

    // Parse filters
    let filters: Result<Vec<_>, _> = filter_strings
        .iter()
        .map(|s| commands::ConversationFilter::parse(s))
        .collect();
    let filters = filters.map_err(|e| e.to_string())?;

    // Determine if we should prefer user token (when types includes private_channel)
    let prefer_user_token = types
        .as_ref()
        .map(|t| t.contains("private_channel"))
        .unwrap_or(false);

    let (client, is_user_token) =
        get_api_client_with_token_type(profile, prefer_user_token).await?;
    let mut response = commands::conv_list(&client, types.clone(), limit)
        .await
        .map_err(|e| e.to_string())?;

    // Check if we should show guidance for empty private_channel list
    if should_show_conv_list_guidance(&types, &response, is_user_token) {
        eprintln!();
        eprintln!("Note: The conversation list for private channels is empty.");
        eprintln!("Bot tokens can only see private channels where the bot is a member.");
        eprintln!("To list all private channels, use a User Token with appropriate scopes.");
        eprintln!("Run: slackcli auth login (with user_scopes)");
        eprintln!();
    }

    // Apply filters
    commands::apply_filters(&mut response, &filters);

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    Ok(())
}

pub async fn run_conv_select(args: &[String]) -> Result<(), String> {
    let types = get_option(args, "--types=");
    let limit = get_option(args, "--limit=").and_then(|s| s.parse().ok());
    let profile = get_option(args, "--profile=");
    let filter_strings = get_all_options(args, "--filter=");

    // Parse filters
    let filters: Result<Vec<_>, _> = filter_strings
        .iter()
        .map(|s| commands::ConversationFilter::parse(s))
        .collect();
    let filters = filters.map_err(|e| e.to_string())?;

    let client = get_api_client(profile).await?;
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

        let client = get_api_client(profile.clone()).await?;
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

    let client = get_api_client(profile).await?;
    let response = commands::conv_history(&client, channel, limit, oldest, latest)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    Ok(())
}

pub async fn run_users_info(args: &[String]) -> Result<(), String> {
    let user = args[3].clone();
    let profile = get_option(args, "--profile=");

    let client = get_api_client(profile).await?;
    let response = commands::users_info(&client, user)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    Ok(())
}

pub async fn run_users_cache_update(args: &[String]) -> Result<(), String> {
    let profile_name = get_option(args, "--profile=").unwrap_or_else(|| "default".to_string());
    let force = has_flag(args, "--force");

    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let config = load_config(&config_path).map_err(|e| e.to_string())?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    let client = get_api_client(Some(profile_name.clone())).await?;

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
        return Err("Usage: msg post <channel> <text> [--thread-ts=TS] [--reply-broadcast] [--profile=NAME]".to_string());
    }

    let channel = args[3].clone();
    let text = args[4].clone();
    let thread_ts = get_option(args, "--thread-ts=");
    let reply_broadcast = has_flag(args, "--reply-broadcast");
    let profile = get_option(args, "--profile=");

    // Validate: --reply-broadcast requires --thread-ts
    if reply_broadcast && thread_ts.is_none() {
        return Err("Error: --reply-broadcast requires --thread-ts".to_string());
    }

    let client = get_api_client(profile).await?;
    let response = commands::msg_post(&client, channel, text, thread_ts, reply_broadcast)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    Ok(())
}

pub async fn run_msg_update(args: &[String]) -> Result<(), String> {
    if args.len() < 6 {
        return Err("Usage: msg update <channel> <ts> <text> [--yes] [--profile=NAME]".to_string());
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let text = args[5].clone();
    let yes = has_flag(args, "--yes");
    let profile = get_option(args, "--profile=");

    let client = get_api_client(profile).await?;
    let response = commands::msg_update(&client, channel, ts, text, yes)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    Ok(())
}

pub async fn run_msg_delete(args: &[String]) -> Result<(), String> {
    if args.len() < 5 {
        return Err("Usage: msg delete <channel> <ts> [--yes] [--profile=NAME]".to_string());
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let yes = has_flag(args, "--yes");
    let profile = get_option(args, "--profile=");

    let client = get_api_client(profile).await?;
    let response = commands::msg_delete(&client, channel, ts, yes)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    Ok(())
}

pub async fn run_react_add(args: &[String]) -> Result<(), String> {
    if args.len() < 6 {
        return Err("Usage: react add <channel> <ts> <emoji> [--profile=NAME]".to_string());
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let emoji = args[5].clone();
    let profile = get_option(args, "--profile=");

    let client = get_api_client(profile).await?;
    let response = commands::react_add(&client, channel, ts, emoji)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    Ok(())
}

pub async fn run_react_remove(args: &[String]) -> Result<(), String> {
    if args.len() < 6 {
        return Err(
            "Usage: react remove <channel> <ts> <emoji> [--yes] [--profile=NAME]".to_string(),
        );
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let emoji = args[5].clone();
    let yes = has_flag(args, "--yes");
    let profile = get_option(args, "--profile=");

    let client = get_api_client(profile).await?;
    let response = commands::react_remove(&client, channel, ts, emoji, yes)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    Ok(())
}

pub async fn run_file_upload(args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        return Err(
            "Usage: file upload <path> [--channel=ID] [--channels=IDs] [--title=TITLE] [--comment=TEXT] [--profile=NAME]"
                .to_string(),
        );
    }

    let file_path = args[3].clone();

    // Support both --channel and --channels
    let channels = get_option(args, "--channel=").or_else(|| get_option(args, "--channels="));
    let title = get_option(args, "--title=");
    let comment = get_option(args, "--comment=");
    let profile = get_option(args, "--profile=");

    let client = get_api_client(profile).await?;
    let response = commands::file_upload(&client, file_path, channels, title, comment)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    Ok(())
}

pub fn print_conv_usage(prog: &str) {
    println!("Conv command usage:");
    println!(
        "  {} conv list [--types=TYPE] [--limit=N] [--filter=KEY:VALUE]... [--profile=NAME]",
        prog
    );
    println!("    Filters: name:<glob>, is_member:true|false, is_private:true|false");
    println!(
        "  {} conv select [--types=TYPE] [--filter=KEY:VALUE]... [--profile=NAME]",
        prog
    );
    println!("    Interactively select a conversation and output its channel ID");
    println!(
        "  {} conv history <channel> [--limit=N] [--oldest=TS] [--latest=TS] [--profile=NAME]",
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
    println!("  {} users info <user_id> [--profile=NAME]", prog);
    println!("  {} users cache-update [--profile=NAME] [--force]", prog);
    println!("  {} users resolve-mentions <text> [--profile=NAME] [--format=display_name|real_name|username]", prog);
}

pub fn print_msg_usage(prog: &str) {
    println!("Msg command usage:");
    println!(
        "  {} msg post <channel> <text> [--thread-ts=TS] [--reply-broadcast] [--profile=NAME]",
        prog
    );
    println!(
        "  {} msg update <channel> <ts> <text> [--yes] [--profile=NAME]",
        prog
    );
    println!(
        "  {} msg delete <channel> <ts> [--yes] [--profile=NAME]",
        prog
    );
}

pub fn print_react_usage(prog: &str) {
    println!("React command usage:");
    println!(
        "  {} react add <channel> <ts> <emoji> [--profile=NAME]",
        prog
    );
    println!(
        "  {} react remove <channel> <ts> <emoji> [--yes] [--profile=NAME]",
        prog
    );
}

pub fn print_file_usage(prog: &str) {
    println!("File command usage:");
    println!(
        "  {} file upload <path> [--channel=ID] [--channels=IDs] [--title=TITLE] [--comment=TEXT] [--profile=NAME]",
        prog
    );
}
