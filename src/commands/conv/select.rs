//! Interactive selection functionality for conversations

use crate::api::ApiResponse;
use serde::{Deserialize, Serialize};

/// Conversation item for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationItem {
    pub id: String,
    pub name: String,
    pub is_private: bool,
}

impl ConversationItem {
    /// Format for display in selection UI
    pub fn display(&self) -> String {
        let privacy = if self.is_private { " [private]" } else { "" };
        format!("#{} ({}){}", self.name, self.id, privacy)
    }
}

/// Extract conversation items from API response
pub fn extract_conversations(response: &ApiResponse) -> Vec<ConversationItem> {
    let mut items = Vec::new();

    if let Some(channels) = response.data.get("channels") {
        if let Some(channels_array) = channels.as_array() {
            for conv in channels_array {
                if let (Some(id), Some(name)) = (
                    conv.get("id").and_then(|v| v.as_str()),
                    conv.get("name").and_then(|v| v.as_str()),
                ) {
                    let is_private = conv
                        .get("is_private")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    items.push(ConversationItem {
                        id: id.to_string(),
                        name: name.to_string(),
                        is_private,
                    });
                }
            }
        }
    }

    items
}

/// Trait for interactive selection UI (allows for stubbing in tests)
pub trait ConversationSelector {
    /// Select a conversation from a list
    fn select(&self, items: &[ConversationItem]) -> Result<String, String>;
}

/// Default implementation using stdin
pub struct StdinSelector;

impl ConversationSelector for StdinSelector {
    fn select(&self, items: &[ConversationItem]) -> Result<String, String> {
        if items.is_empty() {
            return Err("No conversations available".to_string());
        }

        println!("Select a conversation:");
        for (i, item) in items.iter().enumerate() {
            println!("  {}: {}", i + 1, item.display());
        }
        println!("Enter number (or 0 to cancel): ");

        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let mut line = String::new();
        stdin
            .lock()
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;

        let choice: usize = line
            .trim()
            .parse()
            .map_err(|_| "Invalid number".to_string())?;

        if choice == 0 {
            return Err("Selection cancelled".to_string());
        }

        if choice > items.len() {
            return Err("Invalid selection".to_string());
        }

        Ok(items[choice - 1].id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_extract_conversations() {
        let response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "general", "is_private": false},
                    {"id": "C2", "name": "private", "is_private": true},
                ]),
            )]),
            error: None,
        };

        let items = extract_conversations(&response);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "C1");
        assert_eq!(items[0].name, "general");
        assert!(!items[0].is_private);
        assert_eq!(items[1].id, "C2");
        assert_eq!(items[1].name, "private");
        assert!(items[1].is_private);
    }

    #[test]
    fn test_conversation_item_display() {
        let item = ConversationItem {
            id: "C123".to_string(),
            name: "general".to_string(),
            is_private: false,
        };
        assert_eq!(item.display(), "#general (C123)");

        let item = ConversationItem {
            id: "C456".to_string(),
            name: "secret".to_string(),
            is_private: true,
        };
        assert_eq!(item.display(), "#secret (C456) [private]");
    }

    // Mock selector for testing
    pub struct MockSelector {
        pub selected_index: usize,
    }

    impl ConversationSelector for MockSelector {
        fn select(&self, items: &[ConversationItem]) -> Result<String, String> {
            if items.is_empty() {
                return Err("No conversations available".to_string());
            }
            if self.selected_index >= items.len() {
                return Err("Invalid selection".to_string());
            }
            Ok(items[self.selected_index].id.clone())
        }
    }

    #[test]
    fn test_mock_selector() {
        let items = vec![
            ConversationItem {
                id: "C1".to_string(),
                name: "general".to_string(),
                is_private: false,
            },
            ConversationItem {
                id: "C2".to_string(),
                name: "random".to_string(),
                is_private: false,
            },
        ];

        let selector = MockSelector { selected_index: 0 };
        assert_eq!(selector.select(&items).unwrap(), "C1");

        let selector = MockSelector { selected_index: 1 };
        assert_eq!(selector.select(&items).unwrap(), "C2");
    }
}
