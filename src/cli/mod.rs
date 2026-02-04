//! CLI command routing and handlers

use crate::api::ApiClient;
use crate::commands;
use crate::profile::{
    default_config_path, load_config, make_token_key, KeyringTokenStore, TokenStore,
};

/// Get API client for a profile
pub async fn get_api_client(profile_name: Option<String>) -> Result<ApiClient, String> {
    let profile_name = profile_name.unwrap_or_else(|| "default".to_string());
    let config_path = default_config_path().map_err(|e| e.to_string())?;
    let config = load_config(&config_path).map_err(|e| e.to_string())?;

    let profile = config
        .get(&profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    let token_store = KeyringTokenStore::default_service();
    let token_key = make_token_key(&profile.team_id, &profile.user_id);
    let token = token_store
        .get(&token_key)
        .map_err(|e| format!("Failed to get token: {}", e))?;

    Ok(ApiClient::with_token(token))
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

pub async fn run_conv_list(args: &[String]) -> Result<(), String> {
    let types = get_option(args, "--types=");
    let limit = get_option(args, "--limit=").and_then(|s| s.parse().ok());
    let profile = get_option(args, "--profile=");

    let client = get_api_client(profile).await?;
    let response = commands::conv_list(&client, types, limit)
        .await
        .map_err(|e| e.to_string())?;

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    Ok(())
}

pub async fn run_conv_history(args: &[String]) -> Result<(), String> {
    let channel = args[3].clone();
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
        "  {} conv list [--types=TYPE] [--limit=N] [--profile=NAME]",
        prog
    );
    println!(
        "  {} conv history <channel> [--limit=N] [--oldest=TS] [--latest=TS] [--profile=NAME]",
        prog
    );
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
