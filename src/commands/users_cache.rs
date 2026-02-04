//! Users cache for mention resolution
//!
//! Provides caching for user information to enable mention resolution
//! without repeated API calls. Cache is stored per workspace with TTL.

use crate::api::{ApiClient, ApiError};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Default cache TTL in seconds (24 hours)
const DEFAULT_TTL_SECONDS: u64 = 86400;

/// Cached user information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedUser {
    pub id: String,
    pub name: String,
    pub real_name: Option<String>,
    pub display_name: Option<String>,
    pub deleted: bool,
    pub is_bot: bool,
}

/// Workspace-specific user cache
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceCache {
    pub team_id: String,
    pub updated_at: u64,
    pub users: HashMap<String, CachedUser>,
}

/// Users cache file containing multiple workspace caches
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsersCacheFile {
    pub caches: HashMap<String, WorkspaceCache>,
}

impl UsersCacheFile {
    /// Create a new empty cache file
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
        }
    }

    /// Get the default cache file path
    pub fn default_path() -> Result<PathBuf, String> {
        directories::ProjectDirs::from("", "", "slack-rs")
            .map(|dirs| dirs.config_dir().join("users_cache.json"))
            .ok_or_else(|| "Could not determine config directory".to_string())
    }

    /// Load cache from file
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read cache file: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse cache file: {}", e))
    }

    /// Save cache to file
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize cache: {}", e))?;
        fs::write(path, content).map_err(|e| format!("Failed to write cache file: {}", e))
    }

    /// Get workspace cache
    pub fn get_workspace(&self, team_id: &str) -> Option<&WorkspaceCache> {
        self.caches.get(team_id)
    }

    /// Set workspace cache
    pub fn set_workspace(&mut self, cache: WorkspaceCache) {
        self.caches.insert(cache.team_id.clone(), cache);
    }

    /// Check if workspace cache is expired
    pub fn is_expired(&self, team_id: &str, ttl_seconds: u64) -> bool {
        match self.get_workspace(team_id) {
            Some(cache) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                now - cache.updated_at > ttl_seconds
            }
            None => true, // No cache means expired
        }
    }
}

impl Default for UsersCacheFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Format option for mention resolution
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MentionFormat {
    DisplayName,
    RealName,
    Username,
}

impl std::str::FromStr for MentionFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "display_name" => Ok(Self::DisplayName),
            "real_name" => Ok(Self::RealName),
            "username" => Ok(Self::Username),
            _ => Err(format!("Invalid format: {}", s)),
        }
    }
}

/// Fetch all users from Slack API with pagination
///
/// # Arguments
/// * `client` - API client with authentication
/// * `team_id` - Team ID for the workspace
///
/// # Returns
/// * `Ok(WorkspaceCache)` with all users
/// * `Err(ApiError)` if the operation fails
pub async fn fetch_all_users(
    client: &ApiClient,
    team_id: String,
) -> Result<WorkspaceCache, ApiError> {
    let mut all_users = HashMap::new();
    let mut cursor: Option<String> = None;
    let limit = 200;

    loop {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), serde_json::json!(limit));
        if let Some(c) = &cursor {
            params.insert("cursor".to_string(), serde_json::json!(c));
        }

        let response = client
            .call_method(crate::api::ApiMethod::UsersList, params)
            .await?;

        // Extract users from response
        if let Some(members) = response.data.get("members").and_then(|v| v.as_array()) {
            for member in members {
                if let Some(user) = parse_user_from_json(member) {
                    all_users.insert(user.id.clone(), user);
                }
            }
        }

        // Check for next cursor
        cursor = response
            .data
            .get("response_metadata")
            .and_then(|v| v.get("next_cursor"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        if cursor.is_none() {
            break;
        }
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(WorkspaceCache {
        team_id,
        updated_at: now,
        users: all_users,
    })
}

/// Parse user from JSON value
fn parse_user_from_json(value: &serde_json::Value) -> Option<CachedUser> {
    let id = value.get("id")?.as_str()?.to_string();
    let name = value.get("name")?.as_str()?.to_string();

    let profile = value.get("profile");
    let display_name = profile
        .and_then(|p| p.get("display_name"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let real_name = profile
        .and_then(|p| p.get("real_name"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let deleted = value
        .get("deleted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let is_bot = value
        .get("is_bot")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Some(CachedUser {
        id,
        name,
        real_name,
        display_name,
        deleted,
        is_bot,
    })
}

/// Resolve mentions in text using cache
///
/// # Arguments
/// * `text` - Input text containing mentions
/// * `cache` - Workspace cache with user information
/// * `format` - Format to use for resolved mentions
///
/// # Returns
/// Text with mentions resolved to user names
pub fn resolve_mentions(text: &str, cache: &WorkspaceCache, format: MentionFormat) -> String {
    let mention_regex = Regex::new(r"<@(U[A-Z0-9]+)(?:\|[^>]+)?>").unwrap();

    mention_regex
        .replace_all(text, |caps: &regex::Captures| {
            let user_id = &caps[1];
            match cache.users.get(user_id) {
                Some(user) => {
                    let name = match format {
                        MentionFormat::DisplayName => user
                            .display_name
                            .as_deref()
                            .or(Some(&user.name))
                            .unwrap_or(&user.name),
                        MentionFormat::RealName => user.real_name.as_deref().unwrap_or(&user.name),
                        MentionFormat::Username => &user.name,
                    };

                    format!("@{}", name)
                }
                None => caps[0].to_string(), // Keep original if not found
            }
        })
        .to_string()
}

/// Update users cache for a workspace
///
/// # Arguments
/// * `client` - API client
/// * `team_id` - Team ID
/// * `force` - Force update even if cache is not expired
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if the operation fails
pub async fn update_cache(client: &ApiClient, team_id: String, force: bool) -> Result<(), String> {
    let cache_path = UsersCacheFile::default_path()?;
    let mut cache_file = UsersCacheFile::load(&cache_path)?;

    // Check if update is needed
    if !force && !cache_file.is_expired(&team_id, DEFAULT_TTL_SECONDS) {
        return Err("Cache is still valid. Use --force to update anyway.".to_string());
    }

    // Fetch users
    let workspace_cache = fetch_all_users(client, team_id)
        .await
        .map_err(|e| format!("Failed to fetch users: {}", e))?;

    // Update cache
    cache_file.set_workspace(workspace_cache);
    cache_file.save(&cache_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn test_cache_file_new() {
        let cache = UsersCacheFile::new();
        assert!(cache.caches.is_empty());
    }

    #[test]
    fn test_cache_file_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("users_cache.json");

        let mut cache_file = UsersCacheFile::new();
        let workspace = WorkspaceCache {
            team_id: "T123".to_string(),
            updated_at: 1700000000,
            users: HashMap::new(),
        };
        cache_file.set_workspace(workspace);

        cache_file.save(&cache_path).unwrap();
        assert!(cache_path.exists());

        let loaded = UsersCacheFile::load(&cache_path).unwrap();
        assert_eq!(cache_file, loaded);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache_file = UsersCacheFile::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Recent cache should not be expired
        let workspace = WorkspaceCache {
            team_id: "T123".to_string(),
            updated_at: now - 1000, // 1000 seconds ago
            users: HashMap::new(),
        };
        cache_file.set_workspace(workspace);

        assert!(!cache_file.is_expired("T123", 86400)); // 24 hours TTL

        // Old cache should be expired
        let old_workspace = WorkspaceCache {
            team_id: "T456".to_string(),
            updated_at: now - 100000, // > 24 hours ago
            users: HashMap::new(),
        };
        cache_file.set_workspace(old_workspace);

        assert!(cache_file.is_expired("T456", 86400));

        // Non-existent cache should be expired
        assert!(cache_file.is_expired("T999", 86400));
    }

    #[test]
    fn test_mention_resolution() {
        let mut users = HashMap::new();
        users.insert(
            "U123".to_string(),
            CachedUser {
                id: "U123".to_string(),
                name: "john".to_string(),
                real_name: Some("John Doe".to_string()),
                display_name: Some("johnd".to_string()),
                deleted: false,
                is_bot: false,
            },
        );
        users.insert(
            "U456".to_string(),
            CachedUser {
                id: "U456".to_string(),
                name: "jane".to_string(),
                real_name: Some("Jane Smith".to_string()),
                display_name: None,
                deleted: true,
                is_bot: false,
            },
        );

        let cache = WorkspaceCache {
            team_id: "T123".to_string(),
            updated_at: 1700000000,
            users,
        };

        // Test display_name format
        let text = "Hello <@U123> and <@U456>!";
        let result = resolve_mentions(text, &cache, MentionFormat::DisplayName);
        assert_eq!(result, "Hello @johnd and @jane!");

        // Test real_name format
        let result = resolve_mentions(text, &cache, MentionFormat::RealName);
        assert_eq!(result, "Hello @John Doe and @Jane Smith!");

        // Test username format
        let result = resolve_mentions(text, &cache, MentionFormat::Username);
        assert_eq!(result, "Hello @john and @jane!");

        // Test unknown user
        let text_unknown = "Hello <@U999>!";
        let result = resolve_mentions(text_unknown, &cache, MentionFormat::DisplayName);
        assert_eq!(result, "Hello <@U999>!");

        // Test mention with pipe notation
        let text_pipe = "Hello <@U123|john>!";
        let result = resolve_mentions(text_pipe, &cache, MentionFormat::DisplayName);
        assert_eq!(result, "Hello @johnd!");
    }

    #[test]
    fn test_parse_user_from_json() {
        let json = serde_json::json!({
            "id": "U123",
            "name": "john",
            "profile": {
                "display_name": "johnd",
                "real_name": "John Doe"
            },
            "deleted": false,
            "is_bot": false
        });

        let user = parse_user_from_json(&json).unwrap();
        assert_eq!(user.id, "U123");
        assert_eq!(user.name, "john");
        assert_eq!(user.display_name, Some("johnd".to_string()));
        assert_eq!(user.real_name, Some("John Doe".to_string()));
        assert!(!user.deleted);
        assert!(!user.is_bot);
    }

    #[test]
    fn test_mention_format_from_str() {
        use std::str::FromStr;
        assert_eq!(
            MentionFormat::from_str("display_name"),
            Ok(MentionFormat::DisplayName)
        );
        assert_eq!(
            MentionFormat::from_str("real_name"),
            Ok(MentionFormat::RealName)
        );
        assert_eq!(
            MentionFormat::from_str("username"),
            Ok(MentionFormat::Username)
        );
        assert!(MentionFormat::from_str("invalid").is_err());
    }

    #[tokio::test]
    async fn test_fetch_all_users_with_pagination() {
        let mock_server = MockServer::start().await;

        // First page response
        let first_response = serde_json::json!({
            "ok": true,
            "members": [
                {
                    "id": "U001",
                    "name": "user1",
                    "profile": {
                        "display_name": "User One",
                        "real_name": "User One"
                    },
                    "deleted": false,
                    "is_bot": false
                },
                {
                    "id": "U002",
                    "name": "user2",
                    "profile": {
                        "display_name": "User Two",
                        "real_name": "User Two"
                    },
                    "deleted": false,
                    "is_bot": false
                }
            ],
            "response_metadata": {
                "next_cursor": "cursor123"
            }
        });

        // Second page response
        let second_response = serde_json::json!({
            "ok": true,
            "members": [
                {
                    "id": "U003",
                    "name": "user3",
                    "profile": {
                        "display_name": "User Three",
                        "real_name": "User Three"
                    },
                    "deleted": false,
                    "is_bot": false
                }
            ],
            "response_metadata": {
                "next_cursor": ""
            }
        });

        // Use up() to respond to first request
        Mock::given(method("POST"))
            .and(path("/users.list"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(&first_response)
                    .append_header("content-type", "application/json"),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // Use up() to respond to second request
        Mock::given(method("POST"))
            .and(path("/users.list"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(&second_response)
                    .append_header("content-type", "application/json"),
            )
            .mount(&mock_server)
            .await;

        let client =
            crate::api::ApiClient::new_with_base_url("test-token".to_string(), mock_server.uri());

        let result = fetch_all_users(&client, "T123".to_string()).await;
        assert!(result.is_ok());

        let cache = result.unwrap();
        assert_eq!(cache.team_id, "T123");
        assert_eq!(cache.users.len(), 3);
        assert!(cache.users.contains_key("U001"));
        assert!(cache.users.contains_key("U002"));
        assert!(cache.users.contains_key("U003"));
    }
}
