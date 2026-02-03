mod profile;

use profile::{
    default_config_path, load_config, make_token_key, resolve_profile, InMemoryTokenStore, Profile,
    ProfilesConfig, TokenStore,
};

// KeyringTokenStore is mentioned in comments/docs but not directly used in this demo
#[allow(unused_imports)]
use profile::KeyringTokenStore;

fn main() {
    println!("Slack CLI - Profile storage foundation established");
    println!();

    // Demonstrate profile storage integration
    demonstrate_profile_storage();

    // Demonstrate token storage integration
    demonstrate_token_storage();
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
