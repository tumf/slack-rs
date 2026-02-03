//! Slack API client module
//!
//! Provides generic API calling functionality and command wrappers

pub mod client;
pub mod types;

pub use client::{ApiClient, ApiError};
pub use types::{ApiMethod, ApiResponse};
