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

pub mod clipboard;
pub mod cloudflared;
pub mod commands;
pub mod crypto;
pub mod export_import;
pub mod format;
pub mod i18n;
pub mod manifest;
pub mod ngrok;

pub use cloudflared::{CloudflaredError, CloudflaredTunnel};
pub use commands::{
    list, login_with_credentials, login_with_credentials_extended, logout,
    prompt_for_client_secret, rename, status, ExtendedLoginOptions,
};
pub use export_import::{
    export_profiles, import_profiles, ExportOptions, ImportAction, ImportOptions, ImportResult,
    ProfileImportResult,
};
pub use i18n::{Language, Messages};
pub use manifest::generate_manifest;
pub use ngrok::{NgrokError, NgrokTunnel};
