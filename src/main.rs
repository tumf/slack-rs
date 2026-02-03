mod api;
mod profile;

use api::{execute_api_call, ApiCallArgs, ApiCallContext, ApiClient};
use profile::{
    default_config_path, load_config, make_token_key, resolve_profile, resolve_profile_full,
    save_config, InMemoryTokenStore, KeyringTokenStore, Profile, ProfilesConfig, TokenStore,
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check for CLI commands
    if args.len() > 1 {
        match args[1].as_str() {
            "api" if args.len() > 2 && args[2] == "call" => {
                // Run api call command
                let api_args: Vec<String> = args[3..].to_vec();
                if let Err(e) = run_api_call(api_args).await {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            _ => {}
        }
    }

    // Default behavior: run demonstrations
    println!("Slack CLI - Profile storage foundation established");
    println!();

    // Demonstrate profile storage integration
    demonstrate_profile_storage();

    // Demonstrate token storage integration
    demonstrate_token_storage();

    // Demonstrate profile persistence (save and reload)
    demonstrate_profile_persistence();

    // Demonstrate keyring token storage
    demonstrate_keyring_token_storage();

    println!("\nRun with --help to see available commands");
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
