//! Auth commands for Slack CLI
//!
//! Provides commands for managing authentication:
//! - login: Perform OAuth authentication
//! - status: Show current profile status
//! - list: List all profiles
//! - rename: Rename a profile
//! - logout: Remove authentication

pub mod commands;

pub use commands::{list, login, logout, rename, status};
