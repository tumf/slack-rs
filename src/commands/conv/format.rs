//! Output formatting functionality for conversations

use crate::api::{ApiClient, ApiError, ApiResponse};
use crate::commands::users_cache::{CachedUser, WorkspaceCache};
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

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

/// Recursively collect user IDs from `<@Uxxxx>` mentions in all string values
/// within a Value (including nested blocks, arrays, and objects).
fn collect_ids_from_value(value: &Value, ids: &mut BTreeSet<String>, re: &Regex) {
    match value {
        Value::String(s) => {
            for cap in re.captures_iter(s) {
                ids.insert(cap[1].to_string());
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_ids_from_value(v, ids, re);
            }
        }
        Value::Object(obj) => {
            for v in obj.values() {
                collect_ids_from_value(v, ids, re);
            }
        }
        _ => {}
    }
}

/// Collect all user IDs referenced by a Slack message.
///
/// Gathers user IDs from:
/// - The `user` field (the message poster)
/// - `<@Uxxxx>` mentions in `text`
/// - `<@Uxxxx>` mentions anywhere within `blocks`
/// - User IDs in `reactions[].users[]`
///
/// Results are deduplicated and sorted lexicographically.
pub fn collect_message_user_ids(message: &Value) -> Vec<String> {
    let mention_re = Regex::new(r"<@(U[A-Z0-9]+)(?:\|[^>]+)?>").unwrap();
    let mut ids = BTreeSet::new();

    // Poster
    if let Some(user) = message.get("user").and_then(|v| v.as_str()) {
        ids.insert(user.to_string());
    }

    // Mentions in text
    if let Some(text) = message.get("text").and_then(|v| v.as_str()) {
        for cap in mention_re.captures_iter(text) {
            ids.insert(cap[1].to_string());
        }
    }

    // Mentions in blocks (recursive)
    if let Some(blocks) = message.get("blocks") {
        collect_ids_from_value(blocks, &mut ids, &mention_re);
    }

    // Reaction actors from reactions[].users[]
    if let Some(reactions) = message.get("reactions").and_then(|r| r.as_array()) {
        for reaction in reactions {
            if let Some(users) = reaction.get("users").and_then(|u| u.as_array()) {
                for user in users {
                    if let Some(user_id) = user.as_str() {
                        ids.insert(user_id.to_string());
                    }
                }
            }
        }
    }

    ids.into_iter().collect()
}

/// Enrich a single Slack message with resolved user information.
///
/// Adds a `users` array to the message object containing `id`, `name`,
/// `display_name`, and `real_name` for every user ID found in the message.
/// Names are populated from the workspace cache when available; uncached
/// users appear with only the `id` field (graceful degradation).
pub fn enrich_message_with_users(message: &mut Value, cache: Option<&WorkspaceCache>) {
    let ids = collect_message_user_ids(message);

    let users: Vec<Value> = ids
        .iter()
        .map(|id| {
            let mut entry = serde_json::Map::new();
            entry.insert("id".to_string(), serde_json::Value::String(id.clone()));

            // Populate name fields when cache has this user
            if let Some(cache) = cache {
                if let Some(cached) = cache.users.get(id.as_str()) {
                    entry.insert(
                        "name".to_string(),
                        serde_json::Value::String(cached.name.clone()),
                    );
                    entry.insert(
                        "display_name".to_string(),
                        match &cached.display_name {
                            Some(dn) => serde_json::Value::String(dn.clone()),
                            None => serde_json::Value::Null,
                        },
                    );
                    entry.insert(
                        "real_name".to_string(),
                        match &cached.real_name {
                            Some(rn) => serde_json::Value::String(rn.clone()),
                            None => serde_json::Value::Null,
                        },
                    );
                }
            }

            serde_json::Value::Object(entry)
        })
        .collect();

    if let Value::Object(ref mut map) = message {
        map.insert("users".to_string(), Value::Array(users));
    }
}

/// Collect all user IDs referenced across a Slack thread response.
///
/// Gathers IDs from each message's author, text mentions, block mentions, and
/// `reactions[].users[]`. Results are deduplicated and sorted lexicographically
/// for deterministic resolution and output.
pub fn collect_thread_user_ids(messages: &[Value]) -> Vec<String> {
    messages
        .iter()
        .flat_map(collect_message_user_ids)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn cached_user_to_value(user: &CachedUser) -> Value {
    let mut entry = serde_json::Map::new();
    entry.insert("id".to_string(), Value::String(user.id.clone()));
    entry.insert("name".to_string(), Value::String(user.name.clone()));
    entry.insert(
        "display_name".to_string(),
        user.display_name
            .as_ref()
            .map(|value| Value::String(value.clone()))
            .unwrap_or(Value::Null),
    );
    entry.insert(
        "real_name".to_string(),
        user.real_name
            .as_ref()
            .map(|value| Value::String(value.clone()))
            .unwrap_or(Value::Null),
    );
    entry.insert("deleted".to_string(), Value::Bool(user.deleted));
    entry.insert("is_bot".to_string(), Value::Bool(user.is_bot));
    Value::Object(entry)
}

fn slack_user_to_resolved_value(user: &Value) -> Option<Value> {
    let id = user.get("id")?.as_str()?;
    let name = user.get("name")?.as_str()?;
    let profile = user.get("profile");

    let mut entry = serde_json::Map::new();
    entry.insert("id".to_string(), Value::String(id.to_string()));
    entry.insert("name".to_string(), Value::String(name.to_string()));
    entry.insert(
        "display_name".to_string(),
        profile
            .and_then(|profile| profile.get("display_name"))
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .map(|value| Value::String(value.to_string()))
            .unwrap_or(Value::Null),
    );
    entry.insert(
        "real_name".to_string(),
        profile
            .and_then(|profile| profile.get("real_name"))
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .map(|value| Value::String(value.to_string()))
            .unwrap_or(Value::Null),
    );
    entry.insert(
        "deleted".to_string(),
        Value::Bool(
            user.get("deleted")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
        ),
    );
    entry.insert(
        "is_bot".to_string(),
        Value::Bool(
            user.get("is_bot")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
        ),
    );

    Some(Value::Object(entry))
}

/// Resolve thread-scoped user metadata without mutating Slack message payloads.
///
/// Cache hits are used first. Cache misses call `users.info`; only successfully
/// hydrated user records are returned in `resolved_users`. IDs that cannot be
/// resolved are returned separately as `unresolved_user_ids`.
pub async fn resolve_thread_users(
    client: &ApiClient,
    messages: &[Value],
    cache: Option<&WorkspaceCache>,
) -> Result<(BTreeMap<String, Value>, Vec<String>), ApiError> {
    let ids = collect_thread_user_ids(messages);
    let mut resolved_users = BTreeMap::new();
    let mut unresolved_user_ids = Vec::new();

    for id in ids {
        if let Some(cached) = cache.and_then(|cache| cache.users.get(&id)) {
            resolved_users.insert(id, cached_user_to_value(cached));
            continue;
        }

        match crate::commands::users_info(client, id.clone()).await {
            Ok(response) if response.ok => {
                if let Some(user) = response
                    .data
                    .get("user")
                    .and_then(slack_user_to_resolved_value)
                {
                    resolved_users.insert(id, user);
                } else {
                    unresolved_user_ids.push(id);
                }
            }
            Ok(_) => unresolved_user_ids.push(id),
            Err(error) => return Err(error),
        }
    }

    Ok((resolved_users, unresolved_user_ids))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::users_cache::CachedUser;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_output_format_parse() {
        assert_eq!(OutputFormat::parse("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::parse("jsonl").unwrap(), OutputFormat::Jsonl);
        assert_eq!(OutputFormat::parse("table").unwrap(), OutputFormat::Table);
        assert_eq!(OutputFormat::parse("tsv").unwrap(), OutputFormat::Tsv);
        assert!(OutputFormat::parse("invalid").is_err());
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

    // ── Enrichment tests ──

    #[test]
    fn test_collect_ids_poster_only() {
        let msg = json!({"user": "U111", "text": "hello world"});
        let ids = collect_message_user_ids(&msg);
        assert_eq!(ids, vec!["U111"]);
    }

    #[test]
    fn test_collect_ids_text_mentions() {
        let msg = json!({"user": "U111", "text": "hey <@U222> and <@U333>"});
        let ids = collect_message_user_ids(&msg);
        assert_eq!(ids, vec!["U111", "U222", "U333"]);
    }

    #[test]
    fn test_collect_ids_blocks_mentions() {
        let msg = json!({
            "user": "U111",
            "text": "hello",
            "blocks": [
                {
                    "type": "section",
                    "text": {"type": "mrkdwn", "text": "hey <@U444>"}
                },
                {
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                {"type": "text", "text": "and <@U555>"}
                            ]
                        }
                    ]
                }
            ]
        });
        let ids = collect_message_user_ids(&msg);
        assert_eq!(ids, vec!["U111", "U444", "U555"]);
    }

    #[test]
    fn test_collect_ids_deduplicated() {
        let msg = json!({"user": "U111", "text": "hey <@U111> <@U111> <@U222>"});
        let ids = collect_message_user_ids(&msg);
        assert_eq!(ids, vec!["U111", "U222"]);
    }

    #[test]
    fn test_collect_ids_sorted() {
        let msg = json!({"user": "U999", "text": "<@U111> <@U555> <@U222>"});
        let ids = collect_message_user_ids(&msg);
        assert_eq!(ids, vec!["U111", "U222", "U555", "U999"]);
    }

    #[test]
    fn test_enrich_with_cache() {
        let mut msg = json!({"user": "U111", "text": "hello <@U222>"});

        let mut users_map = HashMap::new();
        users_map.insert(
            "U111".to_string(),
            CachedUser {
                id: "U111".to_string(),
                name: "alice".to_string(),
                real_name: Some("Alice Smith".to_string()),
                display_name: Some("alice.s".to_string()),
                deleted: false,
                is_bot: false,
            },
        );
        users_map.insert(
            "U222".to_string(),
            CachedUser {
                id: "U222".to_string(),
                name: "bob".to_string(),
                real_name: Some("Bob Jones".to_string()),
                display_name: None,
                deleted: false,
                is_bot: false,
            },
        );

        let cache = WorkspaceCache {
            team_id: "T001".to_string(),
            updated_at: 1700000000,
            users: users_map,
        };

        enrich_message_with_users(&mut msg, Some(&cache));

        let users = msg.get("users").unwrap().as_array().unwrap();
        assert_eq!(users.len(), 2);

        let u0 = &users[0];
        assert_eq!(u0["id"], "U111");
        assert_eq!(u0["name"], "alice");
        assert_eq!(u0["display_name"], "alice.s");
        assert_eq!(u0["real_name"], "Alice Smith");

        let u1 = &users[1];
        assert_eq!(u1["id"], "U222");
        assert_eq!(u1["name"], "bob");
        assert_eq!(u1["display_name"], Value::Null);
        assert_eq!(u1["real_name"], "Bob Jones");
    }

    #[test]
    fn test_enrich_uncached_user() {
        let mut msg = json!({"user": "U999", "text": "hello"});

        let cache = WorkspaceCache {
            team_id: "T001".to_string(),
            updated_at: 1700000000,
            users: HashMap::new(), // empty cache
        };

        enrich_message_with_users(&mut msg, Some(&cache));

        let users = msg.get("users").unwrap().as_array().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["id"], "U999");
        // Uncached user: only id, no other fields
        assert!(users[0].get("name").is_none());
        assert!(users[0].get("display_name").is_none());
        assert!(users[0].get("real_name").is_none());
    }

    #[test]
    fn test_enrich_no_cache() {
        let mut msg = json!({"user": "U111", "text": "hello <@U222>"});

        enrich_message_with_users(&mut msg, None);

        let users = msg.get("users").unwrap().as_array().unwrap();
        assert_eq!(users.len(), 2);
        // All users have only id when no cache
        for u in users {
            assert!(u.get("id").is_some());
            assert!(u.get("name").is_none());
        }
    }

    #[test]
    fn test_enrich_with_pipe_mentions() {
        let mut msg = json!({"user": "U111", "text": "hello <@U222|bob> and <@U333>"});

        let cache = WorkspaceCache {
            team_id: "T001".to_string(),
            updated_at: 1700000000,
            users: HashMap::new(),
        };

        enrich_message_with_users(&mut msg, Some(&cache));

        let users = msg.get("users").unwrap().as_array().unwrap();
        assert_eq!(users.len(), 3);
        let ids: Vec<&str> = users.iter().map(|u| u["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["U111", "U222", "U333"]);
    }

    #[test]
    fn test_collect_ids_reaction_users() {
        let msg = json!({
            "user": "U111",
            "text": "hello <@U222>",
            "reactions": [
                {"name": "thumbsup", "users": ["U333", "U111"]},
                {"name": "eyes", "users": ["U444"]}
            ]
        });

        let ids = collect_message_user_ids(&msg);
        assert_eq!(ids, vec!["U111", "U222", "U333", "U444"]);
    }

    #[test]
    fn test_collect_thread_user_ids_deduplicates_across_messages() {
        let messages = vec![
            json!({"user": "U222", "text": "hello <@U111>", "reactions": [{"users": ["U333"]}]}),
            json!({"user": "U333", "text": "again <@U222>", "reactions": [{"users": ["U444", "U111"]}]}),
        ];

        let ids = collect_thread_user_ids(&messages);
        assert_eq!(ids, vec!["U111", "U222", "U333", "U444"]);
    }
}
