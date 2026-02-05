//! Unified output envelope for all commands
//!
//! Provides a consistent output structure with response and metadata

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Unified command response with envelope
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse {
    /// Original API response
    pub response: Value,

    /// Execution metadata
    pub meta: CommandMeta,
}

/// Command execution metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandMeta {
    pub profile_name: Option<String>,
    pub team_id: String,
    pub user_id: String,
    pub method: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
}

impl CommandResponse {
    /// Create a new command response with metadata
    pub fn new(
        response: Value,
        profile_name: Option<String>,
        team_id: String,
        user_id: String,
        method: String,
        command: String,
    ) -> Self {
        Self {
            response,
            meta: CommandMeta {
                profile_name,
                team_id,
                user_id,
                method,
                command,
                token_type: None,
            },
        }
    }

    /// Create a new command response with metadata including token type
    pub fn with_token_type(
        response: Value,
        profile_name: Option<String>,
        team_id: String,
        user_id: String,
        method: String,
        command: String,
        token_type: Option<String>,
    ) -> Self {
        Self {
            response,
            meta: CommandMeta {
                profile_name,
                team_id,
                user_id,
                method,
                command,
                token_type,
            },
        }
    }
}
