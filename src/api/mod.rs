//! Slack API client and call handling
//!
//! This module provides the core functionality for making Slack API calls:
//! - HTTP client with retry logic
//! - Argument parsing
//! - API call execution with metadata
//! - Wrapper commands for common operations

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod args;
pub mod call;
pub mod client;
pub mod envelope;
pub mod types;

// Re-export commonly used types for generic API calls
pub use args::{ApiCallArgs, ArgsError};
pub use call::{execute_api_call, ApiCallContext, ApiCallError, ApiCallMeta, ApiCallResponse};
pub use client::{ApiClient, ApiClientConfig, ApiClientError, ApiError, RequestBody};

// Re-export unified envelope types
pub use envelope::{CommandMeta, CommandResponse};

// Re-export types for wrapper commands
pub use types::{ApiMethod, ApiResponse};
