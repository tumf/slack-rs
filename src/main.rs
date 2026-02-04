mod api;
mod auth;
mod cli;
mod commands;
mod oauth;
mod profile;

use api::{execute_api_call, ApiCallArgs, ApiCallContext, ApiClient};
use cli::*;
use profile::{
    default_config_path, load_config, make_token_key, resolve_profile, resolve_profile_full,
    save_config, InMemoryTokenStore, KeyringTokenStore, Profile, ProfilesConfig, TokenStore,
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
                    // Load OAuth config from environment variables
                    let client_id = match std::env::var("SLACKRS_CLIENT_ID") {
                        Ok(val) => val,
                        Err(_) => {
                            eprintln!("Error: SLACKRS_CLIENT_ID environment variable is required");
                            eprintln!("Please set it with your Slack OAuth client ID");
                            std::process::exit(1);
                        }
                    };

                    let client_secret = match std::env::var("SLACKRS_CLIENT_SECRET") {
                        Ok(val) => val,
                        Err(_) => {
                            eprintln!(
                                "Error: SLACKRS_CLIENT_SECRET environment variable is required"
                            );
                            eprintln!("Please set it with your Slack OAuth client secret");
                            std::process::exit(1);
                        }
                    };

                    let redirect_uri = std::env::var("SLACKRS_REDIRECT_URI")
                        .unwrap_or_else(|_| "http://127.0.0.1:3000/callback".to_string());

                    let scopes = std::env::var("SLACKRS_SCOPES")
                        .unwrap_or_else(|_| "chat:write,users:read".to_string())
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();

                    let config = oauth::OAuthConfig {
                        client_id,
                        client_secret,
                        redirect_uri,
                        scopes,
                    };

                    let profile_name = args.get(3).cloned();
                    let base_url = std::env::var("SLACK_OAUTH_BASE_URL").ok();

                    match auth::login(config, profile_name, base_url).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Login failed: {}", e);
                            std::process::exit(1);
                        }
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

            // Demonstrate profile storage integration
            demonstrate_profile_storage();

            // Demonstrate token storage integration
            demonstrate_token_storage();

            // Demonstrate profile persistence (save and reload)
            demonstrate_profile_persistence();

            // Demonstrate keyring token storage
            demonstrate_keyring_token_storage();
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
    println!("    search <query>                   Search messages");
    println!("    conv list                        List conversations");
    println!("    conv history <channel>           Get conversation history");
    println!("    users info <user_id>             Get user information");
    println!("    msg post <channel> <text>        Post a message");
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
    println!("  search <query>                 - Search messages (supports --count, --page, --sort, --sort_dir)");
    println!("  conv list                      - List conversations");
    println!("  conv history <channel>         - Get conversation history");
    println!("  users info <user_id>           - Get user information");
    println!("  msg post <channel> <text>      - Post a message (requires --allow-write)");
    println!("  msg update <channel> <ts> <text> - Update a message (requires --allow-write)");
    println!("  msg delete <channel> <ts>      - Delete a message (requires --allow-write)");
    println!("  react add <channel> <ts> <emoji> - Add a reaction (requires --allow-write)");
    println!("  react remove <channel> <ts> <emoji> - Remove a reaction (requires --allow-write)");
    println!("  file upload <path>             - Upload a file using external upload method (requires --allow-write)");
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
    println!("  auth login [profile_name]           - Authenticate with Slack");
    println!("  auth status [profile_name]          - Show profile status");
    println!("  auth list                           - List all profiles");
    println!("  auth rename <old> <new>             - Rename a profile");
    println!("  auth logout [profile_name]          - Remove authentication");
    println!("  auth export [options]               - Export profiles to encrypted file");
    println!("  auth import [options]               - Import profiles from encrypted file");
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

    let token_store = KeyringTokenStore::default_service();
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

    let token_store = KeyringTokenStore::default_service();
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
    // Try keyring first, fall back to environment variable
    let token = {
        let keyring_store = KeyringTokenStore::default_service();
        match keyring_store.get(&token_key) {
            Ok(t) => t,
            Err(_) => {
                // If keyring fails, check if there's a token in environment
                if let Ok(env_token) = std::env::var("SLACK_TOKEN") {
                    env_token
                } else {
                    return Err(format!(
                        "No token found for profile '{}' ({}:{}). Set SLACK_TOKEN environment variable or store token in keyring.",
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

    // Note about KeyringTokenStore
    println!("\nNote: KeyringTokenStore is available for production use:");
    println!("  let store = KeyringTokenStore::default_service();");
    println!("  // Uses OS keyring with service='slackcli'");

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
    };

    let profile2 = Profile {
        team_id: "T789GHI".to_string(),
        user_id: "U012JKL".to_string(),
        team_name: Some("Another Team".to_string()),
        user_name: Some("Another User".to_string()),
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
    };
    match config.set_or_update("personal".to_string(), updated_profile2) {
        Ok(_) => println!("Updated 'personal' profile using set_or_update()"),
        Err(e) => println!("Failed to update profile: {}", e),
    }

    // Save config to temp location for demonstration
    if let Ok(_config_path) = default_config_path() {
        // Create a test path in a temp directory
        let temp_dir = std::env::temp_dir();
        let test_config_path = temp_dir.join("slackcli_test_profiles.json");

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
fn demonstrate_keyring_token_storage() {
    println!("=== Keyring Token Storage Demo ===");

    // Create KeyringTokenStore with default service name
    let keyring_store = KeyringTokenStore::default_service();
    println!("Created KeyringTokenStore with service='slackcli'");

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
