//! Integration tests for wrapper commands
//!
//! Tests that each command calls the correct Slack API methods with proper parameters

use serial_test::serial;
use slack_rs::api::ApiClient;
use slack_rs::commands;
use slack_rs::commands::ConversationSelector;
use std::collections::HashMap;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_search_calls_correct_api() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Setup mock response
    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));
    response_data.insert(
        "messages".to_string(),
        serde_json::json!({"total": 1, "matches": []}),
    );

    Mock::given(method("GET"))
        .and(path("/search.messages"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create API client
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Call search command
    let result = commands::search(&client, "test query".to_string(), None, None, None, None).await;

    assert!(result.is_ok());
}

#[tokio::test]
#[serial(write_guard)]
async fn test_msg_post_with_thread_ts() {
    let mock_server = MockServer::start().await;

    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));
    response_data.insert("ts".to_string(), serde_json::json!("1234567890.123456"));

    Mock::given(method("POST"))
        .and(path("/chat.postMessage"))
        .and(header("authorization", "Bearer test_token"))
        .and(body_string_contains("thread_ts"))
        .and(body_string_contains("1234567890.111111"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::msg_post(
        &client,
        "C123456".to_string(),
        "thread reply".to_string(),
        Some("1234567890.111111".to_string()),
        false,
        true,
        false,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
#[serial(write_guard)]
async fn test_msg_post_with_thread_ts_and_reply_broadcast() {
    let mock_server = MockServer::start().await;

    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));
    response_data.insert("ts".to_string(), serde_json::json!("1234567890.123456"));

    Mock::given(method("POST"))
        .and(path("/chat.postMessage"))
        .and(header("authorization", "Bearer test_token"))
        .and(body_string_contains("thread_ts"))
        .and(body_string_contains("1234567890.111111"))
        .and(body_string_contains("reply_broadcast"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::msg_post(
        &client,
        "C123456".to_string(),
        "broadcast reply".to_string(),
        Some("1234567890.111111".to_string()),
        true, // reply_broadcast = true
        true,
        false,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
#[serial(write_guard)]
async fn test_msg_post_without_thread_ts_ignores_reply_broadcast() {
    let mock_server = MockServer::start().await;

    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));
    response_data.insert("ts".to_string(), serde_json::json!("1234567890.123456"));

    // Mock should NOT expect reply_broadcast or thread_ts in the body
    Mock::given(method("POST"))
        .and(path("/chat.postMessage"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::msg_post(
        &client,
        "C123456".to_string(),
        "normal message".to_string(),
        None, // no thread_ts
        true, // reply_broadcast = true (should be ignored)
        true,
        false,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_conv_history_calls_correct_api() {
    let mock_server = MockServer::start().await;

    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));
    response_data.insert("messages".to_string(), serde_json::json!([]));

    Mock::given(method("GET"))
        .and(path("/conversations.history"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::conv_history(&client, "C123456".to_string(), None, None, None).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_users_info_calls_correct_api() {
    let mock_server = MockServer::start().await;

    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));
    response_data.insert("user".to_string(), serde_json::json!({"id": "U123456"}));

    Mock::given(method("GET"))
        .and(path("/users.info"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::users_info(&client, "U123456".to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
#[serial(write_guard)]
async fn test_msg_post_requires_allow_write() {
    std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Should fail when SLACKCLI_ALLOW_WRITE=false
    let result = commands::msg_post(
        &client,
        "C123456".to_string(),
        "test message".to_string(),
        None,
        false,
        true,
        false,
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SLACKCLI_ALLOW_WRITE"));
    std::env::remove_var("SLACKCLI_ALLOW_WRITE");
}

#[tokio::test]
#[serial(write_guard)]
async fn test_msg_post_calls_correct_api_with_allow_write() {
    std::env::remove_var("SLACKCLI_ALLOW_WRITE"); // Default is allow
    let mock_server = MockServer::start().await;

    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));
    response_data.insert("ts".to_string(), serde_json::json!("1234567890.123456"));

    Mock::given(method("POST"))
        .and(path("/chat.postMessage"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::msg_post(
        &client,
        "C123456".to_string(),
        "test message".to_string(),
        None,
        false,
        true,
        false,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
#[serial(write_guard)]
async fn test_msg_update_requires_allow_write() {
    std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    let result = commands::msg_update(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        "updated text".to_string(),
        true,  // yes = true (skip confirmation)
        false, // non_interactive = false
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SLACKCLI_ALLOW_WRITE"));
    std::env::remove_var("SLACKCLI_ALLOW_WRITE");
}

#[tokio::test]
#[serial(write_guard)]
async fn test_msg_update_calls_correct_api() {
    std::env::remove_var("SLACKCLI_ALLOW_WRITE"); // Default is allow
    let mock_server = MockServer::start().await;

    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));

    Mock::given(method("POST"))
        .and(path("/chat.update"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::msg_update(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        "updated text".to_string(),
        true,  // yes = true (skip confirmation)
        false, // non_interactive = false
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
#[serial(write_guard)]
async fn test_msg_delete_requires_allow_write() {
    std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    let result = commands::msg_delete(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        true,  // yes = true
        false, // non_interactive = false
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SLACKCLI_ALLOW_WRITE"));
    std::env::remove_var("SLACKCLI_ALLOW_WRITE");
}

#[tokio::test]
#[serial(write_guard)]
async fn test_msg_delete_calls_correct_api() {
    std::env::remove_var("SLACKCLI_ALLOW_WRITE"); // Default is allow
    let mock_server = MockServer::start().await;

    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));

    Mock::given(method("POST"))
        .and(path("/chat.delete"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::msg_delete(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        true,  // yes = true
        false, // non_interactive = false
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
#[serial(write_guard)]
async fn test_react_add_requires_allow_write() {
    std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    let result = commands::react_add(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        "thumbsup".to_string(),
        true,
        false,
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SLACKCLI_ALLOW_WRITE"));
    std::env::remove_var("SLACKCLI_ALLOW_WRITE");
}

#[tokio::test]
#[serial(write_guard)]
async fn test_react_add_calls_correct_api() {
    std::env::remove_var("SLACKCLI_ALLOW_WRITE"); // Default is allow
    let mock_server = MockServer::start().await;

    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));

    Mock::given(method("POST"))
        .and(path("/reactions.add"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::react_add(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        "thumbsup".to_string(),
        true,
        false,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
#[serial(write_guard)]
async fn test_react_remove_requires_allow_write() {
    std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    let result = commands::react_remove(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        "thumbsup".to_string(),
        true,  // yes = true
        false, // non_interactive = false
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SLACKCLI_ALLOW_WRITE"));
    std::env::remove_var("SLACKCLI_ALLOW_WRITE");
}

#[tokio::test]
#[serial(write_guard)]
async fn test_react_remove_calls_correct_api() {
    std::env::remove_var("SLACKCLI_ALLOW_WRITE"); // Default is allow
    let mock_server = MockServer::start().await;

    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));

    Mock::given(method("POST"))
        .and(path("/reactions.remove"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::react_remove(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        "thumbsup".to_string(),
        true,  // yes = true
        false, // non_interactive = false
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
#[serial(write_guard)]
async fn test_file_upload_requires_allow_write() {
    // Set env var to deny write
    std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");

    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    let result = commands::file_upload(
        &client,
        "/tmp/test.txt".to_string(),
        None,
        None,
        None,
        true,
        false,
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Write operation denied"));

    std::env::remove_var("SLACKCLI_ALLOW_WRITE");
}

#[tokio::test]
#[serial(write_guard)]
async fn test_file_upload_external_flow() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a temporary file for testing
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "test content").unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();

    // Start a mock server for Slack API endpoints
    let mock_server = MockServer::start().await;

    // Mock files.getUploadURLExternal
    let get_url_response = serde_json::json!({
        "ok": true,
        "upload_url": format!("{}/upload", mock_server.uri()),
        "file_id": "F12345"
    });

    Mock::given(method("POST"))
        .and(path("/files.getUploadURLExternal"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&get_url_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock upload endpoint (external URL)
    Mock::given(method("POST"))
        .and(path("/upload"))
        .and(header("content-type", "application/octet-stream"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock files.completeUploadExternal
    let complete_response = serde_json::json!({
        "ok": true,
        "files": [{
            "id": "F12345",
            "title": "test file"
        }]
    });

    Mock::given(method("POST"))
        .and(path("/files.completeUploadExternal"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&complete_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create API client
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Ensure write is allowed
    std::env::remove_var("SLACKCLI_ALLOW_WRITE");

    // Call file_upload command
    let result = commands::file_upload(
        &client,
        file_path,
        Some("C123456".to_string()),
        Some("Test File".to_string()),
        Some("Test comment".to_string()),
        true,
        false,
    )
    .await;

    assert!(result.is_ok());
    let response_value = result.unwrap();
    assert!(response_value.get("ok").is_some());
}

#[tokio::test]
#[serial(write_guard)]
async fn test_file_upload_nonexistent_file() {
    // Ensure write is allowed
    std::env::remove_var("SLACKCLI_ALLOW_WRITE");

    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    let result = commands::file_upload(
        &client,
        "/nonexistent/file.txt".to_string(),
        None,
        None,
        None,
        true,
        false,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("File not found"));
}

#[tokio::test]
async fn test_conv_list_with_filters() {
    let mock_server = MockServer::start().await;

    // Mock conversations.list response
    let response_data = serde_json::json!({
        "ok": true,
        "channels": [
            {"id": "C1", "name": "test-public", "is_member": true, "is_private": false},
            {"id": "C2", "name": "test-private", "is_member": true, "is_private": true},
            {"id": "C3", "name": "general", "is_member": true, "is_private": false},
            {"id": "C4", "name": "test-nomember", "is_member": false, "is_private": false},
        ]
    });

    Mock::given(method("GET"))
        .and(path("/conversations.list"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Get the conversation list
    let mut response = commands::conv_list(&client, None, None).await.unwrap();

    // Apply filters: name:test* AND is_member:true
    let filters = vec![
        commands::ConversationFilter::parse("name:test*").unwrap(),
        commands::ConversationFilter::parse("is_member:true").unwrap(),
    ];
    commands::apply_filters(&mut response, &filters);

    // Extract and verify filtered results
    let items = commands::extract_conversations(&response);
    assert_eq!(items.len(), 2); // C1 and C2
    assert_eq!(items[0].id, "C1");
    assert_eq!(items[1].id, "C2");
}

#[tokio::test]
async fn test_conv_select_with_mock_selector() {
    let mock_server = MockServer::start().await;

    // Mock conversations.list response
    let response_data = serde_json::json!({
        "ok": true,
        "channels": [
            {"id": "C1", "name": "general", "is_private": false},
            {"id": "C2", "name": "random", "is_private": false},
        ]
    });

    Mock::given(method("GET"))
        .and(path("/conversations.list"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Get the conversation list
    let response = commands::conv_list(&client, None, None).await.unwrap();
    let items = commands::extract_conversations(&response);

    // Use mock selector to select second item
    struct TestSelector;
    impl commands::ConversationSelector for TestSelector {
        fn select(&self, items: &[commands::ConversationItem]) -> Result<String, String> {
            Ok(items[1].id.clone())
        }
    }

    let selector = TestSelector;
    let selected_id = selector.select(&items).unwrap();
    assert_eq!(selected_id, "C2");
}

#[tokio::test]
async fn test_conversation_item_display_format() {
    let item = commands::ConversationItem {
        id: "C123".to_string(),
        name: "general".to_string(),
        is_private: false,
    };
    assert_eq!(item.display(), "#general (C123)");

    let private_item = commands::ConversationItem {
        id: "C456".to_string(),
        name: "secret".to_string(),
        is_private: true,
    };
    assert_eq!(private_item.display(), "#secret (C456) [private]");
}

#[tokio::test]
async fn test_conv_search_filter_injection() {
    // Test that conv search applies name filter correctly
    let mock_server = MockServer::start().await;

    // Mock conversations.list response
    let response_data = serde_json::json!({
        "ok": true,
        "channels": [
            {"id": "C1", "name": "dev-backend", "is_member": true, "is_private": false},
            {"id": "C2", "name": "dev-frontend", "is_member": true, "is_private": false},
            {"id": "C3", "name": "general", "is_member": true, "is_private": false},
            {"id": "C4", "name": "dev-ops", "is_member": false, "is_private": false},
        ]
    });

    Mock::given(method("GET"))
        .and(path("/conversations.list"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Get the conversation list
    let mut response = commands::conv_list(&client, None, None).await.unwrap();

    // Apply search filter (simulates conv search "dev*")
    let filters = vec![commands::ConversationFilter::parse("name:dev*").unwrap()];
    commands::apply_filters(&mut response, &filters);

    // Extract and verify filtered results
    let items = commands::extract_conversations(&response);
    assert_eq!(items.len(), 3); // C1, C2, C4
    assert_eq!(items[0].id, "C1");
    assert_eq!(items[1].id, "C2");
    assert_eq!(items[2].id, "C4");
}

#[tokio::test]
async fn test_conv_search_with_additional_filters() {
    // Test that conv search can combine name filter with additional filters
    let mock_server = MockServer::start().await;

    // Mock conversations.list response
    let response_data = serde_json::json!({
        "ok": true,
        "channels": [
            {"id": "C1", "name": "test-public", "is_member": true, "is_private": false},
            {"id": "C2", "name": "test-private", "is_member": true, "is_private": true},
            {"id": "C3", "name": "test-nomember", "is_member": false, "is_private": false},
            {"id": "C4", "name": "general", "is_member": true, "is_private": false},
        ]
    });

    Mock::given(method("GET"))
        .and(path("/conversations.list"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Get the conversation list
    let mut response = commands::conv_list(&client, None, None).await.unwrap();

    // Apply search filter with additional filters (simulates conv search "test*" --filter is_member:true)
    let filters = vec![
        commands::ConversationFilter::parse("name:test*").unwrap(),
        commands::ConversationFilter::parse("is_member:true").unwrap(),
    ];
    commands::apply_filters(&mut response, &filters);

    // Extract and verify filtered results
    let items = commands::extract_conversations(&response);
    assert_eq!(items.len(), 2); // C1 and C2
    assert_eq!(items[0].id, "C1");
    assert_eq!(items[1].id, "C2");
}

#[tokio::test]
async fn test_file_download_uses_form_params_for_files_info() {
    use tempfile::TempDir;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Create mock server
    let mock_server = MockServer::start().await;

    // Mock files.info response - verify it receives form-encoded parameters
    let files_info_response = serde_json::json!({
        "ok": true,
        "file": {
            "id": "F12345",
            "name": "test_file.txt",
            "url_private_download": format!("{}/download/test_file.txt", mock_server.uri())
        }
    });

    // This mock expects form-encoded body (Content-Type: application/x-www-form-urlencoded)
    // and verifies that the parameter is sent as form data, not JSON
    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock file download response
    let file_content = b"test file content";
    Mock::given(method("GET"))
        .and(path("/download/test_file.txt"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(file_content))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create API client
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Create temp directory for output
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("downloaded_file.txt");

    // Call file_download
    let result = commands::file_download(
        &client,
        Some("F12345".to_string()),
        None,
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(result.is_ok(), "file_download should succeed");
    let response = result.unwrap();
    assert_eq!(response["ok"], true);

    // Verify file was written
    let downloaded_content = std::fs::read(&output_path).unwrap();
    assert_eq!(downloaded_content, file_content);
}

#[tokio::test]
async fn test_file_download_rejects_json_body_for_files_info() {
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Create mock server
    let mock_server = MockServer::start().await;

    // Mock that ONLY accepts JSON content-type and returns error
    // This simulates the Slack API behavior when receiving JSON instead of form data
    let error_response = serde_json::json!({
        "ok": false,
        "error": "invalid_arguments"
    });

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&error_response))
        .expect(0) // Expect 0 calls - we should NOT send JSON
        .mount(&mock_server)
        .await;

    // Mock that accepts form-encoded (the correct way)
    let files_info_response = serde_json::json!({
        "ok": true,
        "file": {
            "id": "F12345",
            "name": "test_file.txt",
            "url_private_download": format!("{}/download/test_file.txt", mock_server.uri())
        }
    });

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1) // Expect 1 call with form encoding
        .mount(&mock_server)
        .await;

    // Mock file download response
    Mock::given(method("GET"))
        .and(path("/download/test_file.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"content"))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create API client
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Call file_download - should succeed because we use form encoding
    let result = commands::file_download(
        &client,
        Some("F12345".to_string()),
        None,
        Some("-".to_string()), // Write to stdout
    )
    .await;

    assert!(
        result.is_ok(),
        "file_download should succeed with form encoding"
    );
}

#[tokio::test]
async fn test_file_download_complete_flow_with_files_info() {
    use tempfile::TempDir;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Create mock server
    let mock_server = MockServer::start().await;

    // Mock files.info response
    let files_info_response = serde_json::json!({
        "ok": true,
        "file": {
            "id": "F12345",
            "name": "important_doc.pdf",
            "url_private_download": format!("{}/files/download/important_doc.pdf", mock_server.uri())
        }
    });

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock file download response
    let file_content = b"PDF content here";
    Mock::given(method("GET"))
        .and(path("/files/download/important_doc.pdf"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(file_content)
                .insert_header("content-type", "application/pdf"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create API client
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Create temp directory for output
    let temp_dir = TempDir::new().unwrap();

    // Test: Download to specific file path
    let output_file = temp_dir.path().join("output.pdf");
    let result = commands::file_download(
        &client,
        Some("F12345".to_string()),
        None,
        Some(output_file.to_str().unwrap().to_string()),
    )
    .await;

    assert!(result.is_ok(), "file_download should succeed");
    let response = result.unwrap();
    assert_eq!(response["ok"], true);
    assert_eq!(response["size"], file_content.len());

    // Verify file was written correctly
    let downloaded_content = std::fs::read(&output_file).unwrap();
    assert_eq!(downloaded_content, file_content);
}
