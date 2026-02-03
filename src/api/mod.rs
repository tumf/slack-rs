//! Slack API client and call handling
//!
//! This module provides the core functionality for making Slack API calls:
//! - HTTP client with retry logic
//! - Argument parsing
//! - API call execution with metadata

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod args;
pub mod call;
pub mod client;

// Re-export commonly used types
pub use args::{ApiCallArgs, ArgsError};
pub use call::{execute_api_call, ApiCallContext, ApiCallError, ApiCallResponse};
pub use client::{ApiClient, ApiClientConfig, ApiClientError, RequestBody};
