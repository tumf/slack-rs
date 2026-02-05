//! Conversation command implementations

use crate::api::{ApiClient, ApiError, ApiMethod, ApiResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Filter error types
#[derive(Debug, Error)]
pub enum FilterError {
    #[error("Invalid filter format: {0}")]
    InvalidFormat(String),
    #[error("Invalid boolean value: {0}")]
    InvalidBoolean(String),
}

/// Output format for conversation list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Jsonl,
    Table,
    Tsv,
}

impl OutputFormat {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "json" => Ok(OutputFormat::Json),
            "jsonl" => Ok(OutputFormat::Jsonl),
            "table" => Ok(OutputFormat::Table),
            "tsv" => Ok(OutputFormat::Tsv),
            _ => Err(format!(
                "Invalid format '{}'. Valid values: json, jsonl, table, tsv",
                s
            )),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Jsonl => write!(f, "jsonl"),
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Tsv => write!(f, "tsv"),
        }
    }
}

/// Sort key for conversation list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Created,
    NumMembers,
}

impl SortKey {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "name" => Ok(SortKey::Name),
            "created" => Ok(SortKey::Created),
            "num_members" => Ok(SortKey::NumMembers),
            _ => Err(format!(
                "Invalid sort key '{}'. Valid values: name, created, num_members",
                s
            )),
        }
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "asc" => Ok(SortDirection::Asc),
            "desc" => Ok(SortDirection::Desc),
            _ => Err(format!(
                "Invalid sort direction '{}'. Valid values: asc, desc",
                s
            )),
        }
    }
}

/// Filter type for conversation list
#[derive(Debug, Clone, PartialEq)]
pub enum ConversationFilter {
    /// Filter by name pattern (glob)
    Name(String),
    /// Filter by membership status
    IsMember(bool),
    /// Filter by private/public status
    IsPrivate(bool),
}

impl ConversationFilter {
    /// Parse filter from string format "key:value"
    pub fn parse(s: &str) -> Result<Self, FilterError> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(FilterError::InvalidFormat(format!(
                "Expected format 'key:value', got '{}'",
                s
            )));
        }

        match parts[0] {
            "name" => Ok(ConversationFilter::Name(parts[1].to_string())),
            "is_member" => {
                let value = parts[1].parse::<bool>().map_err(|_| {
                    FilterError::InvalidBoolean(format!(
                        "Expected 'true' or 'false', got '{}'",
                        parts[1]
                    ))
                })?;
                Ok(ConversationFilter::IsMember(value))
            }
            "is_private" => {
                let value = parts[1].parse::<bool>().map_err(|_| {
                    FilterError::InvalidBoolean(format!(
                        "Expected 'true' or 'false', got '{}'",
                        parts[1]
                    ))
                })?;
                Ok(ConversationFilter::IsPrivate(value))
            }
            _ => Err(FilterError::InvalidFormat(format!(
                "Unknown filter key: {}",
                parts[0]
            ))),
        }
    }

    /// Apply filter to conversation JSON value
    pub fn matches(&self, conv: &Value) -> bool {
        match self {
            ConversationFilter::Name(pattern) => {
                if let Some(name) = conv.get("name").and_then(|v| v.as_str()) {
                    glob_match(pattern, name)
                } else {
                    false
                }
            }
            ConversationFilter::IsMember(expected) => {
                if let Some(is_member) = conv.get("is_member").and_then(|v| v.as_bool()) {
                    is_member == *expected
                } else {
                    false
                }
            }
            ConversationFilter::IsPrivate(expected) => {
                if let Some(is_private) = conv.get("is_private").and_then(|v| v.as_bool()) {
                    is_private == *expected
                } else {
                    false
                }
            }
        }
    }
}

/// Simple glob pattern matching (supports * wildcard)
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let pattern_parts: Vec<&str> = pattern.split('*').collect();

    // No wildcard - exact match
    if pattern_parts.len() == 1 {
        return pattern == text;
    }

    // Pattern starts with wildcard
    if pattern.starts_with('*') && pattern_parts.len() == 2 && pattern_parts[1].is_empty() {
        return true; // Pattern is just "*"
    }

    // Pattern ends with wildcard
    if pattern.ends_with('*') && pattern_parts.len() == 2 && pattern_parts[0].is_empty() {
        return true; // Pattern is just "*"
    }

    // General wildcard matching
    let mut text_pos = 0;

    for (i, part) in pattern_parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 && !pattern.starts_with('*') {
            // First part must match at the beginning
            if !text[text_pos..].starts_with(part) {
                return false;
            }
            text_pos += part.len();
        } else if i == pattern_parts.len() - 1 && !pattern.ends_with('*') {
            // Last part must match at the end
            if !text.ends_with(part) {
                return false;
            }
        } else {
            // Middle part - find it
            if let Some(pos) = text[text_pos..].find(part) {
                text_pos += pos + part.len();
            } else {
                return false;
            }
        }
    }

    true
}

/// Apply multiple filters with AND logic to conversations
pub fn apply_filters(response: &mut ApiResponse, filters: &[ConversationFilter]) {
    if filters.is_empty() {
        return;
    }

    if let Some(channels) = response.data.get_mut("channels") {
        if let Some(channels_array) = channels.as_array_mut() {
            channels_array.retain(|conv| filters.iter().all(|filter| filter.matches(conv)));
        }
    }
}

/// Sort conversations by the specified key and direction
pub fn sort_conversations(response: &mut ApiResponse, key: SortKey, direction: SortDirection) {
    if let Some(channels) = response.data.get_mut("channels") {
        if let Some(channels_array) = channels.as_array_mut() {
            channels_array.sort_by(|a, b| {
                let ordering = match key {
                    SortKey::Name => {
                        let a_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let b_name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        a_name.cmp(b_name)
                    }
                    SortKey::Created => {
                        let a_created = a.get("created").and_then(|v| v.as_i64()).unwrap_or(0);
                        let b_created = b.get("created").and_then(|v| v.as_i64()).unwrap_or(0);
                        a_created.cmp(&b_created)
                    }
                    SortKey::NumMembers => {
                        let a_members = a.get("num_members").and_then(|v| v.as_i64()).unwrap_or(0);
                        let b_members = b.get("num_members").and_then(|v| v.as_i64()).unwrap_or(0);
                        a_members.cmp(&b_members)
                    }
                };

                match direction {
                    SortDirection::Asc => ordering,
                    SortDirection::Desc => ordering.reverse(),
                }
            });
        }
    }
}

/// Format response for output
pub fn format_response(response: &ApiResponse, format: OutputFormat) -> Result<String, String> {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&response)
            .map_err(|e| format!("Failed to serialize JSON: {}", e)),
        OutputFormat::Jsonl => {
            if let Some(channels) = response.data.get("channels") {
                if let Some(channels_array) = channels.as_array() {
                    let lines: Vec<String> = channels_array
                        .iter()
                        .filter_map(|conv| serde_json::to_string(conv).ok())
                        .collect();
                    Ok(lines.join("\n"))
                } else {
                    Ok(String::new())
                }
            } else {
                Ok(String::new())
            }
        }
        OutputFormat::Table => format_as_table(response),
        OutputFormat::Tsv => format_as_tsv(response),
    }
}

/// Format response as table
fn format_as_table(response: &ApiResponse) -> Result<String, String> {
    let channels = match response.data.get("channels").and_then(|v| v.as_array()) {
        Some(ch) => ch,
        None => return Ok(String::new()),
    };

    if channels.is_empty() {
        return Ok(String::new());
    }

    // Calculate column widths
    let mut max_id = "ID".len();
    let mut max_name = "NAME".len();
    let max_private = "PRIVATE".len();
    let max_member = "MEMBER".len();
    let mut max_num_members = "NUM_MEMBERS".len();

    for conv in channels {
        if let Some(id) = conv.get("id").and_then(|v| v.as_str()) {
            max_id = max_id.max(id.len());
        }
        if let Some(name) = conv.get("name").and_then(|v| v.as_str()) {
            max_name = max_name.max(name.len());
        }
        if let Some(num) = conv.get("num_members").and_then(|v| v.as_i64()) {
            max_num_members = max_num_members.max(num.to_string().len());
        }
    }

    // Build header
    let mut output = String::new();
    output.push_str(&format!(
        "{:width_id$}  {:width_name$}  {:width_private$}  {:width_member$}  {:width_num$}\n",
        "ID",
        "NAME",
        "PRIVATE",
        "MEMBER",
        "NUM_MEMBERS",
        width_id = max_id,
        width_name = max_name,
        width_private = max_private,
        width_member = max_member,
        width_num = max_num_members,
    ));

    // Build separator
    output.push_str(&format!(
        "{}  {}  {}  {}  {}\n",
        "-".repeat(max_id),
        "-".repeat(max_name),
        "-".repeat(max_private),
        "-".repeat(max_member),
        "-".repeat(max_num_members),
    ));

    // Build rows
    for conv in channels {
        let id = conv.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let name = conv.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let is_private = conv
            .get("is_private")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_member = conv
            .get("is_member")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let num_members = conv.get("num_members").and_then(|v| v.as_i64());

        let num_members_str = num_members.map(|n| n.to_string()).unwrap_or_default();

        output.push_str(&format!(
            "{:width_id$}  {:width_name$}  {:width_private$}  {:width_member$}  {:width_num$}\n",
            id,
            name,
            is_private,
            is_member,
            num_members_str,
            width_id = max_id,
            width_name = max_name,
            width_private = max_private,
            width_member = max_member,
            width_num = max_num_members,
        ));
    }

    Ok(output)
}

/// Format response as TSV
fn format_as_tsv(response: &ApiResponse) -> Result<String, String> {
    let channels = match response.data.get("channels").and_then(|v| v.as_array()) {
        Some(ch) => ch,
        None => return Ok(String::new()),
    };

    if channels.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::new();

    // Header
    output.push_str("id\tname\tis_private\tis_member\tnum_members\n");

    // Rows
    for conv in channels {
        let id = conv.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let name = conv.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let is_private = conv
            .get("is_private")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_member = conv
            .get("is_member")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let num_members = conv.get("num_members").and_then(|v| v.as_i64());

        let num_members_str = num_members.map(|n| n.to_string()).unwrap_or_default();

        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            id, name, is_private, is_member, num_members_str
        ));
    }

    Ok(output)
}

/// Conversation item for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationItem {
    pub id: String,
    pub name: String,
    pub is_private: bool,
}

impl ConversationItem {
    /// Format for display in selection UI
    pub fn display(&self) -> String {
        let privacy = if self.is_private { " [private]" } else { "" };
        format!("#{} ({}){}", self.name, self.id, privacy)
    }
}

/// Extract conversation items from API response
pub fn extract_conversations(response: &ApiResponse) -> Vec<ConversationItem> {
    let mut items = Vec::new();

    if let Some(channels) = response.data.get("channels") {
        if let Some(channels_array) = channels.as_array() {
            for conv in channels_array {
                if let (Some(id), Some(name)) = (
                    conv.get("id").and_then(|v| v.as_str()),
                    conv.get("name").and_then(|v| v.as_str()),
                ) {
                    let is_private = conv
                        .get("is_private")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    items.push(ConversationItem {
                        id: id.to_string(),
                        name: name.to_string(),
                        is_private,
                    });
                }
            }
        }
    }

    items
}

/// Trait for interactive selection UI (allows for stubbing in tests)
pub trait ConversationSelector {
    /// Select a conversation from a list
    fn select(&self, items: &[ConversationItem]) -> Result<String, String>;
}

/// Default implementation using stdin
pub struct StdinSelector;

impl ConversationSelector for StdinSelector {
    fn select(&self, items: &[ConversationItem]) -> Result<String, String> {
        if items.is_empty() {
            return Err("No conversations available".to_string());
        }

        println!("Select a conversation:");
        for (i, item) in items.iter().enumerate() {
            println!("  {}: {}", i + 1, item.display());
        }
        println!("Enter number (or 0 to cancel): ");

        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let mut line = String::new();
        stdin
            .lock()
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;

        let choice: usize = line
            .trim()
            .parse()
            .map_err(|_| "Invalid number".to_string())?;

        if choice == 0 {
            return Err("Selection cancelled".to_string());
        }

        if choice > items.len() {
            return Err("Invalid selection".to_string());
        }

        Ok(items[choice - 1].id.clone())
    }
}

/// List conversations
///
/// # Arguments
/// * `client` - API client
/// * `types` - Optional comma-separated list of conversation types (public_channel, private_channel, mpim, im)
/// * `limit` - Optional number of results to return (default: 100)
///
/// # Returns
/// * `Ok(ApiResponse)` with conversation list
/// * `Err(ApiError)` if the operation fails
pub async fn conv_list(
    client: &ApiClient,
    types: Option<String>,
    limit: Option<u32>,
) -> Result<ApiResponse, ApiError> {
    let mut params = HashMap::new();

    if let Some(types) = types {
        params.insert("types".to_string(), json!(types));
    }

    if let Some(limit) = limit {
        params.insert("limit".to_string(), json!(limit));
    }

    client
        .call_method(ApiMethod::ConversationsList, params)
        .await
}

/// Get conversation history
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID
/// * `limit` - Optional number of messages to return (default: 100)
/// * `oldest` - Optional oldest timestamp to include
/// * `latest` - Optional latest timestamp to include
///
/// # Returns
/// * `Ok(ApiResponse)` with conversation history
/// * `Err(ApiError)` if the operation fails
pub async fn conv_history(
    client: &ApiClient,
    channel: String,
    limit: Option<u32>,
    oldest: Option<String>,
    latest: Option<String>,
) -> Result<ApiResponse, ApiError> {
    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));

    if let Some(limit) = limit {
        params.insert("limit".to_string(), json!(limit));
    }

    if let Some(oldest) = oldest {
        params.insert("oldest".to_string(), json!(oldest));
    }

    if let Some(latest) = latest {
        params.insert("latest".to_string(), json!(latest));
    }

    client
        .call_method(ApiMethod::ConversationsHistory, params)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_conv_list_basic() {
        let client = ApiClient::with_token("test_token".to_string());
        let result = conv_list(&client, None, None).await;
        // Result will fail because there's no mock server, but that's expected
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_conv_history_basic() {
        let client = ApiClient::with_token("test_token".to_string());
        let result = conv_history(&client, "C123456".to_string(), None, None, None).await;
        // Result will fail because there's no mock server, but that's expected
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_parse_name() {
        let filter = ConversationFilter::parse("name:test*").unwrap();
        assert_eq!(filter, ConversationFilter::Name("test*".to_string()));
    }

    #[test]
    fn test_filter_parse_is_member() {
        let filter = ConversationFilter::parse("is_member:true").unwrap();
        assert_eq!(filter, ConversationFilter::IsMember(true));

        let filter = ConversationFilter::parse("is_member:false").unwrap();
        assert_eq!(filter, ConversationFilter::IsMember(false));
    }

    #[test]
    fn test_filter_parse_is_private() {
        let filter = ConversationFilter::parse("is_private:true").unwrap();
        assert_eq!(filter, ConversationFilter::IsPrivate(true));

        let filter = ConversationFilter::parse("is_private:false").unwrap();
        assert_eq!(filter, ConversationFilter::IsPrivate(false));
    }

    #[test]
    fn test_filter_parse_invalid_format() {
        let result = ConversationFilter::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_parse_invalid_key() {
        let result = ConversationFilter::parse("unknown:value");
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_parse_invalid_boolean() {
        let result = ConversationFilter::parse("is_member:maybe");
        assert!(result.is_err());
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("test", "test"));
        assert!(!glob_match("test", "other"));
    }

    #[test]
    fn test_glob_match_wildcard() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("test*", "test"));
        assert!(glob_match("test*", "test123"));
        assert!(!glob_match("test*", "other"));

        assert!(glob_match("*test", "test"));
        assert!(glob_match("*test", "mytest"));
        assert!(!glob_match("*test", "testing"));

        assert!(glob_match("*test*", "test"));
        assert!(glob_match("*test*", "mytest123"));
        assert!(!glob_match("*test*", "other"));
    }

    #[test]
    fn test_filter_matches_name() {
        let filter = ConversationFilter::Name("general".to_string());
        let conv = json!({"name": "general", "id": "C123"});
        assert!(filter.matches(&conv));

        let conv = json!({"name": "random", "id": "C124"});
        assert!(!filter.matches(&conv));
    }

    #[test]
    fn test_filter_matches_name_glob() {
        let filter = ConversationFilter::Name("test*".to_string());
        let conv = json!({"name": "test-channel", "id": "C123"});
        assert!(filter.matches(&conv));

        let conv = json!({"name": "other", "id": "C124"});
        assert!(!filter.matches(&conv));
    }

    #[test]
    fn test_filter_matches_is_member() {
        let filter = ConversationFilter::IsMember(true);
        let conv = json!({"name": "general", "is_member": true});
        assert!(filter.matches(&conv));

        let conv = json!({"name": "general", "is_member": false});
        assert!(!filter.matches(&conv));
    }

    #[test]
    fn test_filter_matches_is_private() {
        let filter = ConversationFilter::IsPrivate(true);
        let conv = json!({"name": "private", "is_private": true});
        assert!(filter.matches(&conv));

        let conv = json!({"name": "public", "is_private": false});
        assert!(!filter.matches(&conv));
    }

    #[test]
    fn test_apply_filters_and_condition() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "test-public", "is_member": true, "is_private": false},
                    {"id": "C2", "name": "test-private", "is_member": true, "is_private": true},
                    {"id": "C3", "name": "other", "is_member": true, "is_private": false},
                    {"id": "C4", "name": "test-nomember", "is_member": false, "is_private": false},
                ]),
            )]),
            error: None,
        };

        let filters = vec![
            ConversationFilter::Name("test*".to_string()),
            ConversationFilter::IsMember(true),
        ];

        apply_filters(&mut response, &filters);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(channels.len(), 2); // C1 and C2
        assert_eq!(channels[0].get("id").unwrap().as_str().unwrap(), "C1");
        assert_eq!(channels[1].get("id").unwrap().as_str().unwrap(), "C2");
    }

    #[test]
    fn test_extract_conversations() {
        let response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "general", "is_private": false},
                    {"id": "C2", "name": "private", "is_private": true},
                ]),
            )]),
            error: None,
        };

        let items = extract_conversations(&response);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "C1");
        assert_eq!(items[0].name, "general");
        assert!(!items[0].is_private);
        assert_eq!(items[1].id, "C2");
        assert_eq!(items[1].name, "private");
        assert!(items[1].is_private);
    }

    #[test]
    fn test_conversation_item_display() {
        let item = ConversationItem {
            id: "C123".to_string(),
            name: "general".to_string(),
            is_private: false,
        };
        assert_eq!(item.display(), "#general (C123)");

        let item = ConversationItem {
            id: "C456".to_string(),
            name: "secret".to_string(),
            is_private: true,
        };
        assert_eq!(item.display(), "#secret (C456) [private]");
    }

    // Mock selector for testing
    pub struct MockSelector {
        pub selected_index: usize,
    }

    impl ConversationSelector for MockSelector {
        fn select(&self, items: &[ConversationItem]) -> Result<String, String> {
            if items.is_empty() {
                return Err("No conversations available".to_string());
            }
            if self.selected_index >= items.len() {
                return Err("Invalid selection".to_string());
            }
            Ok(items[self.selected_index].id.clone())
        }
    }

    #[test]
    fn test_mock_selector() {
        let items = vec![
            ConversationItem {
                id: "C1".to_string(),
                name: "general".to_string(),
                is_private: false,
            },
            ConversationItem {
                id: "C2".to_string(),
                name: "random".to_string(),
                is_private: false,
            },
        ];

        let selector = MockSelector { selected_index: 0 };
        assert_eq!(selector.select(&items).unwrap(), "C1");

        let selector = MockSelector { selected_index: 1 };
        assert_eq!(selector.select(&items).unwrap(), "C2");
    }

    #[test]
    fn test_output_format_parse() {
        assert_eq!(OutputFormat::parse("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::parse("jsonl").unwrap(), OutputFormat::Jsonl);
        assert_eq!(OutputFormat::parse("table").unwrap(), OutputFormat::Table);
        assert_eq!(OutputFormat::parse("tsv").unwrap(), OutputFormat::Tsv);
        assert!(OutputFormat::parse("invalid").is_err());
    }

    #[test]
    fn test_sort_key_parse() {
        assert_eq!(SortKey::parse("name").unwrap(), SortKey::Name);
        assert_eq!(SortKey::parse("created").unwrap(), SortKey::Created);
        assert_eq!(SortKey::parse("num_members").unwrap(), SortKey::NumMembers);
        assert!(SortKey::parse("invalid").is_err());
    }

    #[test]
    fn test_sort_direction_parse() {
        assert_eq!(SortDirection::parse("asc").unwrap(), SortDirection::Asc);
        assert_eq!(SortDirection::parse("desc").unwrap(), SortDirection::Desc);
        assert!(SortDirection::parse("invalid").is_err());
    }

    #[test]
    fn test_sort_conversations_by_name() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "zebra"},
                    {"id": "C2", "name": "alpha"},
                    {"id": "C3", "name": "beta"},
                ]),
            )]),
            error: None,
        };

        sort_conversations(&mut response, SortKey::Name, SortDirection::Asc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(channels[0].get("name").unwrap().as_str().unwrap(), "alpha");
        assert_eq!(channels[1].get("name").unwrap().as_str().unwrap(), "beta");
        assert_eq!(channels[2].get("name").unwrap().as_str().unwrap(), "zebra");
    }

    #[test]
    fn test_sort_conversations_by_name_desc() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "alpha"},
                    {"id": "C2", "name": "zebra"},
                    {"id": "C3", "name": "beta"},
                ]),
            )]),
            error: None,
        };

        sort_conversations(&mut response, SortKey::Name, SortDirection::Desc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(channels[0].get("name").unwrap().as_str().unwrap(), "zebra");
        assert_eq!(channels[1].get("name").unwrap().as_str().unwrap(), "beta");
        assert_eq!(channels[2].get("name").unwrap().as_str().unwrap(), "alpha");
    }

    #[test]
    fn test_sort_conversations_by_created() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "newest", "created": 300},
                    {"id": "C2", "name": "oldest", "created": 100},
                    {"id": "C3", "name": "middle", "created": 200},
                ]),
            )]),
            error: None,
        };

        sort_conversations(&mut response, SortKey::Created, SortDirection::Asc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(channels[0].get("created").unwrap().as_i64().unwrap(), 100);
        assert_eq!(channels[1].get("created").unwrap().as_i64().unwrap(), 200);
        assert_eq!(channels[2].get("created").unwrap().as_i64().unwrap(), 300);
    }

    #[test]
    fn test_sort_conversations_by_num_members() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "large", "num_members": 100},
                    {"id": "C2", "name": "small", "num_members": 10},
                    {"id": "C3", "name": "medium", "num_members": 50},
                ]),
            )]),
            error: None,
        };

        sort_conversations(&mut response, SortKey::NumMembers, SortDirection::Asc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(
            channels[0].get("num_members").unwrap().as_i64().unwrap(),
            10
        );
        assert_eq!(
            channels[1].get("num_members").unwrap().as_i64().unwrap(),
            50
        );
        assert_eq!(
            channels[2].get("num_members").unwrap().as_i64().unwrap(),
            100
        );
    }

    #[test]
    fn test_sort_conversations_missing_fields() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "has_members", "num_members": 50},
                    {"id": "C2", "name": "no_members"},
                    {"id": "C3", "name": "also_has", "num_members": 10},
                ]),
            )]),
            error: None,
        };

        sort_conversations(&mut response, SortKey::NumMembers, SortDirection::Asc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        // Missing field treated as 0, so it should be first
        assert_eq!(
            channels[0].get("name").unwrap().as_str().unwrap(),
            "no_members"
        );
        assert_eq!(
            channels[1].get("num_members").unwrap().as_i64().unwrap(),
            10
        );
        assert_eq!(
            channels[2].get("num_members").unwrap().as_i64().unwrap(),
            50
        );
    }

    #[test]
    fn test_format_response_jsonl() {
        let response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "general"},
                    {"id": "C2", "name": "random"},
                ]),
            )]),
            error: None,
        };

        let output = format_response(&response, OutputFormat::Jsonl).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"id\":\"C1\""));
        assert!(lines[1].contains("\"id\":\"C2\""));
    }

    #[test]
    fn test_format_response_tsv() {
        let response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "general", "is_private": false, "is_member": true, "num_members": 42},
                    {"id": "C2", "name": "private", "is_private": true, "is_member": false},
                ]),
            )]),
            error: None,
        };

        let output = format_response(&response, OutputFormat::Tsv).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 rows
        assert_eq!(lines[0], "id\tname\tis_private\tis_member\tnum_members");
        assert_eq!(lines[1], "C1\tgeneral\tfalse\ttrue\t42");
        assert_eq!(lines[2], "C2\tprivate\ttrue\tfalse\t"); // num_members missing -> empty
    }

    #[test]
    fn test_format_response_table() {
        let response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "general", "is_private": false, "is_member": true, "num_members": 42},
                ]),
            )]),
            error: None,
        };

        let output = format_response(&response, OutputFormat::Table).unwrap();
        assert!(output.contains("ID"));
        assert!(output.contains("NAME"));
        assert!(output.contains("PRIVATE"));
        assert!(output.contains("MEMBER"));
        assert!(output.contains("NUM_MEMBERS"));
        assert!(output.contains("C1"));
        assert!(output.contains("general"));
        assert!(output.contains("42"));
    }

    #[test]
    fn test_filter_then_sort() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "test-zebra", "is_member": true},
                    {"id": "C2", "name": "test-alpha", "is_member": true},
                    {"id": "C3", "name": "other", "is_member": true},
                    {"id": "C4", "name": "test-beta", "is_member": false},
                ]),
            )]),
            error: None,
        };

        // Apply filter (name:test*, is_member:true)
        let filters = vec![
            ConversationFilter::Name("test*".to_string()),
            ConversationFilter::IsMember(true),
        ];
        apply_filters(&mut response, &filters);

        // Apply sort (name, asc)
        sort_conversations(&mut response, SortKey::Name, SortDirection::Asc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(channels.len(), 2); // C1 and C2
        assert_eq!(
            channels[0].get("name").unwrap().as_str().unwrap(),
            "test-alpha"
        );
        assert_eq!(
            channels[1].get("name").unwrap().as_str().unwrap(),
            "test-zebra"
        );
    }
}
