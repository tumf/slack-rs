//! Integration tests for thread get command
//!
//! Tests with mock HTTP server to verify:
//! - Thread message retrieval
//! - Parameter passing
//! - Pagination following next_cursor
//! - Message aggregation from multiple pages

use httpmock::prelude::*;
use serde_json::json;
use slack_rs::api::ApiClient;
use slack_rs::cli::{add_thread_resolution_metadata, build_thread_get_output};
use slack_rs::commands::conv::{collect_thread_user_ids, resolve_thread_users};
use slack_rs::commands::thread_get;
use slack_rs::commands::users_cache::{CachedUser, WorkspaceCache};
use slack_rs::profile::ProfilesConfig;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_thread_get_single_page() {
    // Start a mock server
    let server = MockServer::start();

    // Create a mock endpoint for conversations.replies
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/conversations.replies")
            .header("Authorization", "Bearer test-token")
            .query_param("channel", "C123456")
            .query_param("ts", "1234567890.123456")
            .query_param("limit", "100");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "ok": true,
                "messages": [
                    {
                        "type": "message",
                        "user": "U123",
                        "text": "Parent message",
                        "ts": "1234567890.123456"
                    },
                    {
                        "type": "message",
                        "user": "U456",
                        "text": "Reply 1",
                        "ts": "1234567891.123456",
                        "thread_ts": "1234567890.123456"
                    }
                ],
                "has_more": false,
                "response_metadata": {
                    "next_cursor": ""
                }
            }));
    });

    // Create API client with mock server URL
    let client = ApiClient::new_with_base_url("test-token".to_string(), server.base_url());

    // Execute thread_get
    let response = thread_get(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    // Verify response
    assert!(response.ok);
    let messages = response.data.get("messages").unwrap().as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["text"], "Parent message");
    assert_eq!(messages[1]["text"], "Reply 1");

    // Verify mock was called once
    mock.assert();
}

#[tokio::test]
async fn test_thread_get_with_pagination() {
    // Start a mock server
    let server = MockServer::start();

    // First page returns a cursor for the second page
    let mock_page_1 = server.mock(|when, then| {
        when.method(GET)
            .path("/conversations.replies")
            .query_param("channel", "C123456")
            .query_param("ts", "1234567890.123456")
            .query_param("limit", "2")
            .query_param_missing("cursor");
        then.status(200).json_body(json!({
            "ok": true,
            "messages": [
                {"type": "message", "user": "U123", "text": "Message 1", "ts": "1234567890.123456"},
                {"type": "message", "user": "U456", "text": "Message 2", "ts": "1234567891.123456"},
            ],
            "has_more": true,
            "response_metadata": {"next_cursor": "cursor-1"}
        }));
    });

    // Second page (with cursor) returns the remaining message and no next cursor
    let mock_page_2 = server.mock(|when, then| {
        when.method(GET)
            .path("/conversations.replies")
            .query_param("channel", "C123456")
            .query_param("ts", "1234567890.123456")
            .query_param("limit", "2")
            .query_param("cursor", "cursor-1");
        then.status(200).json_body(json!({
            "ok": true,
            "messages": [
                {"type": "message", "user": "U789", "text": "Message 3", "ts": "1234567892.123456"}
            ],
            "has_more": false,
            "response_metadata": {"next_cursor": ""}
        }));
    });

    // Create API client with mock server URL
    let client = ApiClient::new_with_base_url("test-token".to_string(), server.base_url());

    // Execute thread_get
    let response = thread_get(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        Some(2),
        None,
    )
    .await
    .unwrap();

    // Verify both pages were fetched
    assert_eq!(mock_page_1.calls(), 1);
    assert_eq!(mock_page_2.calls(), 1);

    // Verify response contains all messages from both pages
    assert!(response.ok);
    let messages = response.data.get("messages").unwrap().as_array().unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0]["text"], "Message 1");
    assert_eq!(messages[1]["text"], "Message 2");
    assert_eq!(messages[2]["text"], "Message 3");

    // Verify response_metadata has empty next_cursor
    let metadata = response.data.get("response_metadata").unwrap();
    assert_eq!(metadata["next_cursor"], "");
}

#[tokio::test]
async fn test_thread_get_with_inclusive_param() {
    // Start a mock server
    let server = MockServer::start();

    // Create a mock endpoint with inclusive parameter
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/conversations.replies")
            .header("Authorization", "Bearer test-token")
            .query_param("channel", "C123456")
            .query_param("ts", "1234567890.123456")
            .query_param("limit", "100")
            .query_param("inclusive", "true");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "ok": true,
                "messages": [
                    {
                        "type": "message",
                        "user": "U123",
                        "text": "Parent message",
                        "ts": "1234567890.123456"
                    }
                ],
                "response_metadata": {
                    "next_cursor": ""
                }
            }));
    });

    // Create API client with mock server URL
    let client = ApiClient::new_with_base_url("test-token".to_string(), server.base_url());

    // Execute thread_get with inclusive=true
    let response = thread_get(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        None,
        Some(true),
    )
    .await
    .unwrap();

    // Verify response
    assert!(response.ok);
    let messages = response.data.get("messages").unwrap().as_array().unwrap();
    assert_eq!(messages.len(), 1);

    // Verify mock was called with inclusive parameter
    mock.assert();
}

#[tokio::test]
async fn test_thread_user_resolution_uses_response_scope_without_message_users() {
    let server = MockServer::start();
    let replies_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/conversations.replies")
            .query_param("channel", "C123456")
            .query_param("ts", "1234567890.123456")
            .query_param("limit", "100");
        then.status(200).json_body(json!({
            "ok": true,
            "messages": [
                {
                    "type": "message",
                    "user": "U111",
                    "text": "hello <@U222>",
                    "blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "cc <@U333>"}}],
                    "reactions": [{"name": "eyes", "users": ["U444", "U222"]}],
                    "ts": "1234567890.123456"
                }
            ],
            "response_metadata": {"next_cursor": ""}
        }));
    });

    let users_info_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/users.info")
            .query_param("user", "U444");
        then.status(200).json_body(json!({
            "ok": true,
            "user": {
                "id": "U444",
                "name": "dave",
                "profile": {"display_name": "Dave", "real_name": "Dave D"},
                "deleted": false,
                "is_bot": false
            }
        }));
    });

    let client = ApiClient::new_with_base_url("test-token".to_string(), server.base_url());
    let mut response = thread_get(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    let messages = response
        .data
        .get("messages")
        .unwrap()
        .as_array()
        .unwrap()
        .clone();
    assert!(messages
        .iter()
        .all(|message| message.get("users").is_none()));

    let mut users = HashMap::new();
    users.insert(
        "U111".to_string(),
        CachedUser {
            id: "U111".to_string(),
            name: "alice".to_string(),
            real_name: Some("Alice A".to_string()),
            display_name: Some("Alice".to_string()),
            deleted: false,
            is_bot: false,
        },
    );
    users.insert(
        "U222".to_string(),
        CachedUser {
            id: "U222".to_string(),
            name: "bob".to_string(),
            real_name: Some("Bob B".to_string()),
            display_name: Some("Bob".to_string()),
            deleted: false,
            is_bot: false,
        },
    );
    users.insert(
        "U333".to_string(),
        CachedUser {
            id: "U333".to_string(),
            name: "carol".to_string(),
            real_name: Some("Carol C".to_string()),
            display_name: Some("Carol".to_string()),
            deleted: false,
            is_bot: false,
        },
    );
    let cache = WorkspaceCache {
        team_id: "T001".to_string(),
        updated_at: 1700000000,
        users,
    };

    let (resolved_users, unresolved_user_ids) =
        resolve_thread_users(&client, &messages, Some(&cache))
            .await
            .unwrap();
    response.data.insert(
        "resolved_users".to_string(),
        serde_json::to_value(resolved_users).unwrap(),
    );
    if !unresolved_user_ids.is_empty() {
        response.data.insert(
            "unresolved_user_ids".to_string(),
            serde_json::to_value(unresolved_user_ids).unwrap(),
        );
    }

    replies_mock.assert();
    users_info_mock.assert();
    assert_eq!(
        collect_thread_user_ids(&messages),
        vec!["U111", "U222", "U333", "U444"]
    );
    let resolved = response.data.get("resolved_users").unwrap();
    assert_eq!(resolved["U111"]["name"], "alice");
    assert_eq!(resolved["U222"]["name"], "bob");
    assert_eq!(resolved["U333"]["name"], "carol");
    assert_eq!(resolved["U444"]["name"], "dave");
    assert!(!response.data.contains_key("unresolved_user_ids"));
    assert!(response
        .data
        .get("messages")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .all(|message| message.get("users").is_none()));
}

#[tokio::test]
async fn test_thread_get_raw_shape_has_no_wrapper_resolution_metadata() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/conversations.replies")
            .query_param("channel", "C123456")
            .query_param("ts", "1234567890.123456")
            .query_param("limit", "100");
        then.status(200).json_body(json!({
            "ok": true,
            "messages": [{"type": "message", "user": "U111", "text": "hello <@U222>", "ts": "1234567890.123456"}],
            "response_metadata": {"next_cursor": ""}
        }));
    });

    let client = ApiClient::new_with_base_url("test-token".to_string(), server.base_url());
    let response = thread_get(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    mock.assert();
    assert!(!response.data.contains_key("resolved_users"));
    assert!(!response.data.contains_key("unresolved_user_ids"));
    assert!(response
        .data
        .get("messages")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .all(|message| message.get("users").is_none()));
}

#[tokio::test]
async fn test_resolve_thread_users_records_users_info_slack_error_as_unresolved() {
    let server = MockServer::start();
    let users_info_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/users.info")
            .query_param("user", "U404");
        then.status(200).json_body(json!({
            "ok": false,
            "error": "user_not_found"
        }));
    });

    let client = ApiClient::new_with_base_url("test-token".to_string(), server.base_url());
    let messages = vec![json!({
        "type": "message",
        "user": "U404",
        "text": "missing profile",
        "ts": "1234567890.123456"
    })];

    let (resolved_users, unresolved_user_ids) = resolve_thread_users(&client, &messages, None)
        .await
        .expect("users.info Slack errors should not abort thread enrichment");

    users_info_mock.assert();
    assert!(resolved_users.is_empty());
    assert_eq!(unresolved_user_ids, vec!["U404"]);
}

#[tokio::test]
async fn test_thread_get_default_cli_metadata_keeps_messages_slack_native() {
    let server = MockServer::start();
    let users_info_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/users.info")
            .query_param("user", "U222");
        then.status(200).json_body(json!({
            "ok": true,
            "user": {
                "id": "U222",
                "name": "bob",
                "profile": {"display_name": "Bob", "real_name": "Bob B"},
                "deleted": false,
                "is_bot": false
            }
        }));
    });

    let client = ApiClient::new_with_base_url("test-token".to_string(), server.base_url());
    let mut response = slack_rs::api::ApiResponse {
        ok: true,
        data: HashMap::from([(
            "messages".to_string(),
            json!([{
                "type": "message",
                "user": "U111",
                "text": "hello <@U222>",
                "ts": "1234567890.123456"
            }]),
        )]),
        error: None,
    };
    let messages = response.data["messages"].as_array().unwrap().clone();
    let cache = WorkspaceCache {
        team_id: "T001".to_string(),
        updated_at: 1700000000,
        users: HashMap::from([(
            "U111".to_string(),
            CachedUser {
                id: "U111".to_string(),
                name: "alice".to_string(),
                real_name: Some("Alice A".to_string()),
                display_name: Some("Alice".to_string()),
                deleted: false,
                is_bot: false,
            },
        )]),
    };

    add_thread_resolution_metadata(&client, &mut response, &messages, Some(&cache))
        .await
        .unwrap();

    users_info_mock.assert();
    let resolved = response.data.get("resolved_users").unwrap();
    assert_eq!(resolved["U111"]["name"], "alice");
    assert_eq!(resolved["U222"]["name"], "bob");
    assert!(!response.data.contains_key("unresolved_user_ids"));
    assert!(response
        .data
        .get("messages")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .all(|message| message.get("users").is_none()));
}

#[tokio::test]
async fn test_thread_get_default_cli_metadata_separates_unresolved_ids() {
    let server = MockServer::start();
    let users_info_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/users.info")
            .query_param("user", "U404");
        then.status(200).json_body(json!({
            "ok": false,
            "error": "user_not_found"
        }));
    });

    let client = ApiClient::new_with_base_url("test-token".to_string(), server.base_url());
    let mut response = slack_rs::api::ApiResponse {
        ok: true,
        data: HashMap::from([(
            "messages".to_string(),
            json!([{
                "type": "message",
                "user": "U111",
                "text": "cc <@U404>",
                "ts": "1234567890.123456"
            }]),
        )]),
        error: None,
    };
    let messages = response.data["messages"].as_array().unwrap().clone();
    let cache = WorkspaceCache {
        team_id: "T001".to_string(),
        updated_at: 1700000000,
        users: HashMap::from([(
            "U111".to_string(),
            CachedUser {
                id: "U111".to_string(),
                name: "alice".to_string(),
                real_name: Some("Alice A".to_string()),
                display_name: Some("Alice".to_string()),
                deleted: false,
                is_bot: false,
            },
        )]),
    };

    add_thread_resolution_metadata(&client, &mut response, &messages, Some(&cache))
        .await
        .expect("user lookup failure should not abort default thread output metadata");

    users_info_mock.assert();
    let resolved = response.data.get("resolved_users").unwrap();
    assert_eq!(resolved["U111"]["name"], "alice");
    assert!(resolved.get("U404").is_none());
    assert_eq!(response.data["unresolved_user_ids"], json!(["U404"]));
    assert!(response
        .data
        .get("messages")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .all(|message| message.get("users").is_none()));
}

fn write_default_profile_config(temp_dir: &TempDir) -> std::path::PathBuf {
    let config_path = temp_dir.path().join("profiles.json");
    let mut config = ProfilesConfig::new();
    config.set(
        "default".to_string(),
        slack_rs::profile::Profile::with_scopes(
            "T001".to_string(),
            "UCLI".to_string(),
            Some("Test Team".to_string()),
            Some("CLI User".to_string()),
            None,
            None,
            None,
            None,
        ),
    );
    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
    config_path
}

#[tokio::test]
#[serial_test::serial]
async fn test_thread_get_cli_output_builder_default_envelope_separates_resolution_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = write_default_profile_config(&temp_dir);
    std::env::set_var("SLACK_RS_CONFIG_PATH", &config_path);

    let server = MockServer::start();
    let replies_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/conversations.replies")
            .query_param("channel", "C123456")
            .query_param("ts", "1234567890.123456")
            .query_param("limit", "100");
        then.status(200).json_body(json!({
            "ok": true,
            "messages": [{
                "type": "message",
                "user": "U111",
                "text": "hello <@U222> cc <@U404>",
                "ts": "1234567890.123456"
            }],
            "response_metadata": {"next_cursor": ""}
        }));
    });
    let resolved_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/users.info")
            .query_param("user", "U111");
        then.status(200).json_body(json!({
            "ok": true,
            "user": {
                "id": "U111",
                "name": "alice",
                "profile": {"display_name": "Alice", "real_name": "Alice A"},
                "deleted": false,
                "is_bot": false
            }
        }));
    });
    let mentioned_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/users.info")
            .query_param("user", "U222");
        then.status(200).json_body(json!({
            "ok": true,
            "user": {
                "id": "U222",
                "name": "bob",
                "profile": {"display_name": "Bob", "real_name": "Bob B"},
                "deleted": false,
                "is_bot": false
            }
        }));
    });
    let unresolved_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/users.info")
            .query_param("user", "U404");
        then.status(200).json_body(json!({
            "ok": false,
            "error": "user_not_found"
        }));
    });

    let client = ApiClient::new_with_base_url("test-token".to_string(), server.base_url());
    let output = build_thread_get_output(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        None,
        None,
        false,
        "default".to_string(),
        None,
    )
    .await
    .expect("default CLI output should survive users.info failures");

    std::env::remove_var("SLACK_RS_CONFIG_PATH");
    replies_mock.assert();
    resolved_mock.assert();
    mentioned_mock.assert();
    unresolved_mock.assert();

    assert_eq!(
        output["response"]["resolved_users"]["U111"]["name"],
        "alice"
    );
    assert_eq!(output["response"]["resolved_users"]["U222"]["name"], "bob");
    assert!(output["response"]["resolved_users"].get("U404").is_none());
    assert_eq!(output["response"]["unresolved_user_ids"], json!(["U404"]));
    assert!(output["response"]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .all(|message| message.get("users").is_none()));
    assert_eq!(output["meta"]["method"], "conversations.replies");
    assert_eq!(output["meta"]["command"], "thread get");
}

#[tokio::test]
#[serial_test::serial]
async fn test_thread_get_cli_output_builder_raw_skips_wrapper_resolution() {
    std::env::remove_var("SLACK_RS_CONFIG_PATH");
    let server = MockServer::start();
    let replies_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/conversations.replies")
            .query_param("channel", "C123456")
            .query_param("ts", "1234567890.123456")
            .query_param("limit", "100");
        then.status(200).json_body(json!({
            "ok": true,
            "messages": [{
                "type": "message",
                "user": "U111",
                "text": "hello <@U222>",
                "ts": "1234567890.123456"
            }],
            "response_metadata": {"next_cursor": ""}
        }));
    });
    let users_info_mock = server.mock(|when, then| {
        when.method(GET).path("/users.info");
        then.status(500)
            .json_body(json!({"ok": false, "error": "should_not_be_called"}));
    });

    let client = ApiClient::new_with_base_url("test-token".to_string(), server.base_url());
    let output = build_thread_get_output(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        None,
        None,
        true,
        "default".to_string(),
        None,
    )
    .await
    .expect("raw CLI output should not need profile config or user enrichment");

    replies_mock.assert();
    users_info_mock.assert_calls(0);
    assert!(output.get("response").is_none());
    assert!(output.get("resolved_users").is_none());
    assert!(output.get("unresolved_user_ids").is_none());
    assert!(output["messages"]
        .as_array()
        .unwrap()
        .iter()
        .all(|message| message.get("users").is_none()));
}
