// Allow unused imports - some are used only in lib.rs for public API
#![allow(unused_imports)]

mod api;
mod auth;
mod cli;
mod commands;
mod debug;
mod oauth;
mod profile;

use api::{execute_api_call, ApiCallArgs, ApiCallContext, ApiClient};
use cli::*;
use profile::{
    default_config_path, load_config, make_token_key, resolve_profile, resolve_profile_full,
    save_config, FileTokenStore, InMemoryTokenStore, KeyringTokenStore, Profile, ProfilesConfig,
    TokenStore,
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "--version" | "-v" => {
            print_version();
            return;
        }
        "api" => {
            if args.len() > 2 && args[2] == "call" {
                // Run api call command
                let api_args: Vec<String> = args[3..].to_vec();
                if let Err(e) = run_api_call(api_args).await {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            } else {
                print_api_usage();
            }
        }
        "auth" => {
            if args.len() < 3 {
                print_auth_usage();
                return;
            }
            match args[2].as_str() {
                "login" => {
                    if let Err(e) = run_auth_login(&args[3..]).await {
                        eprintln!("Login failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "status" => {
                    let profile_name = args.get(3).cloned();
                    if let Err(e) = auth::status(profile_name) {
                        eprintln!("Status command failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "list" => {
                    if let Err(e) = auth::list() {
                        eprintln!("List command failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "rename" => {
                    if args.len() < 5 {
                        eprintln!("Usage: {} auth rename <old_name> <new_name>", args[0]);
                        std::process::exit(1);
                    }
                    if let Err(e) = auth::rename(args[3].clone(), args[4].clone()) {
                        eprintln!("Rename command failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "logout" => {
                    let profile_name = args.get(3).cloned();
                    if let Err(e) = auth::logout(profile_name) {
                        eprintln!("Logout command failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "export" => {
                    handle_export_command(&args[3..]).await;
                }
                "import" => {
                    handle_import_command(&args[3..]).await;
                }
                _ => {
                    print_auth_usage();
                }
            }
        }
        "config" => {
            if args.len() < 3 {
                print_config_usage(&args[0]);
                return;
            }
            match args[2].as_str() {
                "oauth" => {
                    if args.len() < 4 {
                        print_config_oauth_usage(&args[0]);
                        std::process::exit(1);
                    }
                    match args[3].as_str() {
                        "set" => {
                            if let Err(e) = run_config_oauth_set(&args[4..]) {
                                eprintln!("OAuth config set failed: {}", e);
                                std::process::exit(1);
                            }
                        }
                        "show" => {
                            if let Err(e) = run_config_oauth_show(&args[4..]) {
                                eprintln!("OAuth config show failed: {}", e);
                                std::process::exit(1);
                            }
                        }
                        "delete" => {
                            if let Err(e) = run_config_oauth_delete(&args[4..]) {
                                eprintln!("OAuth config delete failed: {}", e);
                                std::process::exit(1);
                            }
                        }
                        _ => {
                            print_config_oauth_usage(&args[0]);
                        }
                    }
                }
                _ => {
                    print_config_usage(&args[0]);
                }
            }
        }
        "search" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} search <query> [--count=N] [--page=N] [--sort=TYPE] [--sort_dir=DIR] [--profile=NAME]",
                    args[0]
                );
                std::process::exit(1);
            }
            if let Err(e) = run_search(&args).await {
                eprintln!("Search failed: {}", e);
                std::process::exit(1);
            }
        }
        "conv" => {
            if args.len() < 3 {
                print_conv_usage(&args[0]);
                std::process::exit(1);
            }
            match args[2].as_str() {
                "list" => {
                    if let Err(e) = run_conv_list(&args).await {
                        eprintln!("Conv list failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "history" => {
                    if args.len() < 4 {
                        eprintln!(
                            "Usage: {} conv history <channel> [--limit=N] [--profile=NAME]",
                            args[0]
                        );
                        std::process::exit(1);
                    }
                    if let Err(e) = run_conv_history(&args).await {
                        eprintln!("Conv history failed: {}", e);
                        std::process::exit(1);
                    }
                }
                _ => print_conv_usage(&args[0]),
            }
        }
        "users" => {
            if args.len() < 3 {
                print_users_usage(&args[0]);
                std::process::exit(1);
            }
            match args[2].as_str() {
                "info" => {
                    if args.len() < 4 {
                        eprintln!("Usage: {} users info <user_id> [--profile=NAME]", args[0]);
                        std::process::exit(1);
                    }
                    if let Err(e) = run_users_info(&args).await {
                        eprintln!("Users info failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "cache-update" => {
                    if let Err(e) = run_users_cache_update(&args).await {
                        eprintln!("Users cache-update failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "resolve-mentions" => {
                    if let Err(e) = run_users_resolve_mentions(&args).await {
                        eprintln!("Users resolve-mentions failed: {}", e);
                        std::process::exit(1);
                    }
                }
                _ => print_users_usage(&args[0]),
            }
        }
        "msg" => {
            if args.len() < 3 {
                print_msg_usage(&args[0]);
                std::process::exit(1);
            }
            match args[2].as_str() {
                "post" => {
                    if let Err(e) = run_msg_post(&args).await {
                        eprintln!("Msg post failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "update" => {
                    if let Err(e) = run_msg_update(&args).await {
                        eprintln!("Msg update failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "delete" => {
                    if let Err(e) = run_msg_delete(&args).await {
                        eprintln!("Msg delete failed: {}", e);
                        std::process::exit(1);
                    }
                }
                _ => print_msg_usage(&args[0]),
            }
        }
        "react" => {
            if args.len() < 3 {
                print_react_usage(&args[0]);
                std::process::exit(1);
            }
            match args[2].as_str() {
                "add" => {
                    if let Err(e) = run_react_add(&args).await {
                        eprintln!("React add failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "remove" => {
                    if let Err(e) = run_react_remove(&args).await {
                        eprintln!("React remove failed: {}", e);
                        std::process::exit(1);
                    }
                }
                _ => print_react_usage(&args[0]),
            }
        }
        "file" => {
            if args.len() < 3 {
                print_file_usage(&args[0]);
                std::process::exit(1);
            }
            match args[2].as_str() {
                "upload" => {
                    if let Err(e) = run_file_upload(&args).await {
                        eprintln!("File upload failed: {}", e);
                        std::process::exit(1);
                    }
                }
                _ => print_file_usage(&args[0]),
            }
        }
        "demo" => {
            println!("Slack CLI - OAuth authentication flow");
            println!();
        }

        "--help" | "-h" => {
            print_help();
        }
        _ => {
            print_usage();
        }
    }
}

/// Print version information
fn print_version() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const NAME: &str = env!("CARGO_PKG_NAME");
    println!("{} {}", NAME, VERSION);
}

/// Print CLI help information
fn print_help() {
    println!("Slack CLI");
    println!();
    println!("USAGE:");
    println!("    slack-rs [COMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    api call <method> [params...]    Call a Slack API method");
    println!("    auth login [profile_name]        Authenticate with Slack");
    println!("    auth status [profile_name]       Show profile status");
    println!("    auth list                        List all profiles");
    println!("    auth rename <old> <new>          Rename a profile");
    println!("    auth logout [profile_name]       Remove authentication");
    println!("    config oauth set <profile>       Set OAuth configuration for a profile");
    println!("    config oauth show <profile>      Show OAuth configuration for a profile");
    println!("    config oauth delete <profile>    Delete OAuth configuration for a profile");
    println!("    search <query>                   Search messages");
    println!("    conv list                        List conversations");
    println!("    conv history <channel>           Get conversation history");
    println!("    users info <user_id>             Get user information");
    println!("    users cache-update               Update user cache for mention resolution");
    println!("    users resolve-mentions <text>    Resolve user mentions in text");
    println!("    msg post <channel> <text>        Post a message (supports --thread-ts and --reply-broadcast)");
    println!("    msg update <channel> <ts> <text> Update a message");
    println!("    msg delete <channel> <ts>        Delete a message");
    println!("    react add <channel> <ts> <emoji> Add a reaction");
    println!("    react remove <channel> <ts> <emoji> Remove a reaction");
    println!("    file upload <path>               Upload a file (external upload method)");
    println!("    demo                             Run demonstration");
    println!();
    println!("API CALL OPTIONS:");
    println!("    <method>                         Slack API method (e.g., chat.postMessage)");
    println!("    key=value                        Request parameters");
    println!("    --json                           Send as JSON body (default: form-urlencoded)");
    println!("    --get                            Use GET method (default: POST)");
    println!();
    println!("EXAMPLES:");
    println!("    slack-rs api call users.info user=U123456 --get");
    println!("    slack-rs api call chat.postMessage channel=C123 text=Hello");
    println!("    slack-rs api call chat.postMessage --json channel=C123 text=Hello");
}

fn print_usage() {
    println!("Slack CLI - Usage:");
    println!("  api call <method> [params...]  - Call a Slack API method");
    println!("  auth login [profile_name]      - Authenticate with Slack");
    println!("  auth status [profile_name]     - Show profile status");
    println!("  auth list                      - List all profiles");
    println!("  auth rename <old> <new>        - Rename a profile");
    println!("  auth logout [profile_name]     - Remove authentication");
    println!("  auth export [options]          - Export profiles to encrypted file");
    println!("  auth import [options]          - Import profiles from encrypted file");
    println!("  config oauth set <profile>     - Set OAuth configuration for a profile");
    println!("  config oauth show <profile>    - Show OAuth configuration for a profile");
    println!("  config oauth delete <profile>  - Delete OAuth configuration for a profile");
    println!("  search <query>                 - Search messages (supports --count, --page, --sort, --sort_dir)");
    println!("  conv list                      - List conversations");
    println!("  conv history <channel>         - Get conversation history");
    println!("  users info <user_id>           - Get user information");
    println!("  users cache-update             - Update user cache for mention resolution (supports --profile, --force)");
    println!("  users resolve-mentions <text>  - Resolve user mentions in text (supports --profile, --format)");
    println!("  msg post <channel> <text>      - Post a message (requires --allow-write, supports --thread-ts and --reply-broadcast)");
    println!("  msg update <channel> <ts> <text> - Update a message (requires --allow-write)");
    println!("  msg delete <channel> <ts>      - Delete a message (requires --allow-write)");
    println!("  react add <channel> <ts> <emoji> - Add a reaction (requires --allow-write)");
    println!("  react remove <channel> <ts> <emoji> - Remove a reaction (requires --allow-write)");
    println!("  file upload <path>             - Upload a file using external upload method");
    println!("  demo                           - Run demonstration");
    println!("  --help, -h                     - Show help");
    println!("  --version, -v                  - Show version");
}

fn print_api_usage() {
    println!("API command usage:");
    println!("  api call <method> [params...]  - Call a Slack API method");
    println!();
    println!("OPTIONS:");
    println!("    <method>                     Slack API method (e.g., chat.postMessage)");
    println!("    key=value                    Request parameters");
    println!("    --json                       Send as JSON body (default: form-urlencoded)");
    println!("    --get                        Use GET method (default: POST)");
    println!();
    println!("EXAMPLES:");
    println!("    slack-rs api call users.info user=U123456 --get");
    println!("    slack-rs api call chat.postMessage channel=C123 text=Hello");
}

fn print_auth_usage() {
    println!("Auth command usage:");
    println!("  auth login [profile_name] [options] - Authenticate with Slack");
    println!("  auth status [profile_name]          - Show profile status");
    println!("  auth list                           - List all profiles");
    println!("  auth rename <old> <new>             - Rename a profile");
    println!("  auth logout [profile_name]          - Remove authentication");
    println!("  auth export [options]               - Export profiles to encrypted file");
    println!("  auth import [options]               - Import profiles from encrypted file");
    println!();
    println!("Login options:");
    println!("  --client-id <id>                    - OAuth client ID (optional)");
    println!("  --bot-scopes <scopes>               - Bot scopes (comma-separated or 'all')");
    println!("  --user-scopes <scopes>              - User scopes (comma-separated or 'all')");
    println!("  --cloudflared [path]                - Use cloudflared tunnel for redirect URI");
    println!("                                        (path optional, defaults to 'cloudflared' in PATH)");
    println!("  --ngrok [path]                      - Use ngrok tunnel for redirect URI");
    println!(
        "                                        (path optional, defaults to 'ngrok' in PATH)"
    );
    println!();
    println!("Cloudflared tunnel usage:");
    println!(
        "  When --cloudflared is specified, a temporary tunnel is created for OAuth callback."
    );
    println!("  The generated manifest will include https://*.trycloudflare.com/callback in redirect_urls.");
    println!("  Make sure your Slack App is configured with this wildcard URL.");
    println!();
    println!("Ngrok tunnel usage:");
    println!("  When --ngrok is specified, a temporary tunnel is created for OAuth callback.");
    println!(
        "  The generated manifest will include https://*.ngrok-free.app/callback in redirect_urls."
    );
    println!("  Make sure your Slack App is configured with this wildcard URL.");
    println!("  Note: --cloudflared and --ngrok cannot be used at the same time.");
    println!("  Note: Custom ngrok domains are not supported in this implementation.");
    println!();
    println!("Manifest generation:");
    println!("  After successful authentication, a Slack App Manifest is automatically generated");
    println!("  and saved to ~/.config/slack-rs/<profile>_manifest.yml");
    println!(
        "  This manifest can be uploaded to https://api.slack.com/apps for easy app configuration."
    );
    println!();
    println!("Export options:");
    println!(
        "  --profile <name>                    - Export specific profile (default: 'default')"
    );
    println!("  --all                               - Export all profiles");
    println!("  --out <file>                        - Output file path (required)");
    println!("  --passphrase-env <var>              - Environment variable containing passphrase");
    println!("  --passphrase-prompt                 - Prompt for passphrase");
    println!("  --yes                               - Confirm dangerous operation (required)");
    println!();
    println!("Import options:");
    println!("  --in <file>                         - Input file path (required)");
    println!("  --passphrase-env <var>              - Environment variable containing passphrase");
    println!("  --passphrase-prompt                 - Prompt for passphrase");
    println!("  --yes                               - Automatically accept conflicts");
    println!("  --force                             - Overwrite existing profiles");
}

fn print_config_usage(prog: &str) {
    println!("Config command usage:");
    println!(
        "  {} config oauth set <profile> --client-id <id> --redirect-uri <uri> --scopes <scopes>",
        prog
    );
    println!("  {} config oauth show <profile>", prog);
    println!("  {} config oauth delete <profile>", prog);
}

fn print_config_oauth_usage(prog: &str) {
    println!("OAuth config command usage:");
    println!(
        "  {} config oauth set <profile> --client-id <id> --redirect-uri <uri> --scopes <scopes>",
        prog
    );
    println!("      Set OAuth configuration for a profile");
    println!("      Will prompt for client secret (not stored in config file)");
    println!("      Scopes: comma-separated list or 'all' for comprehensive preset");
    println!();
    println!("  {} config oauth show <profile>", prog);
    println!("      Show OAuth configuration for a profile");
    println!();
    println!("  {} config oauth delete <profile>", prog);
    println!("      Delete OAuth configuration for a profile");
    println!();
    println!("Examples:");
    println!("  {} config oauth set work --client-id 123.456 --redirect-uri http://127.0.0.1:8765/callback --scopes \"chat:write,users:read\"", prog);
    println!("  {} config oauth set work --client-id 123.456 --redirect-uri http://127.0.0.1:8765/callback --scopes \"all\"", prog);
    println!("  {} config oauth show work", prog);
    println!("  {} config oauth delete work", prog);
}

/// Run the auth login command with argument parsing
async fn run_auth_login(args: &[String]) -> Result<(), String> {
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

    // Use default redirect_uri
    let redirect_uri = "http://127.0.0.1:8765/callback".to_string();

    // Keep base_url from environment for testing purposes only
    let base_url = std::env::var("SLACK_OAUTH_BASE_URL").ok();

    // If cloudflared or ngrok is specified, use extended login flow
    if cloudflared_path.is_some() || ngrok_path.is_some() {
        // Prompt for client_id if not provided
        let client_id = if let Some(id) = client_id {
            id
        } else {
            use std::io::{self, Write};
            print!("Enter Slack Client ID: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        };

        // Use default scopes if not provided
        let bot_scopes = bot_scopes.unwrap_or_else(oauth::bot_all_scopes);
        let user_scopes = user_scopes.unwrap_or_else(oauth::user_all_scopes);

        if debug::enabled() {
            debug::log("Preparing to call login_with_credentials_extended");
            debug::log(format!("bot_scopes_count={}", bot_scopes.len()));
            debug::log(format!("user_scopes_count={}", user_scopes.len()));
        }

        // Prompt for client_secret
        let client_secret = auth::prompt_for_client_secret()
            .map_err(|e| format!("Failed to read client secret: {}", e))?;

        // Call extended login with cloudflared support
        auth::login_with_credentials_extended(
            client_id,
            client_secret,
            bot_scopes,
            user_scopes,
            profile_name,
            cloudflared_path.is_some(),
        )
        .await
        .map_err(|e| e.to_string())
    } else {
        // Call standard login with credentials
        // This will prompt for client_secret and other missing OAuth config
        auth::login_with_credentials(
            client_id,
            profile_name,
            redirect_uri,
            vec![], // Legacy scopes parameter (unused)
            bot_scopes,
            user_scopes,
            base_url,
        )
        .await
        .map_err(|e| e.to_string())
    }
}

/// Run config oauth set command
fn run_config_oauth_set(args: &[String]) -> Result<(), String> {
    let mut profile_name: Option<String> = None;
    let mut client_id: Option<String> = None;
    let mut redirect_uri: Option<String> = None;
    let mut scopes: Option<String> = None;

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
                "--redirect-uri" => {
                    i += 1;
                    if i < args.len() {
                        redirect_uri = Some(args[i].clone());
                    } else {
                        return Err("--redirect-uri requires a value".to_string());
                    }
                }
                "--scopes" => {
                    i += 1;
                    if i < args.len() {
                        scopes = Some(args[i].clone());
                    } else {
                        return Err("--scopes requires a value".to_string());
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

    let profile = profile_name.ok_or_else(|| "Profile name is required".to_string())?;
    let client = client_id.ok_or_else(|| "--client-id is required".to_string())?;
    let redirect = redirect_uri.ok_or_else(|| "--redirect-uri is required".to_string())?;
    let scope_str = scopes.ok_or_else(|| "--scopes is required".to_string())?;

    commands::oauth_set(profile, client, redirect, scope_str).map_err(|e| e.to_string())
}

/// Run config oauth show command
fn run_config_oauth_show(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Profile name is required".to_string());
    }

    let profile_name = args[0].clone();
    commands::oauth_show(profile_name).map_err(|e| e.to_string())
}

/// Run config oauth delete command
fn run_config_oauth_delete(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Profile name is required".to_string());
    }

    let profile_name = args[0].clone();
    commands::oauth_delete(profile_name).map_err(|e| e.to_string())
}

async fn handle_export_command(args: &[String]) {
    let mut profile_name: Option<String> = None;
    let mut all = false;
    let mut output_path: Option<String> = None;
    let mut passphrase_env: Option<String> = None;
    let mut yes = false;
    let mut lang: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--profile" => {
                i += 1;
                if i < args.len() {
                    profile_name = Some(args[i].clone());
                }
            }
            "--all" => {
                all = true;
            }
            "--out" => {
                i += 1;
                if i < args.len() {
                    output_path = Some(args[i].clone());
                }
            }
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
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Set up i18n messages
    let messages = if let Some(lang_code) = lang {
        if let Some(language) = auth::Language::from_code(&lang_code) {
            auth::Messages::new(language)
        } else {
            auth::Messages::default()
        }
    } else {
        auth::Messages::default()
    };

    // Show warning and validate --yes
    if !yes {
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

    // Get passphrase (fallback to prompt if env not specified or not set)
    let passphrase = if let Some(env_var) = passphrase_env {
        match std::env::var(&env_var) {
            Ok(val) => val,
            Err(_) => {
                // Fallback to prompt if environment variable is not set
                eprintln!(
                    "Warning: Environment variable {} not found, prompting for passphrase",
                    env_var
                );
                match rpassword::prompt_password(messages.get("prompt.passphrase")) {
                    Ok(pass) => pass,
                    Err(e) => {
                        eprintln!("Error reading passphrase: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    } else {
        // Fallback to prompt mode
        match rpassword::prompt_password(messages.get("prompt.passphrase")) {
            Ok(pass) => pass,
            Err(e) => {
                eprintln!("Error reading passphrase: {}", e);
                std::process::exit(1);
            }
        }
    };

    let options = auth::ExportOptions {
        profile_name,
        all,
        output_path: output,
        passphrase,
        yes,
    };

    let token_store = FileTokenStore::new().expect("Failed to create token store");
    match auth::export_profiles(&token_store, &options) {
        Ok(_) => {
            println!("{}", messages.get("success.export"));
        }
        Err(e) => {
            eprintln!("Export failed: {}", e);
            std::process::exit(1);
        }
    }
}

async fn handle_import_command(args: &[String]) {
    let mut input_path: Option<String> = None;
    let mut passphrase_env: Option<String> = None;
    let mut yes = false;
    let mut force = false;
    let mut lang: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--in" => {
                i += 1;
                if i < args.len() {
                    input_path = Some(args[i].clone());
                }
            }
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
            "--force" => {
                force = true;
            }
            "--lang" => {
                i += 1;
                if i < args.len() {
                    lang = Some(args[i].clone());
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Set up i18n messages
    let messages = if let Some(lang_code) = lang {
        if let Some(language) = auth::Language::from_code(&lang_code) {
            auth::Messages::new(language)
        } else {
            auth::Messages::default()
        }
    } else {
        auth::Messages::default()
    };

    // Validate required options
    let input = match input_path {
        Some(path) => path,
        None => {
            eprintln!("Error: --in <file> is required");
            std::process::exit(1);
        }
    };

    // Get passphrase (fallback to prompt if env not specified or not set)
    let passphrase = if let Some(env_var) = passphrase_env {
        match std::env::var(&env_var) {
            Ok(val) => val,
            Err(_) => {
                // Fallback to prompt if environment variable is not set
                eprintln!(
                    "Warning: Environment variable {} not found, prompting for passphrase",
                    env_var
                );
                match rpassword::prompt_password(messages.get("prompt.passphrase")) {
                    Ok(pass) => pass,
                    Err(e) => {
                        eprintln!("Error reading passphrase: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    } else {
        // Fallback to prompt mode
        match rpassword::prompt_password(messages.get("prompt.passphrase")) {
            Ok(pass) => pass,
            Err(e) => {
                eprintln!("Error reading passphrase: {}", e);
                std::process::exit(1);
            }
        }
    };

    let options = auth::ImportOptions {
        input_path: input,
        passphrase,
        yes,
        force,
    };

    let token_store = FileTokenStore::new().expect("Failed to create token store");
    match auth::import_profiles(&token_store, &options) {
        Ok(_) => {
            println!("{}", messages.get("success.import"));
        }
        Err(e) => {
            eprintln!("Import failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Run the api call command
async fn run_api_call(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments
    let api_args = ApiCallArgs::parse(&args)?;

    // Determine profile name (from environment or default to "default")
    let profile_name = std::env::var("SLACK_PROFILE").unwrap_or_else(|_| "default".to_string());

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

    // Create token key from team_id and user_id
    let token_key = make_token_key(&profile.team_id, &profile.user_id);

    // Retrieve token from token store
    // Try file store first, fall back to environment variable
    let token = {
        let file_store =
            FileTokenStore::new().map_err(|e| format!("Failed to create token store: {}", e))?;
        match file_store.get(&token_key) {
            Ok(t) => t,
            Err(_) => {
                // If file store fails, check if there's a token in environment
                if let Ok(env_token) = std::env::var("SLACK_TOKEN") {
                    env_token
                } else {
                    return Err(format!(
                        "No token found for profile '{}' ({}:{}). Set SLACK_TOKEN environment variable or store token in file store.",
                        profile_name, profile.team_id, profile.user_id
                    ).into());
                }
            }
        }
    };

    // Create API client
    let client = ApiClient::new();

    // Execute API call
    let response = execute_api_call(&client, &api_args, &token, &context).await?;

    // Print response as JSON
    let json = serde_json::to_string_pretty(&response)?;
    println!("{}", json);

    Ok(())
}

/// Demonstrates the profile storage functionality
#[allow(dead_code)]
fn demonstrate_profile_storage() {
    println!("=== Profile Storage Demo ===");

    // Get default config path
    match default_config_path() {
        Ok(path) => {
            println!("Config path: {}", path.display());

            // Load existing config or create new
            match load_config(&path) {
                Ok(config) => {
                    println!("Loaded config with {} profiles", config.profiles.len());

                    // List profiles
                    if !config.profiles.is_empty() {
                        println!("Profiles:");
                        for name in config.list_names() {
                            if let Some(profile) = config.get(&name) {
                                println!(
                                    "  - {}: {} ({}:{})",
                                    name,
                                    profile.team_name.as_deref().unwrap_or("N/A"),
                                    profile.team_id,
                                    profile.user_id
                                );
                            }
                        }
                    }

                    // Demonstrate profile resolution
                    if let Some(name) = config.list_names().first() {
                        match resolve_profile(&path, name) {
                            Ok((team_id, user_id)) => {
                                println!("\nResolved '{}' -> {}:{}", name, team_id, user_id);
                            }
                            Err(e) => {
                                println!("\nFailed to resolve profile: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to load config: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to get config path: {}", e);
        }
    }
    println!();
}

/// Demonstrates the token storage functionality
#[allow(dead_code)]
fn demonstrate_token_storage() {
    println!("=== Token Storage Demo ===");

    // Use in-memory store for demonstration (keyring requires OS integration)
    let store = InMemoryTokenStore::new();

    // Create a sample token key
    let key = make_token_key("T123ABC", "U456DEF");
    println!("Token key: {}", key);

    // Store a token
    match store.set(&key, "xoxb-sample-token") {
        Ok(_) => println!("Token stored successfully"),
        Err(e) => println!("Failed to store token: {}", e),
    }

    // Retrieve the token
    match store.get(&key) {
        Ok(token) => println!("Retrieved token: {}", token),
        Err(e) => println!("Failed to retrieve token: {}", e),
    }

    // Check if token exists
    println!("Token exists: {}", store.exists(&key));

    // Note about FileTokenStore and KeyringTokenStore
    println!("\nNote: FileTokenStore is the default for production use:");
    println!("  let store = FileTokenStore::new().unwrap();");
    println!("  // Stores tokens in ~/.config/slack-rs/tokens.json with 0600 permissions");
    println!("\nKeyringTokenStore is also available:");
    println!("  let store = KeyringTokenStore::default_service();");
    println!("  // Uses OS keyring with service='slack-rs'");

    println!();
}

/// Example of creating and managing profiles programmatically
#[allow(dead_code)]
fn example_profile_management() {
    let mut config = ProfilesConfig::new();

    // Add a profile
    let profile = Profile {
        team_id: "T123ABC".to_string(),
        user_id: "U456DEF".to_string(),
        team_name: Some("Example Team".to_string()),
        user_name: Some("Example User".to_string()),
        client_id: None,
        redirect_uri: None,
        scopes: None,
        bot_scopes: None,
        user_scopes: None,
    };

    // Use add() to prevent duplicates
    match config.add("default".to_string(), profile.clone()) {
        Ok(_) => println!("Profile added"),
        Err(e) => println!("Failed to add profile: {}", e),
    }

    // Use set_or_update() for smart updates
    match config.set_or_update("default".to_string(), profile) {
        Ok(_) => println!("Profile updated"),
        Err(e) => println!("Failed to update profile: {}", e),
    }
}

/// Demonstrates profile persistence (save and reload)
#[allow(dead_code)]
fn demonstrate_profile_persistence() {
    println!("=== Profile Persistence Demo ===");

    // Create a temporary config for demonstration
    let mut config = ProfilesConfig::new();

    // Add profiles using add() and set_or_update()
    let profile1 = Profile {
        team_id: "T123ABC".to_string(),
        user_id: "U456DEF".to_string(),
        team_name: Some("Example Team".to_string()),
        user_name: Some("Example User".to_string()),
        client_id: None,
        redirect_uri: None,
        scopes: None,
        bot_scopes: None,
        user_scopes: None,
    };

    let profile2 = Profile {
        team_id: "T789GHI".to_string(),
        user_id: "U012JKL".to_string(),
        team_name: Some("Another Team".to_string()),
        user_name: Some("Another User".to_string()),
        client_id: None,
        redirect_uri: None,
        scopes: None,
        bot_scopes: None,
        user_scopes: None,
    };

    // Demonstrate add() - should succeed for new profile
    match config.add("work".to_string(), profile1) {
        Ok(_) => println!("Added 'work' profile using add()"),
        Err(e) => println!("Failed to add profile: {}", e),
    }

    // Demonstrate set_or_update() - should succeed for new profile
    match config.set_or_update("personal".to_string(), profile2.clone()) {
        Ok(_) => println!("Added 'personal' profile using set_or_update()"),
        Err(e) => println!("Failed to add profile: {}", e),
    }

    // Demonstrate set_or_update() with same identity - should update
    let updated_profile2 = Profile {
        team_id: "T789GHI".to_string(),
        user_id: "U012JKL".to_string(),
        team_name: Some("Updated Team Name".to_string()),
        user_name: Some("Updated User Name".to_string()),
        client_id: None,
        redirect_uri: None,
        scopes: None,
        bot_scopes: None,
        user_scopes: None,
    };
    match config.set_or_update("personal".to_string(), updated_profile2) {
        Ok(_) => println!("Updated 'personal' profile using set_or_update()"),
        Err(e) => println!("Failed to update profile: {}", e),
    }

    // Save config to temp location for demonstration
    if let Ok(_config_path) = default_config_path() {
        // Create a test path in a temp directory
        let temp_dir = std::env::temp_dir();
        let test_config_path = temp_dir.join("slack-rs_test_profiles.json");

        match save_config(&test_config_path, &config) {
            Ok(_) => {
                println!("Saved config to: {}", test_config_path.display());

                // Reload to verify persistence
                match load_config(&test_config_path) {
                    Ok(loaded_config) => {
                        println!("Reloaded config successfully");
                        println!("Profiles count: {}", loaded_config.profiles.len());

                        // Verify profiles were saved correctly
                        if let Some(work_profile) = loaded_config.get("work") {
                            println!(
                                "  work: {} ({}:{})",
                                work_profile.team_name.as_deref().unwrap_or("N/A"),
                                work_profile.team_id,
                                work_profile.user_id
                            );
                        }
                        if let Some(personal_profile) = loaded_config.get("personal") {
                            println!(
                                "  personal: {} ({}:{})",
                                personal_profile.team_name.as_deref().unwrap_or("N/A"),
                                personal_profile.team_id,
                                personal_profile.user_id
                            );
                        }

                        // Clean up test file
                        let _ = std::fs::remove_file(&test_config_path);
                    }
                    Err(e) => println!("Failed to reload config: {}", e),
                }
            }
            Err(e) => println!("Failed to save config: {}", e),
        }
    } else {
        println!("Failed to get default config path");
    }

    println!();
}

/// Demonstrates keyring token storage using KeyringTokenStore
#[allow(dead_code)]
fn demonstrate_keyring_token_storage() {
    println!("=== Keyring Token Storage Demo ===");

    // Create KeyringTokenStore with default service name
    let keyring_store = KeyringTokenStore::default_service();
    println!("Created KeyringTokenStore with service='slack-rs'");

    // Create a test token key
    let key = make_token_key("T123ABC", "U456DEF");
    println!("Token key: {}", key);

    // Attempt to store a token in keyring
    let test_token = "xoxb-demo-token-12345";
    match keyring_store.set(&key, test_token) {
        Ok(_) => {
            println!("✓ Token stored in OS keyring successfully");

            // Retrieve the token
            match keyring_store.get(&key) {
                Ok(retrieved_token) => {
                    println!("✓ Retrieved token from keyring");
                    // Verify it matches (show partial for security)
                    if retrieved_token == test_token {
                        println!("✓ Token verification successful");
                    }
                }
                Err(e) => println!("✗ Failed to retrieve token: {}", e),
            }

            // Check existence
            if keyring_store.exists(&key) {
                println!("✓ Token existence check successful");
            }

            // Delete the test token
            match keyring_store.delete(&key) {
                Ok(_) => println!("✓ Token deleted from keyring successfully"),
                Err(e) => println!("✗ Failed to delete token: {}", e),
            }

            // Verify deletion
            if !keyring_store.exists(&key) {
                println!("✓ Token deletion verified");
            }
        }
        Err(e) => {
            println!("✗ Failed to store token in keyring: {}", e);
            println!("  This may happen if the keyring is not available on this system");
            println!("  For production use, the keyring integration is fully implemented");
        }
    }

    println!();
}
