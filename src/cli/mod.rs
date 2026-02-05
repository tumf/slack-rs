//! CLI command routing and handlers

use crate::api::call::{ApiCallMeta, ApiCallResponse};
use crate::api::ApiCallContext;
use crate::api::ApiClient;
use crate::commands;
use crate::commands::ConversationSelector;
use crate::profile::{
    default_config_path, load_config, make_token_key, FileTokenStore, TokenStore,
};
use serde::Serialize;

/// Get API client for a profile
pub async fn get_api_client(profile_name: Option<String>) -> Result<ApiClient, String> {
    let (client, _) = get_api_client_with_context(profile_name).await?;
    Ok(client)
}

/// Get API client and execution context for a profile
pub async fn get_api_client_with_context(
    profile_name: Option<String>,
) -> Result<(ApiClient, ApiCallContext), String> {
    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());
    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let config = load_config(&config_path).map_err(|e| e.to_string())?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    let token_store = FileTokenStore::new().map_err(|e| e.to_string())?;

    // Try to get user token first (for APIs that require user scope like search.messages)
    let user_token_key = format!("{}:{}:user", profile.team_id, profile.user_id);

    let token = match token_store.get(&user_token_key) {
        Ok(user_token) => user_token,
        Err(_) => {
            // Fall back to bot token
            let bot_token_key = make_token_key(&profile.team_id, &profile.user_id);
            token_store
                .get(&bot_token_key)
                .map_err(|e| format!("Failed to get token: {}", e))?
        }
    };

    let context = ApiCallContext {
        profile_name: Some(profile_name),
        team_id: profile.team_id.clone(),
        user_id: profile.user_id.clone(),
    };

    Ok((ApiClient::with_token(token), context))
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

fn make_meta(context: &ApiCallContext, method: &str, command: &str) -> ApiCallMeta {
    ApiCallMeta {
        profile_name: context.profile_name.clone(),
        team_id: context.team_id.clone(),
        user_id: context.user_id.clone(),
        method: method.to_string(),
        command: command.to_string(),
    }
}

fn print_unified_output<T: Serialize>(
    response: &T,
    meta: ApiCallMeta,
    raw: bool,
) -> Result<(), String> {
    if raw {
        let json = serde_json::to_string_pretty(response).map_err(|e| e.to_string())?;
        println!("{}", json);
        return Ok(());
    }

    let value = serde_json::to_value(response).map_err(|e| e.to_string())?;
    let envelope = ApiCallResponse {
        response: value,
        meta,
    };
    let json = serde_json::to_string_pretty(&envelope).map_err(|e| e.to_string())?;
    println!("{}", json);
    Ok(())
}

pub async fn run_search(args: &[String]) -> Result<(), String> {
    let query = args[2].clone();
    let count = get_option(args, "--count=").and_then(|s| s.parse().ok());
    let page = get_option(args, "--page=").and_then(|s| s.parse().ok());
    let sort = get_option(args, "--sort=");
    let sort_dir = get_option(args, "--sort_dir=");
    let profile = get_option(args, "--profile=");
    let raw = has_flag(args, "--raw");

    let (client, context) = get_api_client_with_context(profile).await?;
    let response = commands::search(&client, query, count, page, sort, sort_dir)
        .await
        .map_err(|e| e.to_string())?;

    let meta = make_meta(&context, "search.messages", "search");
    print_unified_output(&response, meta, raw)?;
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
    let raw = has_flag(args, "--raw");
    let filter_strings = get_all_options(args, "--filter=");

    // Parse filters
    let filters: Result<Vec<_>, _> = filter_strings
        .iter()
        .map(|s| commands::ConversationFilter::parse(s))
        .collect();
    let filters = filters.map_err(|e| e.to_string())?;

    let (client, context) = get_api_client_with_context(profile).await?;
    let mut response = commands::conv_list(&client, types, limit)
        .await
        .map_err(|e| e.to_string())?;

    // Apply filters
    commands::apply_filters(&mut response, &filters);

    let meta = make_meta(&context, "conversations.list", "conv list");
    print_unified_output(&response, meta, raw)?;
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
    let raw = has_flag(args, "--raw");

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

    let (client, context) = get_api_client_with_context(profile).await?;
    let response = commands::conv_history(&client, channel, limit, oldest, latest)
        .await
        .map_err(|e| e.to_string())?;

    let meta = make_meta(&context, "conversations.history", "conv history");
    print_unified_output(&response, meta, raw)?;
    Ok(())
}

pub async fn run_users_info(args: &[String]) -> Result<(), String> {
    let user = args[3].clone();
    let profile = get_option(args, "--profile=");
    let raw = has_flag(args, "--raw");

    let (client, context) = get_api_client_with_context(profile).await?;
    let response = commands::users_info(&client, user)
        .await
        .map_err(|e| e.to_string())?;

    let meta = make_meta(&context, "users.info", "users info");
    print_unified_output(&response, meta, raw)?;
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
        return Err("Usage: msg post <channel> <text> [--thread-ts=TS] [--reply-broadcast] [--profile=NAME] [--raw]".to_string());
    }

    let channel = args[3].clone();
    let text = args[4].clone();
    let thread_ts = get_option(args, "--thread-ts=");
    let reply_broadcast = has_flag(args, "--reply-broadcast");
    let profile = get_option(args, "--profile=");
    let raw = has_flag(args, "--raw");

    // Validate: --reply-broadcast requires --thread-ts
    if reply_broadcast && thread_ts.is_none() {
        return Err("Error: --reply-broadcast requires --thread-ts".to_string());
    }

    let (client, context) = get_api_client_with_context(profile).await?;
    let response = commands::msg_post(&client, channel, text, thread_ts, reply_broadcast)
        .await
        .map_err(|e| e.to_string())?;

    let meta = make_meta(&context, "chat.postMessage", "msg post");
    print_unified_output(&response, meta, raw)?;
    Ok(())
}

pub async fn run_msg_update(args: &[String]) -> Result<(), String> {
    if args.len() < 6 {
        return Err(
            "Usage: msg update <channel> <ts> <text> [--yes] [--profile=NAME] [--raw]".to_string(),
        );
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let text = args[5].clone();
    let yes = has_flag(args, "--yes");
    let profile = get_option(args, "--profile=");
    let raw = has_flag(args, "--raw");

    let (client, context) = get_api_client_with_context(profile).await?;
    let response = commands::msg_update(&client, channel, ts, text, yes)
        .await
        .map_err(|e| e.to_string())?;

    let meta = make_meta(&context, "chat.update", "msg update");
    print_unified_output(&response, meta, raw)?;
    Ok(())
}

pub async fn run_msg_delete(args: &[String]) -> Result<(), String> {
    if args.len() < 5 {
        return Err(
            "Usage: msg delete <channel> <ts> [--yes] [--profile=NAME] [--raw]".to_string(),
        );
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let yes = has_flag(args, "--yes");
    let profile = get_option(args, "--profile=");
    let raw = has_flag(args, "--raw");

    let (client, context) = get_api_client_with_context(profile).await?;
    let response = commands::msg_delete(&client, channel, ts, yes)
        .await
        .map_err(|e| e.to_string())?;

    let meta = make_meta(&context, "chat.delete", "msg delete");
    print_unified_output(&response, meta, raw)?;
    Ok(())
}

pub async fn run_react_add(args: &[String]) -> Result<(), String> {
    if args.len() < 6 {
        return Err("Usage: react add <channel> <ts> <emoji> [--profile=NAME] [--raw]".to_string());
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let emoji = args[5].clone();
    let profile = get_option(args, "--profile=");
    let raw = has_flag(args, "--raw");

    let (client, context) = get_api_client_with_context(profile).await?;
    let response = commands::react_add(&client, channel, ts, emoji)
        .await
        .map_err(|e| e.to_string())?;

    let meta = make_meta(&context, "reactions.add", "react add");
    print_unified_output(&response, meta, raw)?;
    Ok(())
}

pub async fn run_react_remove(args: &[String]) -> Result<(), String> {
    if args.len() < 6 {
        return Err(
            "Usage: react remove <channel> <ts> <emoji> [--yes] [--profile=NAME] [--raw]"
                .to_string(),
        );
    }

    let channel = args[3].clone();
    let ts = args[4].clone();
    let emoji = args[5].clone();
    let yes = has_flag(args, "--yes");
    let profile = get_option(args, "--profile=");
    let raw = has_flag(args, "--raw");

    let (client, context) = get_api_client_with_context(profile).await?;
    let response = commands::react_remove(&client, channel, ts, emoji, yes)
        .await
        .map_err(|e| e.to_string())?;

    let meta = make_meta(&context, "reactions.remove", "react remove");
    print_unified_output(&response, meta, raw)?;
    Ok(())
}

pub async fn run_file_upload(args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        return Err(
            "Usage: file upload <path> [--channel=ID] [--channels=IDs] [--title=TITLE] [--comment=TEXT] [--profile=NAME] [--raw]"
                .to_string(),
        );
    }

    let file_path = args[3].clone();

    // Support both --channel and --channels
    let channels = get_option(args, "--channel=").or_else(|| get_option(args, "--channels="));
    let title = get_option(args, "--title=");
    let comment = get_option(args, "--comment=");
    let profile = get_option(args, "--profile=");
    let raw = has_flag(args, "--raw");

    let (client, context) = get_api_client_with_context(profile).await?;
    let response = commands::file_upload(&client, file_path, channels, title, comment)
        .await
        .map_err(|e| e.to_string())?;

    let meta = make_meta(
        &context,
        "files.getUploadURLExternal+files.completeUploadExternal",
        "file upload",
    );
    print_unified_output(&response, meta, raw)?;
    Ok(())
}

pub fn print_conv_usage(prog: &str) {
    println!("Conv command usage:");
    println!(
        "  {} conv list [--types=TYPE] [--limit=N] [--filter=KEY:VALUE]... [--profile=NAME] [--raw]",
        prog
    );
    println!("    Filters: name:<glob>, is_member:true|false, is_private:true|false");
    println!(
        "  {} conv select [--types=TYPE] [--filter=KEY:VALUE]... [--profile=NAME]",
        prog
    );
    println!("    Interactively select a conversation and output its channel ID");
    println!(
        "  {} conv history <channel> [--limit=N] [--oldest=TS] [--latest=TS] [--profile=NAME] [--raw]",
        prog
    );
    println!(
        "  {} conv history --interactive [--types=TYPE] [--filter=KEY:VALUE]... [--limit=N] [--profile=NAME] [--raw]",
        prog
    );
    println!("    Select channel interactively before fetching history");
}

pub fn print_users_usage(prog: &str) {
    println!("Users command usage:");
    println!("  {} users info <user_id> [--profile=NAME] [--raw]", prog);
    println!("  {} users cache-update [--profile=NAME] [--force]", prog);
    println!("  {} users resolve-mentions <text> [--profile=NAME] [--format=display_name|real_name|username]", prog);
}

pub fn print_msg_usage(prog: &str) {
    println!("Msg command usage:");
    println!(
        "  {} msg post <channel> <text> [--thread-ts=TS] [--reply-broadcast] [--profile=NAME] [--raw]",
        prog
    );
    println!(
        "  {} msg update <channel> <ts> <text> [--yes] [--profile=NAME] [--raw]",
        prog
    );
    println!(
        "  {} msg delete <channel> <ts> [--yes] [--profile=NAME] [--raw]",
        prog
    );
}

pub fn print_react_usage(prog: &str) {
    println!("React command usage:");
    println!(
        "  {} react add <channel> <ts> <emoji> [--profile=NAME] [--raw]",
        prog
    );
    println!(
        "  {} react remove <channel> <ts> <emoji> [--yes] [--profile=NAME] [--raw]",
        prog
    );
}

pub fn print_file_usage(prog: &str) {
    println!("File command usage:");
    println!(
        "  {} file upload <path> [--channel=ID] [--channels=IDs] [--title=TITLE] [--comment=TEXT] [--profile=NAME] [--raw]",
        prog
    );
}
