//! Integration tests for OAuth flow with mock server

use slack_rs::oauth::{exchange_code, OAuthConfig};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_exchange_code_with_mock_server() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Create mock response matching Slack's oauth.v2.access response
    let response_body = serde_json::json!({
        "ok": true,
        "access_token": "xoxb-mock-token-12345",
        "token_type": "bot",
        "scope": "chat:write,users:read",
        "bot_user_id": "U123BOT",
        "app_id": "A456APP",
        "team": {
            "id": "T789TEAM",
            "name": "Mock Team"
        },
        "authed_user": {
            "id": "U012USER",
            "scope": "chat:write,users:read",
            "access_token": "xoxp-mock-user-token",
            "token_type": "user"
        }
    });

    Mock::given(method("POST"))
        .and(path("/oauth.v2.access"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    // Create OAuth config
    let config = OAuthConfig {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        redirect_uri: "http://localhost:8765/callback".to_string(),
        bot_scopes: vec!["chat:write".to_string()],
        user_scopes: vec![],
    };

    // Exchange code
    let result = exchange_code(
        &config,
        "mock_auth_code",
        "mock_code_verifier",
        Some(&mock_server.uri()),
    )
    .await;

    // Verify result
    assert!(result.is_ok());
    let oauth_response = result.unwrap();
    assert!(oauth_response.ok);
    assert_eq!(
        oauth_response.access_token,
        Some("xoxb-mock-token-12345".to_string())
    );
    assert_eq!(oauth_response.team.as_ref().unwrap().id, "T789TEAM");
    assert_eq!(oauth_response.authed_user.as_ref().unwrap().id, "U012USER");
}

#[tokio::test]
async fn test_exchange_code_slack_error() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Create error response
    let response_body = serde_json::json!({
        "ok": false,
        "error": "invalid_code"
    });

    Mock::given(method("POST"))
        .and(path("/oauth.v2.access"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    // Create OAuth config
    let config = OAuthConfig {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        redirect_uri: "http://localhost:3000/callback".to_string(),
        bot_scopes: vec!["chat:write".to_string()],
        user_scopes: vec![],
    };

    // Exchange code
    let result = exchange_code(
        &config,
        "invalid_auth_code",
        "mock_code_verifier",
        Some(&mock_server.uri()),
    )
    .await;

    // Verify error
    assert!(result.is_err());
    match result {
        Err(slack_rs::oauth::OAuthError::SlackError(msg)) => {
            assert_eq!(msg, "invalid_code");
        }
        _ => panic!("Expected SlackError"),
    }
}

#[tokio::test]
async fn test_exchange_code_http_error() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Return 500 error
    Mock::given(method("POST"))
        .and(path("/oauth.v2.access"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    // Create OAuth config
    let config = OAuthConfig {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        redirect_uri: "http://localhost:3000/callback".to_string(),
        bot_scopes: vec!["chat:write".to_string()],
        user_scopes: vec![],
    };

    // Exchange code
    let result = exchange_code(
        &config,
        "auth_code",
        "code_verifier",
        Some(&mock_server.uri()),
    )
    .await;

    // Verify error
    assert!(result.is_err());
    match result {
        Err(slack_rs::oauth::OAuthError::HttpError(status, _)) => {
            assert_eq!(status, 500);
        }
        _ => panic!("Expected HttpError"),
    }
}
