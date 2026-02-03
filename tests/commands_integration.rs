//! Integration tests for wrapper commands
//!
//! Tests that each command calls the correct Slack API methods with proper parameters

use slack_rs::api::ApiClient;
use slack_rs::commands;
use std::collections::HashMap;
use wiremock::matchers::{header, method, path};
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

    Mock::given(method("POST"))
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
async fn test_search_with_sort_parameters() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Setup mock response
    let mut response_data = HashMap::new();
    response_data.insert("ok".to_string(), serde_json::json!(true));
    response_data.insert(
        "messages".to_string(),
        serde_json::json!({"total": 1, "matches": []}),
    );

    Mock::given(method("POST"))
        .and(path("/search.messages"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create API client
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Call search command with sort parameters
    let result = commands::search(
        &client,
        "test query".to_string(),
        Some(20),
        Some(1),
        Some("timestamp".to_string()),
        Some("desc".to_string()),
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

    Mock::given(method("POST"))
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

    Mock::given(method("POST"))
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
async fn test_msg_post_requires_allow_write() {
    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Should fail without --allow-write flag
    let result = commands::msg_post(
        &client,
        "C123456".to_string(),
        "test message".to_string(),
        false,
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("requires --allow-write"));
}

#[tokio::test]
async fn test_msg_post_calls_correct_api_with_allow_write() {
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
        true, // allow_write = true
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_msg_update_requires_allow_write() {
    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    let result = commands::msg_update(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        "updated text".to_string(),
        false, // allow_write = false
        true,  // yes = true (skip confirmation)
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("requires --allow-write"));
}

#[tokio::test]
async fn test_msg_update_calls_correct_api() {
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
        true, // allow_write = true
        true, // yes = true (skip confirmation)
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_msg_delete_requires_allow_write() {
    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    let result = commands::msg_delete(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        false, // allow_write = false
        true,  // yes = true
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("requires --allow-write"));
}

#[tokio::test]
async fn test_msg_delete_calls_correct_api() {
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
        true, // allow_write = true
        true, // yes = true
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_react_add_requires_allow_write() {
    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    let result = commands::react_add(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        "thumbsup".to_string(),
        false, // allow_write = false
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("requires --allow-write"));
}

#[tokio::test]
async fn test_react_add_calls_correct_api() {
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
        true, // allow_write = true
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_react_remove_requires_allow_write() {
    let mock_server = MockServer::start().await;
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    let result = commands::react_remove(
        &client,
        "C123456".to_string(),
        "1234567890.123456".to_string(),
        "thumbsup".to_string(),
        false, // allow_write = false
        true,  // yes = true
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("requires --allow-write"));
}

#[tokio::test]
async fn test_react_remove_calls_correct_api() {
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
        true, // allow_write = true
        true, // yes = true
    )
    .await;

    assert!(result.is_ok());
}
