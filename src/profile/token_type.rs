//! Token type definitions and resolution logic
//!
//! This module provides:
//! - TokenType enum for bot/user token distinction
//! - Token type resolution logic with priority: CLI flag > profile default > fallback

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Token type for Slack API authentication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// Bot token (xoxb-*)
    Bot,
    /// User token (xoxp-*)
    User,
}

impl TokenType {
    /// Returns "bot" or "user" as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::Bot => "bot",
            TokenType::User => "user",
        }
    }

    /// Resolve token type with priority: CLI flag > profile default > fallback
    ///
    /// # Arguments
    /// * `cli_flag` - Token type specified via --token-type CLI flag
    /// * `profile_default` - Default token type stored in profile
    /// * `fallback` - Fallback token type (typically Bot)
    ///
    /// # Examples
    /// ```
    /// use slack_rs::profile::TokenType;
    ///
    /// // CLI flag takes priority
    /// let resolved = TokenType::resolve(Some(TokenType::User), Some(TokenType::Bot), TokenType::Bot);
    /// assert_eq!(resolved, TokenType::User);
    ///
    /// // Profile default is used when no CLI flag
    /// let resolved = TokenType::resolve(None, Some(TokenType::User), TokenType::Bot);
    /// assert_eq!(resolved, TokenType::User);
    ///
    /// // Fallback is used when neither CLI flag nor profile default
    /// let resolved = TokenType::resolve(None, None, TokenType::Bot);
    /// assert_eq!(resolved, TokenType::Bot);
    /// ```
    pub fn resolve(
        cli_flag: Option<TokenType>,
        profile_default: Option<TokenType>,
        fallback: TokenType,
    ) -> TokenType {
        cli_flag.or(profile_default).unwrap_or(fallback)
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TokenType {
    type Err = TokenTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bot" => Ok(TokenType::Bot),
            "user" => Ok(TokenType::User),
            _ => Err(TokenTypeError::InvalidValue(s.to_string())),
        }
    }
}

#[derive(Debug, Error)]
pub enum TokenTypeError {
    #[error("Invalid token type: {0}. Valid values: bot, user")]
    InvalidValue(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_type_as_str() {
        assert_eq!(TokenType::Bot.as_str(), "bot");
        assert_eq!(TokenType::User.as_str(), "user");
    }

    #[test]
    fn test_token_type_display() {
        assert_eq!(TokenType::Bot.to_string(), "bot");
        assert_eq!(TokenType::User.to_string(), "user");
    }

    #[test]
    fn test_token_type_from_str() {
        assert_eq!("bot".parse::<TokenType>().unwrap(), TokenType::Bot);
        assert_eq!("Bot".parse::<TokenType>().unwrap(), TokenType::Bot);
        assert_eq!("BOT".parse::<TokenType>().unwrap(), TokenType::Bot);
        assert_eq!("user".parse::<TokenType>().unwrap(), TokenType::User);
        assert_eq!("User".parse::<TokenType>().unwrap(), TokenType::User);
        assert_eq!("USER".parse::<TokenType>().unwrap(), TokenType::User);

        assert!("invalid".parse::<TokenType>().is_err());
        assert!("admin".parse::<TokenType>().is_err());
    }

    #[test]
    fn test_token_type_serialization() {
        let bot = TokenType::Bot;
        let user = TokenType::User;

        let bot_json = serde_json::to_string(&bot).unwrap();
        let user_json = serde_json::to_string(&user).unwrap();

        assert_eq!(bot_json, "\"bot\"");
        assert_eq!(user_json, "\"user\"");

        let bot_deserialized: TokenType = serde_json::from_str(&bot_json).unwrap();
        let user_deserialized: TokenType = serde_json::from_str(&user_json).unwrap();

        assert_eq!(bot_deserialized, TokenType::Bot);
        assert_eq!(user_deserialized, TokenType::User);
    }

    #[test]
    fn test_token_type_resolve_cli_flag_priority() {
        // CLI flag takes highest priority
        let resolved =
            TokenType::resolve(Some(TokenType::User), Some(TokenType::Bot), TokenType::Bot);
        assert_eq!(resolved, TokenType::User);

        let resolved =
            TokenType::resolve(Some(TokenType::Bot), Some(TokenType::User), TokenType::User);
        assert_eq!(resolved, TokenType::Bot);
    }

    #[test]
    fn test_token_type_resolve_profile_default() {
        // Profile default is used when no CLI flag
        let resolved = TokenType::resolve(None, Some(TokenType::User), TokenType::Bot);
        assert_eq!(resolved, TokenType::User);

        let resolved = TokenType::resolve(None, Some(TokenType::Bot), TokenType::User);
        assert_eq!(resolved, TokenType::Bot);
    }

    #[test]
    fn test_token_type_resolve_fallback() {
        // Fallback is used when neither CLI flag nor profile default
        let resolved = TokenType::resolve(None, None, TokenType::Bot);
        assert_eq!(resolved, TokenType::Bot);

        let resolved = TokenType::resolve(None, None, TokenType::User);
        assert_eq!(resolved, TokenType::User);
    }

    #[test]
    fn test_token_type_resolve_all_combinations() {
        // Test all possible combinations
        assert_eq!(
            TokenType::resolve(Some(TokenType::Bot), Some(TokenType::Bot), TokenType::Bot),
            TokenType::Bot
        );
        assert_eq!(
            TokenType::resolve(Some(TokenType::Bot), Some(TokenType::Bot), TokenType::User),
            TokenType::Bot
        );
        assert_eq!(
            TokenType::resolve(Some(TokenType::Bot), Some(TokenType::User), TokenType::Bot),
            TokenType::Bot
        );
        assert_eq!(
            TokenType::resolve(Some(TokenType::Bot), Some(TokenType::User), TokenType::User),
            TokenType::Bot
        );
        assert_eq!(
            TokenType::resolve(Some(TokenType::Bot), None, TokenType::Bot),
            TokenType::Bot
        );
        assert_eq!(
            TokenType::resolve(Some(TokenType::Bot), None, TokenType::User),
            TokenType::Bot
        );

        assert_eq!(
            TokenType::resolve(Some(TokenType::User), Some(TokenType::Bot), TokenType::Bot),
            TokenType::User
        );
        assert_eq!(
            TokenType::resolve(Some(TokenType::User), Some(TokenType::Bot), TokenType::User),
            TokenType::User
        );
        assert_eq!(
            TokenType::resolve(Some(TokenType::User), Some(TokenType::User), TokenType::Bot),
            TokenType::User
        );
        assert_eq!(
            TokenType::resolve(
                Some(TokenType::User),
                Some(TokenType::User),
                TokenType::User
            ),
            TokenType::User
        );
        assert_eq!(
            TokenType::resolve(Some(TokenType::User), None, TokenType::Bot),
            TokenType::User
        );
        assert_eq!(
            TokenType::resolve(Some(TokenType::User), None, TokenType::User),
            TokenType::User
        );

        assert_eq!(
            TokenType::resolve(None, Some(TokenType::Bot), TokenType::Bot),
            TokenType::Bot
        );
        assert_eq!(
            TokenType::resolve(None, Some(TokenType::Bot), TokenType::User),
            TokenType::Bot
        );
        assert_eq!(
            TokenType::resolve(None, Some(TokenType::User), TokenType::Bot),
            TokenType::User
        );
        assert_eq!(
            TokenType::resolve(None, Some(TokenType::User), TokenType::User),
            TokenType::User
        );

        assert_eq!(
            TokenType::resolve(None, None, TokenType::Bot),
            TokenType::Bot
        );
        assert_eq!(
            TokenType::resolve(None, None, TokenType::User),
            TokenType::User
        );
    }
}
