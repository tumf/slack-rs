//! Safety guards for write and destructive operations

use crate::api::ApiError;
use std::io::{self, Write};

/// Check if write operations are allowed
///
/// Checks the SLACKCLI_ALLOW_WRITE environment variable.
/// - If unset: allows write operations (default behavior)
/// - If set to "false" or "0": denies write operations
/// - Otherwise: allows write operations
///
/// # Returns
/// * `Ok(())` if write is allowed
/// * `Err(ApiError::WriteNotAllowed)` if write is not allowed
pub fn check_write_allowed() -> Result<(), ApiError> {
    // Check environment variable SLACKCLI_ALLOW_WRITE
    // Default to allow if not set
    match std::env::var("SLACKCLI_ALLOW_WRITE") {
        Ok(value) => {
            let normalized = value.to_lowercase();
            if normalized == "false" || normalized == "0" {
                return Err(ApiError::WriteNotAllowed);
            }
            Ok(())
        }
        Err(_) => Ok(()), // Default: allow write when env var is not set
    }
}

/// Confirm a destructive operation
///
/// # Arguments
/// * `yes` - Whether the --yes flag was provided (skip confirmation)
/// * `operation` - Description of the operation to confirm
///
/// # Returns
/// * `Ok(())` if operation is confirmed
/// * `Err(ApiError::OperationCancelled)` if operation is cancelled
pub fn confirm_destructive(yes: bool, operation: &str) -> Result<(), ApiError> {
    if yes {
        return Ok(());
    }

    print!("Are you sure you want to {}? [y/N]: ", operation);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim().to_lowercase();
    if input == "y" || input == "yes" {
        Ok(())
    } else {
        Err(ApiError::OperationCancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(write_guard)]
    fn test_check_write_allowed_default() {
        // When env var is not set, write should be allowed
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
        assert!(check_write_allowed().is_ok());
    }

    #[test]
    #[serial(write_guard)]
    fn test_check_write_allowed_when_false() {
        // When env var is "false", write should be denied
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
        let result = check_write_allowed();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }

    #[test]
    #[serial(write_guard)]
    fn test_check_write_allowed_when_zero() {
        // When env var is "0", write should be denied
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "0");
        let result = check_write_allowed();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }

    #[test]
    #[serial(write_guard)]
    fn test_check_write_allowed_when_true() {
        // When env var is "true", write should be allowed
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "true");
        assert!(check_write_allowed().is_ok());
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }

    #[test]
    #[serial(write_guard)]
    fn test_check_write_allowed_when_one() {
        // When env var is "1", write should be allowed
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "1");
        assert!(check_write_allowed().is_ok());
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }

    #[test]
    fn test_confirm_destructive_with_yes_flag() {
        assert!(confirm_destructive(true, "delete message").is_ok());
    }
}
