//! Integration tests for file download functionality
//!
//! Tests redirect following and HTML error response handling

use slack_rs::api::ApiClient;
use slack_rs::commands;
use std::collections::HashMap;
use tempfile::TempDir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test that file_download follows 302 redirects and preserves Authorization header
#[tokio::test]
async fn test_file_download_follows_302_redirect_same_host() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("downloaded_file.txt");

    let file_content = b"Hello, this is the file content!";

    // Mock files.info endpoint
    let mut files_info_response = HashMap::new();
    files_info_response.insert("ok".to_string(), serde_json::json!(true));
    files_info_response.insert(
        "file".to_string(),
        serde_json::json!({
            "id": "F123456",
            "name": "test_file.txt",
            "url_private_download": format!("{}/files-pri/initial", mock_server.uri())
        }),
    );

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock initial file download with 302 redirect
    Mock::given(method("GET"))
        .and(path("/files-pri/initial"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/files-pri/final", mock_server.uri())),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock final file download endpoint (should also receive Authorization header)
    Mock::given(method("GET"))
        .and(path("/files-pri/final"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(file_content)
                .insert_header("Content-Type", "application/octet-stream"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create API client and download file
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::file_download(
        &client,
        Some("F123456".to_string()),
        None,
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(result.is_ok(), "Download should succeed: {:?}", result);

    // Verify file was written correctly
    let downloaded_content = std::fs::read(&output_path).unwrap();
    assert_eq!(downloaded_content, file_content);
}

/// Test that file_download follows 302 redirects across different paths (relative redirect)
#[tokio::test]
async fn test_file_download_follows_302_redirect_relative() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("downloaded_file.txt");

    let file_content = b"Redirected file content";

    // Mock files.info endpoint
    let mut files_info_response = HashMap::new();
    files_info_response.insert("ok".to_string(), serde_json::json!(true));
    files_info_response.insert(
        "file".to_string(),
        serde_json::json!({
            "id": "F789012",
            "name": "test_file.txt",
            "url_private_download": format!("{}/files/start", mock_server.uri())
        }),
    );

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock redirect with relative Location header
    Mock::given(method("GET"))
        .and(path("/files/start"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(302).insert_header("Location", "/files/end"))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock final endpoint
    Mock::given(method("GET"))
        .and(path("/files/end"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(file_content)
                .insert_header("Content-Type", "text/plain"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::file_download(
        &client,
        Some("F789012".to_string()),
        None,
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(
        result.is_ok(),
        "Download should succeed after redirect: {:?}",
        result
    );

    let downloaded_content = std::fs::read(&output_path).unwrap();
    assert_eq!(downloaded_content, file_content);
}

/// Test that file_download returns diagnostic info when receiving HTML response (2xx status)
#[tokio::test]
async fn test_file_download_html_response_with_diagnostic_2xx() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("should_not_exist.txt");

    let html_body = r#"<!DOCTYPE html>
<html>
<head><title>Error: Authentication Required</title></head>
<body>
<h1>401 Unauthorized</h1>
<p>You must be authenticated to access this file.</p>
</body>
</html>"#;

    // Mock files.info endpoint
    let mut files_info_response = HashMap::new();
    files_info_response.insert("ok".to_string(), serde_json::json!(true));
    files_info_response.insert(
        "file".to_string(),
        serde_json::json!({
            "id": "F999999",
            "name": "secret.txt",
            "url_private_download": format!("{}/files-pri/secret", mock_server.uri())
        }),
    );

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock file download returning HTML (common error case)
    // Use set_body_bytes and manually set Content-Type header
    Mock::given(method("GET"))
        .and(path("/files-pri/secret"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(html_body.as_bytes())
                .append_header("Content-Type", "text/html; charset=utf-8"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::file_download(
        &client,
        Some("F999999".to_string()),
        None,
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(
        result.is_err(),
        "Download should fail when HTML is returned. Got: {:?}",
        result
    );

    let error_msg = format!("{:?}", result.unwrap_err());

    // Verify error message contains diagnostic info
    assert!(
        error_msg.contains("HTML instead of file"),
        "Error should mention HTML response"
    );
    assert!(
        error_msg.contains("Wrong URL") || error_msg.contains("url_private_download"),
        "Error should mention URL issue"
    );
    assert!(
        error_msg.contains("authentication") || error_msg.contains("scopes"),
        "Error should mention authentication"
    );
    assert!(
        error_msg.contains("Response snippet"),
        "Error should include response snippet"
    );
    // Verify snippet is truncated and contains part of the HTML
    assert!(
        error_msg.contains("DOCTYPE") || error_msg.contains("Unauthorized"),
        "Error should include snippet from HTML body"
    );

    // Verify no file was written
    assert!(
        !output_path.exists(),
        "File should not be written on HTML error"
    );
}

/// Test that file_download returns diagnostic info for HTML response with non-2xx status
#[tokio::test]
async fn test_file_download_html_response_with_diagnostic_401() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("should_not_exist.txt");

    let html_body = r#"<!DOCTYPE html>
<html>
<head><title>401 Unauthorized</title></head>
<body>
<h1>Authentication Failed</h1>
<p>Invalid or missing token.</p>
</body>
</html>"#;

    // Mock files.info endpoint
    let mut files_info_response = HashMap::new();
    files_info_response.insert("ok".to_string(), serde_json::json!(true));
    files_info_response.insert(
        "file".to_string(),
        serde_json::json!({
            "id": "F888888",
            "name": "protected.txt",
            "url_private_download": format!("{}/files-pri/protected", mock_server.uri())
        }),
    );

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock file download returning 401 HTML response
    Mock::given(method("GET"))
        .and(path("/files-pri/protected"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_bytes(html_body.as_bytes())
                .append_header("Content-Type", "text/html"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::file_download(
        &client,
        Some("F888888".to_string()),
        None,
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(result.is_err(), "Download should fail on 401 HTML response");

    let error_msg = format!("{:?}", result.unwrap_err());

    // Verify error message includes status code
    assert!(
        error_msg.contains("401") || error_msg.contains("status"),
        "Error should mention HTTP status"
    );

    // Verify diagnostic information is present even for non-2xx
    assert!(
        error_msg.contains("HTML instead of file"),
        "Error should identify HTML response"
    );
    assert!(
        error_msg.contains("Response snippet"),
        "Error should include diagnostic snippet even for non-2xx"
    );
    assert!(
        error_msg.contains("Unauthorized") || error_msg.contains("Authentication"),
        "Error should include snippet from HTML body"
    );

    // Verify no file was written
    assert!(!output_path.exists(), "File should not be written on error");
}

/// Test that file_download handles multiple redirects (chain)
#[tokio::test]
async fn test_file_download_handles_redirect_chain() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("chained.txt");

    let file_content = b"Final content after redirect chain";

    // Mock files.info endpoint
    let mut files_info_response = HashMap::new();
    files_info_response.insert("ok".to_string(), serde_json::json!(true));
    files_info_response.insert(
        "file".to_string(),
        serde_json::json!({
            "id": "F111111",
            "name": "chained.txt",
            "url_private_download": format!("{}/files/step1", mock_server.uri())
        }),
    );

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Redirect chain: step1 -> step2 -> step3 -> final
    Mock::given(method("GET"))
        .and(path("/files/step1"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(302).insert_header("Location", "/files/step2"))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/files/step2"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(302).insert_header("Location", "/files/step3"))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/files/step3"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(302).insert_header("Location", "/files/final"))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/files/final"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(file_content)
                .insert_header("Content-Type", "application/octet-stream"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::file_download(
        &client,
        Some("F111111".to_string()),
        None,
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(result.is_ok(), "Should handle redirect chain: {:?}", result);

    let downloaded_content = std::fs::read(&output_path).unwrap();
    assert_eq!(downloaded_content, file_content);
}

/// Test that file_download rejects too many redirects
#[tokio::test]
async fn test_file_download_rejects_excessive_redirects() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("never_exists.txt");

    // Mock files.info endpoint
    let mut files_info_response = HashMap::new();
    files_info_response.insert("ok".to_string(), serde_json::json!(true));
    files_info_response.insert(
        "file".to_string(),
        serde_json::json!({
            "id": "F222222",
            "name": "loop.txt",
            "url_private_download": format!("{}/files/loop", mock_server.uri())
        }),
    );

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock infinite redirect loop (redirects to itself)
    Mock::given(method("GET"))
        .and(path("/files/loop"))
        .respond_with(ResponseTemplate::new(302).insert_header("Location", "/files/loop"))
        .expect(11) // Should stop after MAX_REDIRECTS (10) + 1 initial request
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::file_download(
        &client,
        Some("F222222".to_string()),
        None,
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(result.is_err(), "Should fail on too many redirects");

    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("Too many redirects") || error_msg.contains("max"),
        "Error should mention redirect limit"
    );
}

/// Test that file_download preserves Authorization header when redirecting to a different host
#[tokio::test]
async fn test_file_download_follows_redirect_to_different_host() {
    // Start two separate mock servers to simulate cross-host redirect
    let mock_server_1 = MockServer::start().await;
    let mock_server_2 = MockServer::start().await;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("cross_host_file.txt");

    let file_content = b"Content from different host after redirect";

    // Mock files.info endpoint on first server
    let mut files_info_response = HashMap::new();
    files_info_response.insert("ok".to_string(), serde_json::json!(true));
    files_info_response.insert(
        "file".to_string(),
        serde_json::json!({
            "id": "F333333",
            "name": "cross_host.txt",
            "url_private_download": format!("{}/files-pri/initial", mock_server_1.uri())
        }),
    );

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server_1)
        .await;

    // Mock initial download on server 1 - redirects to server 2
    Mock::given(method("GET"))
        .and(path("/files-pri/initial"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(ResponseTemplate::new(302).insert_header(
            "Location",
            format!("{}/files-pri/final-on-different-host", mock_server_2.uri()),
        ))
        .expect(1)
        .mount(&mock_server_1)
        .await;

    // Mock final download on server 2 - MUST receive Authorization header
    Mock::given(method("GET"))
        .and(path("/files-pri/final-on-different-host"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(file_content)
                .insert_header("Content-Type", "application/octet-stream"),
        )
        .expect(1)
        .mount(&mock_server_2)
        .await;

    // Create client pointing to the first server
    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server_1.uri());
    let result = commands::file_download(
        &client,
        Some("F333333".to_string()),
        None,
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(
        result.is_ok(),
        "Download should succeed after cross-host redirect: {:?}",
        result
    );

    // Verify file was written correctly with content from server 2
    let downloaded_content = std::fs::read(&output_path).unwrap();
    assert_eq!(
        downloaded_content, file_content,
        "Downloaded content should match file from second server"
    );
}
