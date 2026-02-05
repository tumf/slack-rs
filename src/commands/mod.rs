//! Command implementations for Slack CLI wrapper commands
//!
//! Provides high-level commands that wrap the generic API client:
//! - search: Search messages
//! - conv: Conversation operations (list, history)
//! - users: User operations (info)
//! - users_cache: User cache and mention resolution
//! - msg: Message operations (post, update, delete)
//! - react: Reaction operations (add, remove)
//! - file: File operations (upload using external upload method)
//! - config: Configuration management (OAuth settings)

pub mod config;
pub mod conv;
pub mod file;
pub mod guards;
pub mod msg;
pub mod react;
pub mod search;
pub mod users;
pub mod users_cache;

pub use config::{oauth_delete, oauth_set, oauth_show, set_default_token_type};
pub use conv::{
    apply_filters, conv_history, conv_list, extract_conversations, format_response,
    sort_conversations, ConversationFilter, ConversationItem, ConversationSelector, OutputFormat,
    SortDirection, SortKey, StdinSelector,
};
pub use file::file_upload;
pub use msg::{msg_delete, msg_post, msg_update};
pub use react::{react_add, react_remove};
pub use search::search;
pub use users::users_info;
pub use users_cache::{resolve_mentions, update_cache, MentionFormat, UsersCacheFile};
