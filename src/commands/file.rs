//! File upload command implementations using external upload method
//!
//! Implements Slack's recommended external upload flow:
//! 1. Call files.getUploadURLExternal to get upload_url and file_id
//! 2. POST raw file bytes to upload_url (not a Slack API endpoint)
//! 3. Call files.completeUploadExternal to finalize and share the file

use crate::api::{ApiClient, ApiError};
use crate::commands::guards::check_write_allowed;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

/// Response from files.getUploadURLExternal
#[derive(Debug, Deserialize)]
struct GetUploadUrlResponse {
    ok: bool,
    upload_url: Option<String>,
    file_id: Option<String>,
    error: Option<String>,
}

/// Response from files.completeUploadExternal
#[derive(Debug, Deserialize, Serialize)]
struct CompleteUploadResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Upload a file using external upload flow
///
/// # Arguments
/// * `client` - API client with token
/// * `file_path` - Path to file to upload
/// * `channels` - Optional channel IDs to share to (comma-separated)
/// * `title` - Optional file title
/// * `comment` - Optional initial comment
///
/// # Returns
/// * `Ok(serde_json::Value)` with upload result
/// * `Err(ApiError)` if the operation fails
pub async fn file_upload(
    client: &ApiClient,
    file_path: String,
    channels: Option<String>,
    title: Option<String>,
    comment: Option<String>,
) -> Result<serde_json::Value, ApiError> {
    check_write_allowed()?;

    // Step 1: Read file and get metadata
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(ApiError::SlackError(format!(
            "File not found: {}",
            file_path
        )));
    }

    let file_bytes = std::fs::read(path)
        .map_err(|e| ApiError::SlackError(format!("Failed to read file {}: {}", file_path, e)))?;

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    let file_length = file_bytes.len();

    // Step 2: Get upload URL
    let mut params = HashMap::new();
    params.insert("filename".to_string(), json!(file_name));
    params.insert("length".to_string(), json!(file_length));

    // Call files.getUploadURLExternal using the base_url from ApiClient
    let url = format!("{}/files.getUploadURLExternal", client.base_url());
    let token = client
        .token
        .as_ref()
        .ok_or_else(|| ApiError::SlackError("No token configured".to_string()))?;

    let http_client = Client::new();
    let get_url_response = http_client
        .post(&url)
        .bearer_auth(token)
        .json(&params)
        .send()
        .await
        .map_err(|e| ApiError::SlackError(format!("Failed to get upload URL: {}", e)))?;

    let get_url_result: GetUploadUrlResponse = get_url_response
        .json()
        .await
        .map_err(|e| ApiError::SlackError(format!("Failed to parse upload URL response: {}", e)))?;

    if !get_url_result.ok {
        return Err(ApiError::SlackError(format!(
            "files.getUploadURLExternal failed: {}",
            get_url_result
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        )));
    }

    let upload_url = get_url_result
        .upload_url
        .ok_or_else(|| ApiError::SlackError("No upload_url in response".to_string()))?;

    let file_id = get_url_result
        .file_id
        .ok_or_else(|| ApiError::SlackError("No file_id in response".to_string()))?;

    // Step 3: Upload file bytes to external URL
    let upload_response = http_client
        .post(&upload_url)
        .header("Content-Type", "application/octet-stream")
        .body(file_bytes)
        .send()
        .await
        .map_err(|e| ApiError::SlackError(format!("Failed to upload file: {}", e)))?;

    if !upload_response.status().is_success() {
        return Err(ApiError::SlackError(format!(
            "File upload failed with status: {}",
            upload_response.status()
        )));
    }

    // Step 4: Complete the upload
    let mut complete_params = HashMap::new();

    // Create files array with single file
    let file_upload = json!({
        "id": file_id,
        "title": title.unwrap_or_else(|| file_name.clone())
    });
    complete_params.insert("files".to_string(), json!([file_upload]));

    // Add optional parameters
    if let Some(ch) = channels {
        complete_params.insert("channel_id".to_string(), json!(ch));
    }
    if let Some(cmt) = comment {
        complete_params.insert("initial_comment".to_string(), json!(cmt));
    }

    let complete_url = format!("{}/files.completeUploadExternal", client.base_url());
    let complete_response = http_client
        .post(&complete_url)
        .bearer_auth(token)
        .json(&complete_params)
        .send()
        .await
        .map_err(|e| ApiError::SlackError(format!("Failed to complete upload: {}", e)))?;

    let complete_result: CompleteUploadResponse = complete_response
        .json()
        .await
        .map_err(|e| ApiError::SlackError(format!("Failed to parse complete response: {}", e)))?;

    if !complete_result.ok {
        return Err(ApiError::SlackError(format!(
            "files.completeUploadExternal failed: {}",
            complete_result
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        )));
    }

    // Return the complete result as JSON
    serde_json::to_value(complete_result)
        .map_err(|e| ApiError::SlackError(format!("Failed to serialize result: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial(write_guard)]
    async fn test_file_upload_write_not_allowed() {
        // Set env var to deny write
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
        let client = ApiClient::with_token("test_token".to_string());
        let result = file_upload(&client, "/tmp/test.txt".to_string(), None, None, None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }

    #[tokio::test]
    #[serial(write_guard)]
    async fn test_file_upload_nonexistent_file() {
        // Ensure write is allowed
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
        let client = ApiClient::with_token("test_token".to_string());
        let result = file_upload(
            &client,
            "/nonexistent/file.txt".to_string(),
            None,
            None,
            None,
        )
        .await;
        assert!(result.is_err());
        if let Err(ApiError::SlackError(msg)) = result {
            assert!(msg.contains("File not found"));
        } else {
            panic!("Expected SlackError with 'File not found'");
        }
    }
}
