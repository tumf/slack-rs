//! Command implementations for Slack CLI wrapper commands
//!
//! Provides high-level commands that wrap the generic API client:
//! - search: Search messages
//! - conv: Conversation operations (list, history)
//! - users: User operations (info)
//! - users_cache: User cache and mention resolution
//! - msg: Message operations (post, update, delete)
//! - react: Reaction operations (add, remove)

pub mod conv;
pub mod guards;
pub mod msg;
pub mod react;
pub mod search;
pub mod users;
pub mod users_cache;

pub use conv::{conv_history, conv_list};
pub use msg::{msg_delete, msg_post, msg_update};
pub use react::{react_add, react_remove};
pub use search::search;
pub use users::users_info;
pub use users_cache::{resolve_mentions, update_cache, MentionFormat, UsersCacheFile};
