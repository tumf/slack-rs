//! Slack CLI library
//!
//! Provides core functionality for the Slack CLI:
//! - API client and call handling
//! - OAuth authentication and profile management
//! - Wrapper commands for common operations

pub mod api;
pub mod auth;
pub mod cli;
pub mod commands;
pub mod debug;
pub mod oauth;
pub mod profile;
