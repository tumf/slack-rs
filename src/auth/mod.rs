//! Auth commands for Slack CLI
//!
//! Provides commands for managing authentication:
//! - login: Perform OAuth authentication
//! - status: Show current profile status
//! - list: List all profiles
//! - rename: Rename a profile
//! - logout: Remove authentication
//! - export: Export profiles to encrypted file
//! - import: Import profiles from encrypted file

pub mod commands;
pub mod crypto;
pub mod export_import;
pub mod format;
pub mod i18n;

pub use commands::{list, login_with_credentials, logout, rename, status};
pub use export_import::{export_profiles, import_profiles, ExportOptions, ImportOptions};
pub use i18n::{Language, Messages};
