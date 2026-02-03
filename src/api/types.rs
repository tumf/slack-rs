//! API types and structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Slack API method identifier
#[derive(Debug, Clone, PartialEq)]
pub enum ApiMethod {
    /// Search messages
    SearchMessages,
    /// List conversations
    ConversationsList,
    /// Get conversation history
    ConversationsHistory,
    /// Get user info
    UsersInfo,
    /// Post message
    ChatPostMessage,
    /// Update message
    ChatUpdate,
    /// Delete message
    ChatDelete,
    /// Add reaction
    ReactionsAdd,
    /// Remove reaction
    ReactionsRemove,
}

impl ApiMethod {
    /// Convert to Slack API method string
    pub fn as_str(&self) -> &str {
        match self {
            ApiMethod::SearchMessages => "search.messages",
            ApiMethod::ConversationsList => "conversations.list",
            ApiMethod::ConversationsHistory => "conversations.history",
            ApiMethod::UsersInfo => "users.info",
            ApiMethod::ChatPostMessage => "chat.postMessage",
            ApiMethod::ChatUpdate => "chat.update",
            ApiMethod::ChatDelete => "chat.delete",
            ApiMethod::ReactionsAdd => "reactions.add",
            ApiMethod::ReactionsRemove => "reactions.remove",
        }
    }

    /// Check if this is a write operation
    #[allow(dead_code)]
    pub fn is_write(&self) -> bool {
        matches!(
            self,
            ApiMethod::ChatPostMessage
                | ApiMethod::ChatUpdate
                | ApiMethod::ChatDelete
                | ApiMethod::ReactionsAdd
                | ApiMethod::ReactionsRemove
        )
    }

    /// Check if this is a destructive operation requiring confirmation
    #[allow(dead_code)]
    pub fn is_destructive(&self) -> bool {
        matches!(
            self,
            ApiMethod::ChatDelete | ApiMethod::ChatUpdate | ApiMethod::ReactionsRemove
        )
    }
}

/// API response with metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    /// Whether the request was successful
    pub ok: bool,
    /// Response data
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
    /// Error message if ok is false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ApiResponse {
    /// Create a successful response
    #[allow(dead_code)]
    pub fn success(data: HashMap<String, serde_json::Value>) -> Self {
        Self {
            ok: true,
            data,
            error: None,
        }
    }

    /// Create an error response
    #[allow(dead_code)]
    pub fn error(error: String) -> Self {
        Self {
            ok: false,
            data: HashMap::new(),
            error: Some(error),
        }
    }
}
