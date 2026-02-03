//! Profile and token storage management
//!
//! This module provides the foundation for managing Slack CLI profiles and tokens:
//! - Profile configuration (non-secret information)
//! - Secure token storage (using OS keyring)
//! - Profile resolution (profile_name -> team_id, user_id)
//!
//! This is a foundational library module intended for use by future CLI features.
//! Dead code and unused import warnings are suppressed as these components will be used by upcoming features.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod resolver;
pub mod storage;
pub mod token_store;
pub mod types;

// Re-export commonly used types and functions
pub use resolver::{list_profiles, resolve_profile, resolve_profile_full, ResolverError};
pub use storage::{default_config_path, load_config, save_config, StorageError};
pub use token_store::{
    make_token_key, InMemoryTokenStore, KeyringTokenStore, TokenStore, TokenStoreError,
};
pub use types::{Profile, ProfilesConfig};
