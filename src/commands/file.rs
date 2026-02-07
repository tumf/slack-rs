//! File upload command implementations using external upload method
//!
//! Implements Slack's recommended external upload flow:
//! 1. Call files.getUploadURLExternal to get upload_url and file_id
//! 2. POST raw file bytes to upload_url (not a Slack API endpoint)
//! 3. Call files.completeUploadExternal to finalize and share the file

use crate::api::{ApiClient, ApiError};
use crate::commands::guards::{check_write_allowed, confirm_destructive_with_hint};
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
/// * `yes` - Skip confirmation prompt
/// * `non_interactive` - Whether running in non-interactive mode
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
    yes: bool,
    non_interactive: bool,
) -> Result<serde_json::Value, ApiError> {
    check_write_allowed()?;

    // Build hint with example command for non-interactive mode
    let hint = format!("Example: slack-rs file upload {} --yes", file_path);
    confirm_destructive_with_hint(yes, "upload this file", non_interactive, Some(&hint))?;

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

/// Response from files.info
#[derive(Debug, Deserialize)]
struct FilesInfoResponse {
    ok: bool,
    file: Option<FileInfo>,
    error: Option<String>,
}

/// File information from files.info
#[derive(Debug, Deserialize)]
struct FileInfo {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    url_private_download: Option<String>,
    #[serde(default)]
    url_private: Option<String>,
}

/// Download a file from Slack
///
/// # Arguments
/// * `client` - API client with token
/// * `file_id` - Optional file ID to download (calls files.info to get URL)
/// * `url` - Optional direct URL to download
/// * `out` - Optional output path ("-" for stdout, directory for auto-naming, file path for specific output)
///
/// # Returns
/// * `Ok(serde_json::Value)` with download result metadata
/// * `Err(ApiError)` if the operation fails
pub async fn file_download(
    client: &ApiClient,
    file_id: Option<String>,
    url: Option<String>,
    out: Option<String>,
) -> Result<serde_json::Value, ApiError> {
    let http_client = Client::new();
    let token = client
        .token
        .as_ref()
        .ok_or_else(|| ApiError::SlackError("No token configured".to_string()))?;

    // Resolve download URL and filename
    let (download_url, filename_hint) = if let Some(fid) = file_id {
        // Call files.info to get download URL
        let info_url = format!("{}/files.info", client.base_url());
        let mut params = HashMap::new();
        params.insert("file".to_string(), json!(fid));

        let info_response = http_client
            .post(&info_url)
            .bearer_auth(token)
            .json(&params)
            .send()
            .await
            .map_err(|e| ApiError::SlackError(format!("Failed to call files.info: {}", e)))?;

        let info_result: FilesInfoResponse = info_response
            .json()
            .await
            .map_err(|e| ApiError::SlackError(format!("Failed to parse files.info response: {}", e)))?;

        if !info_result.ok {
            return Err(ApiError::SlackError(format!(
                "files.info failed: {}",
                info_result.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        let file = info_result.file.ok_or_else(|| {
            ApiError::SlackError("No file information in files.info response".to_string())
        })?;

        // Prefer url_private_download, fallback to url_private
        let url = file
            .url_private_download
            .or(file.url_private)
            .ok_or_else(|| ApiError::SlackError("No download URL found in file info".to_string()))?;

        let name = file.name.unwrap_or_else(|| format!("file-{}", fid));
        (url, name)
    } else if let Some(direct_url) = url {
        // Use provided URL directly
        let name = direct_url
            .rsplit('/')
            .next()
            .unwrap_or("downloaded-file")
            .to_string();
        (direct_url, name)
    } else {
        return Err(ApiError::SlackError(
            "Either file_id or url must be provided".to_string(),
        ));
    };

    // Download the file
    let download_response = http_client
        .get(&download_url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| ApiError::SlackError(format!("Failed to download file: {}", e)))?;

    // Check response status
    if !download_response.status().is_success() {
        return Err(ApiError::SlackError(format!(
            "Download failed with status: {}",
            download_response.status()
        )));
    }

    // Check Content-Type for HTML response (indicates wrong URL or auth issue)
    if let Some(content_type) = download_response.headers().get("content-type") {
        if let Ok(ct_str) = content_type.to_str() {
            if ct_str.contains("text/html") {
                return Err(ApiError::SlackError(
                    "Download returned HTML instead of file. Possible causes:\n\
                     - Wrong URL: Make sure to use url_private_download, not permalink\n\
                     - Missing authentication: Token may lack required scopes\n\
                     - Invalid or expired file"
                        .to_string(),
                ));
            }
        }
    }

    // Get response bytes
    let bytes = download_response
        .bytes()
        .await
        .map_err(|e| ApiError::SlackError(format!("Failed to read response body: {}", e)))?;

    // Handle output
    let output_path = match out.as_deref() {
        Some("-") => {
            // Write to stdout
            use std::io::Write;
            std::io::stdout()
                .write_all(&bytes)
                .map_err(|e| ApiError::SlackError(format!("Failed to write to stdout: {}", e)))?;
            "-".to_string()
        }
        Some(path) => {
            // Write to file
            let target_path = if Path::new(path).is_dir() {
                // Directory: auto-generate filename
                Path::new(path).join(sanitize_filename(&filename_hint))
            } else {
                // File path
                Path::new(path).to_path_buf()
            };

            std::fs::write(&target_path, &bytes).map_err(|e| {
                ApiError::SlackError(format!(
                    "Failed to write file to {}: {}",
                    target_path.display(),
                    e
                ))
            })?;

            target_path.display().to_string()
        }
        None => {
            // Default: current directory with sanitized filename
            let target_path = Path::new(".").join(sanitize_filename(&filename_hint));
            std::fs::write(&target_path, &bytes).map_err(|e| {
                ApiError::SlackError(format!(
                    "Failed to write file to {}: {}",
                    target_path.display(),
                    e
                ))
            })?;

            target_path.display().to_string()
        }
    };

    // Return metadata
    Ok(json!({
        "ok": true,
        "output": output_path,
        "size": bytes.len(),
        "url": download_url
    }))
}

/// Sanitize filename by replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
    let sanitized: String = name
        .chars()
        .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
        .collect();

    // Ensure non-empty
    if sanitized.is_empty() {
        "file".to_string()
    } else {
        sanitized
    }
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
        let result = file_upload(
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
            true,
            false,
        )
        .await;
        assert!(result.is_err());
        if let Err(ApiError::SlackError(msg)) = result {
            assert!(msg.contains("File not found"));
        } else {
            panic!("Expected SlackError with 'File not found'");
        }
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test.txt"), "test.txt");
        assert_eq!(sanitize_filename("test/file.txt"), "test_file.txt");
        assert_eq!(sanitize_filename("test:file.txt"), "test_file.txt");
        assert_eq!(sanitize_filename("test*file?.txt"), "test_file_.txt");
        assert_eq!(sanitize_filename(""), "file");
    }

    #[tokio::test]
    #[serial(write_guard)]
    async fn test_file_download_write_allowed() {
        // Ensure write is NOT allowed
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");

        // file_download should NOT check SLACKCLI_ALLOW_WRITE (read operation)
        let client = ApiClient::with_token("test_token".to_string());
        
        // This would fail with network error (no mock server), but NOT with WriteNotAllowed
        let result = file_download(
            &client,
            Some("F123456".to_string()),
            None,
            Some("/tmp/test_download.txt".to_string()),
        )
        .await;

        // Clean up env
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");

        // Should fail with network/API error, not WriteNotAllowed
        assert!(result.is_err());
        if let Err(e) = result {
            // Should NOT be WriteNotAllowed
            assert!(!matches!(e, ApiError::WriteNotAllowed));
        }
    }
}
