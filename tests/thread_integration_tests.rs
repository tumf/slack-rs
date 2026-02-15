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
use slack_rs::commands::thread_get;

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
