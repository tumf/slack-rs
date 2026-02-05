//! Conversation command implementations

// Module declarations
pub mod api;
pub mod filter;
pub mod format;
pub mod select;
pub mod sort;

// Re-export public API to maintain backward compatibility
pub use api::{conv_history, conv_list};
pub use filter::{apply_filters, ConversationFilter, FilterError};
pub use format::{format_response, OutputFormat};
pub use select::{extract_conversations, ConversationItem, ConversationSelector, StdinSelector};
pub use sort::{sort_conversations, SortDirection, SortKey};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ApiResponse;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_filter_then_sort() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "test-zebra", "is_member": true},
                    {"id": "C2", "name": "test-alpha", "is_member": true},
                    {"id": "C3", "name": "other", "is_member": true},
                    {"id": "C4", "name": "test-beta", "is_member": false},
                ]),
            )]),
            error: None,
        };

        // Apply filter (name:test*, is_member:true)
        let filters = vec![
            ConversationFilter::Name("test*".to_string()),
            ConversationFilter::IsMember(true),
        ];
        apply_filters(&mut response, &filters);

        // Apply sort (name, asc)
        sort_conversations(&mut response, SortKey::Name, SortDirection::Asc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(channels.len(), 2); // C1 and C2
        assert_eq!(
            channels[0].get("name").unwrap().as_str().unwrap(),
            "test-alpha"
        );
        assert_eq!(
            channels[1].get("name").unwrap().as_str().unwrap(),
            "test-zebra"
        );
    }
}
