//! Search command implementation

use crate::api::{ApiClient, ApiError, ApiMethod, ApiResponse};
use serde_json::json;
use std::collections::HashMap;

/// Search messages in Slack
///
/// # Arguments
/// * `client` - API client
/// * `query` - Search query string
/// * `count` - Optional number of results to return (default: 20)
/// * `page` - Optional page number (default: 1)
///
/// # Returns
/// * `Ok(ApiResponse)` with search results
/// * `Err(ApiError)` if the operation fails
pub async fn search(
    client: &ApiClient,
    query: String,
    count: Option<u32>,
    page: Option<u32>,
) -> Result<ApiResponse, ApiError> {
    let mut params = HashMap::new();
    params.insert("query".to_string(), json!(query));

    if let Some(count) = count {
        params.insert("count".to_string(), json!(count));
    }

    if let Some(page) = page {
        params.insert("page".to_string(), json!(page));
    }

    client.call(ApiMethod::SearchMessages, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_basic() {
        // This test requires a mock server to be implemented
        // For now, we just verify the function compiles
        let client = ApiClient::new("test_token".to_string());
        let result = search(&client, "test query".to_string(), None, None).await;
        // Result will fail because there's no mock server, but that's expected
        assert!(result.is_err());
    }
}
