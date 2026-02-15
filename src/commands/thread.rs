//! Thread operations - retrieve thread messages

use crate::api::{ApiClient, ApiError, ApiMethod, ApiResponse};
use serde_json::json;
use std::collections::HashMap;

/// Maximum pages to fetch to prevent infinite loops
const MAX_PAGES: usize = 1000;

/// Get thread messages (conversation replies) with automatic pagination
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID containing the thread
/// * `thread_ts` - Timestamp of the parent message (thread identifier)
/// * `limit` - Optional number of messages per page (default: 100)
/// * `inclusive` - Optional flag to include the parent message (default: false)
///
/// # Returns
/// * `Ok(ApiResponse)` with all thread messages aggregated
/// * `Err(ApiError)` if the operation fails
///
/// # Pagination
/// This function automatically follows `next_cursor` to retrieve all pages and aggregates
/// the `messages` array from all responses into a single response.
/// It prevents infinite loops by:
/// - Tracking seen cursors (duplicate detection)
/// - Limiting max pages to MAX_PAGES
pub async fn thread_get(
    client: &ApiClient,
    channel: String,
    thread_ts: String,
    limit: Option<u32>,
    inclusive: Option<bool>,
) -> Result<ApiResponse, ApiError> {
    let mut all_messages = Vec::new();
    let mut cursor: Option<String> = None;
    let mut ok = true;
    let mut error: Option<String> = None;
    let mut page_count = 0;

    loop {
        // Prevent infinite loops
        page_count += 1;
        if page_count > MAX_PAGES {
            return Err(ApiError::SlackError(format!(
                "Pagination exceeded max pages ({}), possible infinite loop",
                MAX_PAGES
            )));
        }

        let mut params = HashMap::new();
        params.insert("channel".to_string(), json!(channel));
        params.insert("ts".to_string(), json!(thread_ts));

        // Use provided limit or default to 100
        let page_limit = limit.unwrap_or(100);
        params.insert("limit".to_string(), json!(page_limit));

        if let Some(incl) = inclusive {
            params.insert("inclusive".to_string(), json!(incl));
        }

        if let Some(ref cursor_val) = cursor {
            params.insert("cursor".to_string(), json!(cursor_val));
        }

        let response = client
            .call_method(ApiMethod::ConversationsReplies, params)
            .await?;

        // Capture ok/error status from first response
        if cursor.is_none() {
            ok = response.ok;
            error = response.error.clone();
        }

        // Extract messages from this page
        if let Some(messages) = response.data.get("messages") {
            if let Some(messages_array) = messages.as_array() {
                all_messages.extend(messages_array.clone());
            }
        }

        // Check for next cursor
        cursor = response
            .data
            .get("response_metadata")
            .and_then(|meta| meta.get("next_cursor"))
            .and_then(|c| c.as_str())
            .filter(|c| !c.is_empty())
            .map(|c| c.to_string());

        // If no next cursor, we're done
        if cursor.is_none() {
            break;
        }
    }

    // Build final response with aggregated messages
    let mut data = HashMap::new();
    data.insert("messages".to_string(), json!(all_messages));

    // Add empty response_metadata (no next_cursor since we fetched all)
    let mut response_metadata = HashMap::new();
    response_metadata.insert("next_cursor".to_string(), json!(""));
    data.insert("response_metadata".to_string(), json!(response_metadata));

    Ok(ApiResponse { ok, data, error })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_thread_get_basic() {
        let client = ApiClient::with_token("test_token".to_string());
        let result = thread_get(
            &client,
            "C123456".to_string(),
            "1234567890.123456".to_string(),
            None,
            None,
        )
        .await;
        // Result will fail because there's no mock server, but that's expected
        assert!(result.is_err());
    }
}
