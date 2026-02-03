//! Reaction command implementations

use crate::api::{ApiClient, ApiError, ApiMethod, ApiResponse};
use crate::commands::guards::{check_write_allowed, confirm_destructive};
use serde_json::json;
use std::collections::HashMap;

/// Add a reaction to a message
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID
/// * `timestamp` - Message timestamp
/// * `name` - Emoji name (without colons, e.g., "thumbsup")
/// * `allow_write` - Whether write operations are allowed
///
/// # Returns
/// * `Ok(ApiResponse)` with reaction confirmation
/// * `Err(ApiError)` if the operation fails
pub async fn react_add(
    client: &ApiClient,
    channel: String,
    timestamp: String,
    name: String,
    allow_write: bool,
) -> Result<ApiResponse, ApiError> {
    check_write_allowed(allow_write)?;

    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));
    params.insert("timestamp".to_string(), json!(timestamp));
    params.insert("name".to_string(), json!(name));

    client.call(ApiMethod::ReactionsAdd, params).await
}

/// Remove a reaction from a message
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID
/// * `timestamp` - Message timestamp
/// * `name` - Emoji name (without colons, e.g., "thumbsup")
/// * `allow_write` - Whether write operations are allowed
/// * `yes` - Skip confirmation prompt
///
/// # Returns
/// * `Ok(ApiResponse)` with removal confirmation
/// * `Err(ApiError)` if the operation fails
pub async fn react_remove(
    client: &ApiClient,
    channel: String,
    timestamp: String,
    name: String,
    allow_write: bool,
    yes: bool,
) -> Result<ApiResponse, ApiError> {
    check_write_allowed(allow_write)?;
    confirm_destructive(yes, "remove this reaction")?;

    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));
    params.insert("timestamp".to_string(), json!(timestamp));
    params.insert("name".to_string(), json!(name));

    client.call(ApiMethod::ReactionsRemove, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_react_add_without_write_flag() {
        let client = ApiClient::new("test_token".to_string());
        let result = react_add(
            &client,
            "C123456".to_string(),
            "1234567890.123456".to_string(),
            "thumbsup".to_string(),
            false,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
    }

    #[tokio::test]
    async fn test_react_remove_without_write_flag() {
        let client = ApiClient::new("test_token".to_string());
        let result = react_remove(
            &client,
            "C123456".to_string(),
            "1234567890.123456".to_string(),
            "thumbsup".to_string(),
            false,
            true,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
    }
}
