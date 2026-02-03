//! Safety guards for write and destructive operations

use crate::api::ApiError;
use std::io::{self, Write};

/// Check if write operations are allowed
///
/// # Arguments
/// * `allow_write` - Whether the --allow-write flag was provided
///
/// # Returns
/// * `Ok(())` if write is allowed
/// * `Err(ApiError::WriteNotAllowed)` if write is not allowed
pub fn check_write_allowed(allow_write: bool) -> Result<(), ApiError> {
    if !allow_write {
        return Err(ApiError::WriteNotAllowed);
    }
    Ok(())
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

    #[test]
    fn test_check_write_allowed_with_flag() {
        assert!(check_write_allowed(true).is_ok());
    }

    #[test]
    fn test_check_write_allowed_without_flag() {
        let result = check_write_allowed(false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
    }

    #[test]
    fn test_confirm_destructive_with_yes_flag() {
        assert!(confirm_destructive(true, "delete message").is_ok());
    }
}
