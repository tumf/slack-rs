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

/// Regression test: Verify that files.info receives file parameter in correct form format (not JSON)
/// This prevents recurrence of the invalid_arguments error
#[tokio::test]
async fn test_file_download_with_file_id_sends_correct_params() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("regression_test.txt");

    let file_content = b"Regression test content";

    // Mock files.info endpoint - verify it receives form-encoded parameters
    // NOT JSON body (which would cause invalid_arguments)
    let mut files_info_response = HashMap::new();
    files_info_response.insert("ok".to_string(), serde_json::json!(true));
    files_info_response.insert(
        "file".to_string(),
        serde_json::json!({
            "id": "F123456789",
            "name": "regression.txt",
            "url_private_download": format!("{}/files-pri/download", mock_server.uri())
        }),
    );

    // This mock will ONLY match if the request has form-encoded body (not JSON)
    // The header matcher ensures Content-Type is application/x-www-form-urlencoded
    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock file download endpoint
    Mock::given(method("GET"))
        .and(path("/files-pri/download"))
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
        Some("F123456789".to_string()),
        None,
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(
        result.is_ok(),
        "Download should succeed when files.info receives form parameters: {:?}",
        result
    );

    let downloaded_content = std::fs::read(&output_path).unwrap();
    assert_eq!(downloaded_content, file_content);
}

/// Regression test: Verify that --url path downloads with Authorization header
/// This ensures direct URL downloads are authenticated properly
#[tokio::test]
async fn test_file_download_with_url_uses_authorization_header() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("url_download.txt");

    let file_content = b"Direct URL download content";

    // Mock the direct download endpoint
    // This mock will ONLY match if Authorization header is present
    Mock::given(method("GET"))
        .and(path("/files-pri-temp/direct-url-file"))
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
    let direct_url = format!("{}/files-pri-temp/direct-url-file", mock_server.uri());
    let result = commands::file_download(
        &client,
        None,
        Some(direct_url),
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(
        result.is_ok(),
        "Download should succeed when using --url with Authorization header: {:?}",
        result
    );

    let downloaded_content = std::fs::read(&output_path).unwrap();
    assert_eq!(downloaded_content, file_content);
}

/// Integration test: Download image file via file_id path with realistic fixture
#[tokio::test]
async fn test_file_download_image_via_file_id() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test_image.png");

    // Realistic PNG file header (1x1 transparent PNG)
    let png_content: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, // bit depth, color type, etc.
        0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, // IDAT chunk
        0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4,
        0x00, // image data
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, // IEND chunk
        0x42, 0x60, 0x82,
    ];

    // Mock files.info response for image
    let mut files_info_response = HashMap::new();
    files_info_response.insert("ok".to_string(), serde_json::json!(true));
    files_info_response.insert(
        "file".to_string(),
        serde_json::json!({
            "id": "F_IMG_123",
            "name": "screenshot.png",
            "mimetype": "image/png",
            "url_private_download": format!("{}/files-pri/img123", mock_server.uri())
        }),
    );

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("authorization", "Bearer test_token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock image download
    Mock::given(method("GET"))
        .and(path("/files-pri/img123"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(png_content)
                .insert_header("Content-Type", "image/png"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::file_download(
        &client,
        Some("F_IMG_123".to_string()),
        None,
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(
        result.is_ok(),
        "Image download via file_id should succeed: {:?}",
        result
    );

    let downloaded_content = std::fs::read(&output_path).unwrap();
    assert_eq!(downloaded_content, png_content);
    // Verify PNG signature
    assert_eq!(&downloaded_content[0..8], &png_content[0..8]);
}

/// Integration test: Download video file via --url path with realistic fixture
#[tokio::test]
async fn test_file_download_video_via_url() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test_video.mp4");

    // Realistic MP4 file header (minimal valid MP4)
    let mp4_content: &[u8] = &[
        0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, // ftyp box
        0x69, 0x73, 0x6F, 0x6D, 0x00, 0x00, 0x02, 0x00, // isom
        0x69, 0x73, 0x6F, 0x6D, 0x69, 0x73, 0x6F, 0x32, // compatible brands
        0x61, 0x76, 0x63, 0x31, 0x6D, 0x70, 0x34, 0x31, 0x00, 0x00, 0x00, 0x08, 0x66, 0x72, 0x65,
        0x65, // free box
    ];

    let video_url = format!("{}/files-pri-temp/video456", mock_server.uri());

    // Mock direct video download with Authorization header
    Mock::given(method("GET"))
        .and(path("/files-pri-temp/video456"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(mp4_content)
                .insert_header("Content-Type", "video/mp4"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());
    let result = commands::file_download(
        &client,
        None,
        Some(video_url),
        Some(output_path.to_str().unwrap().to_string()),
    )
    .await;

    assert!(
        result.is_ok(),
        "Video download via --url should succeed: {:?}",
        result
    );

    let downloaded_content = std::fs::read(&output_path).unwrap();
    assert_eq!(downloaded_content, mp4_content);
    // Verify MP4 ftyp box signature
    assert_eq!(&downloaded_content[4..8], b"ftyp");
}

/// Integration test: Both paths work with image file
#[tokio::test]
async fn test_file_download_image_both_paths() {
    let mock_server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();

    // Minimal JPEG header
    let jpeg_content: &[u8] = &[
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, // JPEG SOI and APP0
        0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF,
        0xD9, // JPEG EOI
    ];

    // Test 1: file_id path
    let output_path_1 = temp_dir.path().join("via_file_id.jpg");

    let mut files_info_response = HashMap::new();
    files_info_response.insert("ok".to_string(), serde_json::json!(true));
    files_info_response.insert(
        "file".to_string(),
        serde_json::json!({
            "id": "F_JPEG_789",
            "name": "photo.jpg",
            "mimetype": "image/jpeg",
            "url_private_download": format!("{}/files-pri/jpeg789", mock_server.uri())
        }),
    );

    Mock::given(method("POST"))
        .and(path("/files.info"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&files_info_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock the download endpoint - will be called twice (once for each path)
    Mock::given(method("GET"))
        .and(path("/files-pri/jpeg789"))
        .and(header("authorization", "Bearer test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(jpeg_content)
                .insert_header("Content-Type", "image/jpeg"),
        )
        .expect(2) // Expecting 2 calls: one from file_id path, one from --url path
        .mount(&mock_server)
        .await;

    let client = ApiClient::new_with_base_url("test_token".to_string(), mock_server.uri());

    // Test 1: file_id path
    let result_1 = commands::file_download(
        &client,
        Some("F_JPEG_789".to_string()),
        None,
        Some(output_path_1.to_str().unwrap().to_string()),
    )
    .await;

    assert!(result_1.is_ok(), "file_id path should succeed");
    let downloaded_1 = std::fs::read(&output_path_1).unwrap();
    assert_eq!(&downloaded_1[0..2], &[0xFF, 0xD8]); // JPEG SOI

    // Test 2: --url path
    let output_path_2 = temp_dir.path().join("via_url.jpg");
    let direct_url = format!("{}/files-pri/jpeg789", mock_server.uri());

    let result_2 = commands::file_download(
        &client,
        None,
        Some(direct_url),
        Some(output_path_2.to_str().unwrap().to_string()),
    )
    .await;

    assert!(result_2.is_ok(), "--url path should succeed");
    let downloaded_2 = std::fs::read(&output_path_2).unwrap();
    assert_eq!(
        downloaded_1, downloaded_2,
        "Both paths should download the same content"
    );
}
