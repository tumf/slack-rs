// Use library exports instead of module declarations to avoid duplicate test runs
use slack_rs::cli::*;
use slack_rs::profile::{
    default_config_path, load_config, make_token_key, resolve_profile, save_config,
    InMemoryTokenStore, KeyringTokenStore, Profile, ProfilesConfig, TokenStore,
};
use slack_rs::{auth, cli, commands, profile};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse global --non-interactive flag
    let non_interactive = cli::has_flag(&args, "--non-interactive");
    let ctx = cli::CliContext::new(non_interactive);

    if args.len() < 2 {
        print_usage();
        return;
    }

    // Early check for --help --json (applies to all commands)
    if cli::has_flag(&args, "--help") && cli::has_flag(&args, "--json") {
        // Extract command name from args (skip program name and filter out flags)
        let command_parts: Vec<String> = args[1..]
            .iter()
            .filter(|arg| !arg.starts_with("--"))
            .map(|s| s.to_string())
            .collect();

        if !command_parts.is_empty() {
            let command_name = command_parts.join(" ");
            match cli::generate_help(&command_name) {
                Ok(help) => {
                    let json = serde_json::to_string_pretty(&help).unwrap();
                    println!("{}", json);
                    return;
                }
                Err(e) => {
                    eprintln!("Help generation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
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
                if let Err(e) = cli::run_api_call(api_args).await {
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
                    if let Err(e) = cli::run_auth_login(&args[3..], ctx.is_non_interactive()).await
                    {
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
                    cli::handle_export_command(&args[3..]).await;
                }
                "import" => {
                    cli::handle_import_command(&args[3..]).await;
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
                "set" => {
                    if let Err(e) = run_config_set(&args[3..]) {
                        eprintln!("Config set failed: {}", e);
                        std::process::exit(1);
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
                "select" => {
                    if let Err(e) = run_conv_select(&args).await {
                        eprintln!("Conv select failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "search" => {
                    if let Err(e) = run_conv_search(&args).await {
                        eprintln!("Conv search failed: {}", e);
                        std::process::exit(1);
                    }
                }
                "history" => {
                    // --interactive flag makes channel argument optional
                    let has_interactive = args.iter().any(|arg| arg == "--interactive");
                    if !has_interactive && args.len() < 4 {
                        eprintln!(
                            "Usage: {} conv history <channel> [--limit=N] [--profile=NAME]",
                            args[0]
                        );
                        eprintln!(
                            "   or: {} conv history --interactive [--filter=KEY:VALUE]... [--profile=NAME]",
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
                    if let Err(e) = run_msg_update(&args, ctx.is_non_interactive()).await {
                        eprintln!("Msg update failed: {}", e);
                        if cli::is_non_interactive_error(&e) {
                            std::process::exit(2);
                        }
                        std::process::exit(1);
                    }
                }
                "delete" => {
                    if let Err(e) = run_msg_delete(&args, ctx.is_non_interactive()).await {
                        eprintln!("Msg delete failed: {}", e);
                        if cli::is_non_interactive_error(&e) {
                            std::process::exit(2);
                        }
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
                    if let Err(e) = run_react_remove(&args, ctx.is_non_interactive()).await {
                        eprintln!("React remove failed: {}", e);
                        if cli::is_non_interactive_error(&e) {
                            std::process::exit(2);
                        }
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
        "commands" => {
            // Check for --json flag
            if cli::has_flag(&args, "--json") {
                let response = cli::generate_commands_list();
                let json = serde_json::to_string_pretty(&response).unwrap();
                println!("{}", json);
            } else {
                eprintln!("Usage: {} commands --json", args[0]);
                std::process::exit(1);
            }
        }
        "schema" => {
            // Parse --command and --output flags
            let command = cli::get_option(&args, "--command=");
            let output = cli::get_option(&args, "--output=");

            if let (Some(cmd), Some(out)) = (command, output) {
                if out == "json-schema" {
                    match cli::generate_schema(&cmd) {
                        Ok(schema_response) => {
                            let json = serde_json::to_string_pretty(&schema_response).unwrap();
                            println!("{}", json);
                        }
                        Err(e) => {
                            eprintln!("Schema error: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Invalid output format. Use --output json-schema");
                    std::process::exit(1);
                }
            } else {
                eprintln!(
                    "Usage: {} schema --command <cmd> --output json-schema",
                    args[0]
                );
                std::process::exit(1);
            }
        }
        "demo" => {
            println!("Slack CLI - OAuth authentication flow");
            println!();
        }

        "--help" | "-h" => {
            // Check for --json flag
            if cli::has_flag(&args, "--json") {
                // For top-level help, show all commands
                let response = cli::generate_commands_list();
                let json = serde_json::to_string_pretty(&response).unwrap();
                println!("{}", json);
            } else {
                print_help();
            }
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
    println!("    slack-rs [--non-interactive] [COMMAND] [OPTIONS]");
    println!();
    println!("GLOBAL OPTIONS:");
    println!("    --non-interactive              Run without interactive prompts (auto-enabled when stdin is not a TTY)");
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
    println!("    config set <profile> --token-type <type>  Set default token type (bot/user)");
    println!("    search <query>                   Search messages");
    println!("    conv list                        List conversations (supports --filter, --format, --sort)");
    println!("    conv search <pattern>            Search conversations by name");
    println!("    conv select                      Interactively select a conversation");
    println!(
        "    conv history <channel>           Get conversation history (supports --interactive)"
    );
    println!("    users info <user_id>             Get user information");
    println!("    users cache-update               Update user cache for mention resolution");
    println!("    users resolve-mentions <text>    Resolve user mentions in text");
    println!("    msg post <channel> <text>        Post a message (requires SLACKCLI_ALLOW_WRITE=true, supports --thread-ts and --reply-broadcast)");
    println!("    msg update <channel> <ts> <text> Update a message (requires SLACKCLI_ALLOW_WRITE=true)");
    println!("    msg delete <channel> <ts>        Delete a message (requires SLACKCLI_ALLOW_WRITE=true)");
    println!(
        "    react add <channel> <ts> <emoji> Add a reaction (requires SLACKCLI_ALLOW_WRITE=true)"
    );
    println!("    react remove <channel> <ts> <emoji> Remove a reaction (requires SLACKCLI_ALLOW_WRITE=true)");
    println!("    file upload <path>               Upload a file (external upload method)");
    println!("    demo                             Run demonstration");
    println!();
    println!("API CALL OPTIONS:");
    println!("    <method>                         Slack API method (e.g., chat.postMessage)");
    println!("    key=value                        Request parameters");
    println!("    --json                           Send as JSON body (default: form-urlencoded)");
    println!("    --get                            Use GET method (default: POST)");
    println!(
        "    --raw                            Output raw Slack API response (without envelope)"
    );
    println!();
    println!("OUTPUT:");
    println!("    All commands output JSON with unified envelope: {{response, meta}}");
    println!("    Use --raw flag to get raw Slack API response (for backward compatibility)");
    println!();
    println!("EXAMPLES:");
    println!("    slack-rs api call users.info user=U123456 --get");
    println!("    slack-rs api call chat.postMessage channel=C123 text=Hello");
    println!("    slack-rs api call chat.postMessage --json channel=C123 text=Hello");
    println!("    slack-rs conv list --raw  # Get raw response without envelope");
}

fn print_usage() {
    println!("Slack CLI - Usage:");
    println!("  [--non-interactive]                Run without interactive prompts (auto when stdin not a TTY)");
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
    println!("  config set <profile> --token-type <type> - Set default token type (bot/user)");
    println!("  search <query>                 - Search messages (supports --count, --page, --sort, --sort_dir)");
    println!("  conv list                      - List conversations (supports --filter, --format, --sort)");
    println!("  conv search <pattern>          - Search conversations by name (supports --select)");
    println!("  conv select                    - Interactively select a conversation");
    println!(
        "  conv history <channel>         - Get conversation history (supports --interactive)"
    );
    println!("  users info <user_id>           - Get user information");
    println!("  users cache-update             - Update user cache for mention resolution (supports --profile, --force)");
    println!("  users resolve-mentions <text>  - Resolve user mentions in text (supports --profile, --format)");
    println!("  msg post <channel> <text>      - Post a message (requires SLACKCLI_ALLOW_WRITE=true, supports --thread-ts and --reply-broadcast)");
    println!("  msg update <channel> <ts> <text> - Update a message (requires SLACKCLI_ALLOW_WRITE=true)");
    println!(
        "  msg delete <channel> <ts>      - Delete a message (requires SLACKCLI_ALLOW_WRITE=true)"
    );
    println!(
        "  react add <channel> <ts> <emoji> - Add a reaction (requires SLACKCLI_ALLOW_WRITE=true)"
    );
    println!("  react remove <channel> <ts> <emoji> - Remove a reaction (requires SLACKCLI_ALLOW_WRITE=true)");
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
    println!("    --raw                        Output raw Slack API response (without envelope)");
    println!();
    println!("OUTPUT FORMAT:");
    println!("    Default: JSON with 'response' and 'meta' fields (unified envelope)");
    println!("    With --raw: Raw Slack API response only (for backward compatibility)");
    println!();
    println!("EXAMPLES:");
    println!("    slack-rs api call users.info user=U123456 --get");
    println!("    slack-rs api call chat.postMessage channel=C123 text=Hello");
    println!("    slack-rs api call conversations.list --raw  # Raw output without envelope");
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
    println!(
        "  {} config set <profile> --token-type <type>  - Set default token type (bot/user)",
        prog
    );
}

fn print_config_oauth_usage(prog: &str) {
    println!("OAuth config command usage:");
    println!(
        "  {} config oauth set <profile> --client-id <id> --redirect-uri <uri> --scopes <scopes> [secret-options]",
        prog
    );
    println!("      Set OAuth configuration for a profile");
    println!("      Scopes: comma-separated list or 'all' for comprehensive preset");
    println!();
    println!("Client secret options (in priority order):");
    println!("  --client-secret-env <VAR>      Read secret from environment variable");
    println!("  (SLACKRS_CLIENT_SECRET)        Default environment variable (auto-checked)");
    println!("  --client-secret-file <PATH>    Read secret from file");
    println!("  --client-secret <SECRET>       Direct secret value (requires --yes, unsafe)");
    println!("  (interactive prompt)           Prompt for secret if stdin is a TTY");
    println!();
    println!("  {} config oauth show <profile>", prog);
    println!("      Show OAuth configuration for a profile");
    println!();
    println!("  {} config oauth delete <profile>", prog);
    println!("      Delete OAuth configuration for a profile");
    println!();
    println!("Examples:");
    println!("  # Interactive prompt (default):");
    println!("  {} config oauth set work --client-id 123.456 --redirect-uri http://127.0.0.1:8765/callback --scopes \"chat:write,users:read\"", prog);
    println!();
    println!("  # Using environment variable:");
    println!("  export SLACKRS_CLIENT_SECRET=xoxp-...");
    println!("  {} config oauth set work --client-id 123.456 --redirect-uri http://127.0.0.1:8765/callback --scopes \"all\"", prog);
    println!();
    println!("  # Using custom environment variable:");
    println!("  {} config oauth set work --client-id 123.456 --redirect-uri http://127.0.0.1:8765/callback --scopes \"all\" --client-secret-env MY_SECRET", prog);
    println!();
    println!("  # Using file:");
    println!("  {} config oauth set work --client-id 123.456 --redirect-uri http://127.0.0.1:8765/callback --scopes \"all\" --client-secret-file ~/.secrets/slack", prog);
    println!();
    println!("  {} config oauth show work", prog);
    println!("  {} config oauth delete work", prog);
}

/// Run config oauth set command
fn run_config_oauth_set(args: &[String]) -> Result<(), String> {
    let mut profile_name: Option<String> = None;
    let mut client_id: Option<String> = None;
    let mut redirect_uri: Option<String> = None;
    let mut scopes: Option<String> = None;
    let mut client_secret_env: Option<String> = None;
    let mut client_secret_file: Option<String> = None;
    let mut client_secret: Option<String> = None;
    let mut confirmed = false;

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
                "--client-secret-env" => {
                    i += 1;
                    if i < args.len() {
                        client_secret_env = Some(args[i].clone());
                    } else {
                        return Err("--client-secret-env requires a value".to_string());
                    }
                }
                "--client-secret-file" => {
                    i += 1;
                    if i < args.len() {
                        client_secret_file = Some(args[i].clone());
                    } else {
                        return Err("--client-secret-file requires a value".to_string());
                    }
                }
                "--client-secret" => {
                    i += 1;
                    if i < args.len() {
                        client_secret = Some(args[i].clone());
                    } else {
                        return Err("--client-secret requires a value".to_string());
                    }
                }
                "--yes" => {
                    confirmed = true;
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

    commands::oauth_set(commands::OAuthSetParams {
        profile_name: profile,
        client_id: client,
        redirect_uri: redirect,
        scopes: scope_str,
        client_secret_env,
        client_secret_file,
        client_secret,
        confirmed,
    })
    .map_err(|e| e.to_string())
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

/// Run config set command
fn run_config_set(args: &[String]) -> Result<(), String> {
    let mut profile_name: Option<String> = None;
    let mut token_type: Option<profile::TokenType> = None;

    let mut i = 0;
    while i < args.len() {
        if args[i].starts_with("--") {
            match args[i].as_str() {
                "--token-type" => {
                    i += 1;
                    if i < args.len() {
                        token_type = Some(
                            args[i]
                                .parse::<profile::TokenType>()
                                .map_err(|e| format!("Invalid token type: {}", e))?,
                        );
                    } else {
                        return Err("--token-type requires a value".to_string());
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
    let ttype = token_type.ok_or_else(|| "--token-type is required".to_string())?;

    commands::set_default_token_type(profile, ttype).map_err(|e| e.to_string())
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
        default_token_type: None,
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
        default_token_type: None,
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
        default_token_type: None,
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
        default_token_type: None,
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
