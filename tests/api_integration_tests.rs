//! Integration tests for API call functionality
//!
//! Tests with mock HTTP server to verify:
//! - HTTP request handling
//! - Form and JSON content types
//! - Retry logic with 429 responses
//! - Metadata attachment

use httpmock::prelude::*;
use serde_json::json;
use slack_rs::api::{execute_api_call, ApiCallArgs, ApiCallContext, ApiClient, ApiClientConfig};

#[tokio::test]
async fn test_api_call_with_form_data() {
    // Start a mock server
    let server = MockServer::start();

    // Create a mock endpoint
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat.postMessage")
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/x-www-form-urlencoded");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "ok": true,
                "channel": "C123456",
                "ts": "1234567890.123456"
            }));
    });

    // Create API client with mock server URL
    let config = ApiClientConfig {
        base_url: server.base_url(),
        max_retries: 3,
        initial_backoff_ms: 100,
        max_backoff_ms: 1000,
    };
    let client = ApiClient::with_config(config);

    // Parse arguments
    let args_vec = vec![
        "chat.postMessage".to_string(),
        "channel=C123456".to_string(),
        "text=Hello".to_string(),
    ];
    let args = ApiCallArgs::parse(&args_vec).unwrap();

    // Create context
    let context = ApiCallContext {
        profile_name: Some("test".to_string()),
        team_id: "T123ABC".to_string(),
        user_id: "U456DEF".to_string(),
    };

    // Execute API call
    let response = execute_api_call(&client, &args, "test-token", &context, "bot", "api call")
        .await
        .unwrap();

    // Verify response
    assert_eq!(response.response["ok"], true);
    assert_eq!(response.response["channel"], "C123456");

    // Verify metadata
    assert_eq!(response.meta.profile_name, Some("test".to_string()));
    assert_eq!(response.meta.team_id, "T123ABC");
    assert_eq!(response.meta.user_id, "U456DEF");
    assert_eq!(response.meta.method, "chat.postMessage");
    assert_eq!(response.meta.command, "api call");
    assert_eq!(response.meta.token_type, "bot");

    // Verify mock was called
    mock.assert();
}

#[tokio::test]
async fn test_api_call_with_json_data() {
    // Start a mock server
    let server = MockServer::start();

    // Create a mock endpoint
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/chat.postMessage")
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "ok": true,
                "channel": "C123456",
                "ts": "1234567890.123456"
            }));
    });

    // Create API client with mock server URL
    let config = ApiClientConfig {
        base_url: server.base_url(),
        max_retries: 3,
        initial_backoff_ms: 100,
        max_backoff_ms: 1000,
    };
    let client = ApiClient::with_config(config);

    // Parse arguments with --json flag
    let args_vec = vec![
        "chat.postMessage".to_string(),
        "--json".to_string(),
        "channel=C123456".to_string(),
        "text=Hello".to_string(),
    ];
    let args = ApiCallArgs::parse(&args_vec).unwrap();

    // Create context
    let context = ApiCallContext {
        profile_name: None,
        team_id: "T123ABC".to_string(),
        user_id: "U456DEF".to_string(),
    };

    // Execute API call
    let response = execute_api_call(&client, &args, "test-token", &context, "bot", "api call")
        .await
        .unwrap();

    // Verify response
    assert_eq!(response.response["ok"], true);

    // Verify metadata
    assert_eq!(response.meta.profile_name, None);
    assert_eq!(response.meta.team_id, "T123ABC");
    assert_eq!(response.meta.command, "api call");
    assert_eq!(response.meta.token_type, "bot");

    // Verify mock was called
    mock.assert();
}

#[tokio::test]
async fn test_api_call_with_get_method() {
    // Start a mock server
    let server = MockServer::start();

    // Create a mock endpoint
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/users.info")
            .header("Authorization", "Bearer test-token");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "ok": true,
                "user": {
                    "id": "U123456",
                    "name": "testuser"
                }
            }));
    });

    // Create API client with mock server URL
    let config = ApiClientConfig {
        base_url: server.base_url(),
        max_retries: 3,
        initial_backoff_ms: 100,
        max_backoff_ms: 1000,
    };
    let client = ApiClient::with_config(config);

    // Parse arguments with --get flag
    let args_vec = vec![
        "users.info".to_string(),
        "--get".to_string(),
        "user=U123456".to_string(),
    ];
    let args = ApiCallArgs::parse(&args_vec).unwrap();

    // Create context
    let context = ApiCallContext {
        profile_name: Some("default".to_string()),
        team_id: "T123ABC".to_string(),
        user_id: "U456DEF".to_string(),
    };

    // Execute API call
    let response = execute_api_call(&client, &args, "test-token", &context, "user", "api call")
        .await
        .unwrap();

    // Verify response
    assert_eq!(response.response["ok"], true);
    assert_eq!(response.response["user"]["id"], "U123456");
    assert_eq!(response.meta.token_type, "user");

    // Verify mock was called
    mock.assert();
}

#[tokio::test]
async fn test_api_call_retry_on_429() {
    // Start a mock server
    let server = MockServer::start();

    // Create a mock that always returns 429
    // This test verifies that we respect the Retry-After header and eventually give up
    let mock = server.mock(|when, then| {
        when.method(POST).path("/chat.postMessage");
        then.status(429)
            .header("Retry-After", "1")
            .header("content-type", "application/json")
            .json_body(json!({
                "ok": false,
                "error": "rate_limited"
            }));
    });

    // Create API client with mock server URL and short retry delay
    let config = ApiClientConfig {
        base_url: server.base_url(),
        max_retries: 2, // Limit retries for faster test
        initial_backoff_ms: 100,
        max_backoff_ms: 1000,
    };
    let client = ApiClient::with_config(config);

    // Parse arguments
    let args_vec = vec![
        "chat.postMessage".to_string(),
        "channel=C123456".to_string(),
    ];
    let args = ApiCallArgs::parse(&args_vec).unwrap();

    // Create context
    let context = ApiCallContext {
        profile_name: Some("test".to_string()),
        team_id: "T123ABC".to_string(),
        user_id: "U456DEF".to_string(),
    };

    // Execute API call - should retry and eventually fail with RateLimitExceeded
    let result = execute_api_call(&client, &args, "test-token", &context, "bot", "api call").await;

    // Verify that we get a rate limit error after retries
    assert!(result.is_err());

    // Verify we made multiple attempts (initial + max_retries)
    assert!(
        mock.calls() >= 3,
        "Expected at least 3 calls (1 initial + 2 retries), got {}",
        mock.calls()
    );
}

#[tokio::test]
async fn test_output_json_with_meta() {
    // Start a mock server
    let server = MockServer::start();

    // Create a mock endpoint
    server.mock(|when, then| {
        when.method(POST).path("/test.method");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "ok": true,
                "data": "test"
            }));
    });

    // Create API client with mock server URL
    let config = ApiClientConfig {
        base_url: server.base_url(),
        max_retries: 3,
        initial_backoff_ms: 100,
        max_backoff_ms: 1000,
    };
    let client = ApiClient::with_config(config);

    // Parse arguments
    let args_vec = vec!["test.method".to_string()];
    let args = ApiCallArgs::parse(&args_vec).unwrap();

    // Create context with all metadata
    let context = ApiCallContext {
        profile_name: Some("production".to_string()),
        team_id: "T999XYZ".to_string(),
        user_id: "U888ABC".to_string(),
    };

    // Execute API call
    let response = execute_api_call(&client, &args, "test-token", &context, "bot", "api call")
        .await
        .unwrap();

    // Verify all metadata fields are present
    assert_eq!(response.meta.profile_name, Some("production".to_string()));
    assert_eq!(response.meta.team_id, "T999XYZ");
    assert_eq!(response.meta.user_id, "U888ABC");
    assert_eq!(response.meta.method, "test.method");
    assert_eq!(response.meta.token_type, "bot");

    // Verify we can serialize the full response to JSON
    let json = serde_json::to_value(&response).unwrap();
    assert!(json["response"].is_object());
    assert!(json["meta"].is_object());
    assert_eq!(json["meta"]["profile_name"], "production");
    assert_eq!(json["meta"]["team_id"], "T999XYZ");
    assert_eq!(json["meta"]["user_id"], "U888ABC");
    assert_eq!(json["meta"]["method"], "test.method");
}
