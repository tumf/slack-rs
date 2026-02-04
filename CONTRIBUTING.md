# Contributing to slack-rs

Thank you for your interest in contributing to slack-rs! This document provides guidelines and instructions for developers.

## Table of Contents

- [Development Setup](#development-setup)
- [Building](#building)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Project Structure](#project-structure)
- [Code Style Guidelines](#code-style-guidelines)
- [Submitting Changes](#submitting-changes)

## Development Setup

### Prerequisites

- Rust 1.70+ (tested with 1.92.0)
- Git
- A Slack app with OAuth credentials for testing

### Clone and Build

```bash
git clone https://github.com/tumf/slack-rs.git
cd slack-rs
cargo build
```

## Building

```bash
# Debug build
cargo build

# Optimized release build
cargo build --release

# Build with detailed output
cargo build --verbose
```

The binary will be available at:
- Debug: `target/debug/slack-rs`
- Release: `target/release/slack-rs`

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with detailed output
cargo test --verbose

# Run single test by name
cargo test test_api_call_with_form_data

# Run all tests matching a pattern
cargo test oauth

# Show println! output during tests
cargo test -- --nocapture

# Run only library tests
cargo test --lib

# Run specific integration test file
cargo test --test api_integration_tests
```

### Test Organization

- Unit tests: Located in the same file as the code they test, in a `#[cfg(test)] mod tests { ... }` block
- Integration tests: Located in the `tests/` directory
- Test files should mirror source structure

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        // Test async code
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

## Code Quality

### Formatting

```bash
# Format all code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

**Standard**: Default rustfmt settings (100 character line length, 4-space indentation)

### Linting

```bash
# Run linter
cargo clippy

# Fail on warnings (CI standard)
cargo clippy -- -D warnings
```

### Other Quality Checks

```bash
# Fast compile check without codegen
cargo check

# Generate and open documentation
cargo doc --open

# Check for security vulnerabilities
cargo audit
```

### Code Coverage

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --verbose --all-features --workspace --timeout 120
```

## Project Structure

```
slack-rs/
├── src/
│   ├── main.rs           # CLI entry point and command routing
│   ├── lib.rs            # Library root with module exports
│   ├── api/              # Slack API client and call handling
│   ├── auth/             # Auth commands (login, logout, status, export/import)
│   ├── cli/              # CLI helpers and usage messages
│   ├── commands/         # Wrapper commands for common operations
│   ├── oauth/            # OAuth flow implementation (PKCE, server)
│   └── profile/          # Profile and token storage management
├── tests/                # Integration tests
├── docs/                 # Documentation
├── Cargo.toml            # Dependencies and project metadata
├── AGENTS.md             # Detailed coding guidelines for AI agents
└── CONTRIBUTING.md       # This file
```

## Code Style Guidelines

### Module Organization

- Each module has a `mod.rs` with documentation and re-exports
- Start files with doc comments: `//! Module description`
- Use `#![allow(dead_code)]` for foundational/future features
- Group related functionality in submodules

Example `mod.rs`:

```rust
//! Module description and overview
//!
//! Detailed explanation of what this module does.

mod submodule1;
mod submodule2;

pub use submodule1::{PublicType1, public_function};
pub use submodule2::PublicType2;
```

### Imports

- Use crate-relative imports: `use crate::oauth::types::OAuthError;`
- Group imports: std, external crates, then crate modules
- Re-export commonly used types in `mod.rs`

```rust
// Standard library
use std::collections::HashMap;
use std::fs;

// External crates
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Crate modules
use crate::oauth::types::OAuthError;
use crate::profile::Profile;
```

### Error Handling

- Use `thiserror` for all custom errors
- Define errors as enums with descriptive variants
- Return `Result<T, CustomError>` from fallible functions
- Use `?` for error propagation

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("OAuth configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Token exchange failed: {0}")]
    TokenExchangeError(String),
}

pub async fn exchange_code(code: &str) -> Result<String, OAuthError> {
    let response = make_request(code)
        .await
        .map_err(|e| OAuthError::NetworkError(e.to_string()))?;
    
    Ok(response.access_token)
}
```

### Types & Structs

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub token_type: String,
}
```

- Derive `Debug` for all types
- Use `serde` derives for serializable types
- Derive `Clone` only when needed
- Use builder pattern for complex configs

### Async Code

- Use `tokio` runtime
- Mark async functions explicitly
- Use `.await` for async operations

```rust
#[tokio::main]
async fn main() {
    let result = async_operation().await;
}

async fn async_operation() -> Result<String, Error> {
    let response = reqwest::get("https://api.example.com")
        .await?
        .text()
        .await?;
    Ok(response)
}
```

### Naming Conventions

| Type | Convention | Examples |
|------|------------|----------|
| Modules | `snake_case` | `oauth`, `token_store` |
| Types | `PascalCase` | `OAuthConfig`, `ApiClient` |
| Functions | `snake_case` | `exchange_code`, `run_callback_server` |
| Constants | `SCREAMING_SNAKE_CASE` | `DEFAULT_TIMEOUT`, `API_BASE_URL` |
| Error types | End with `Error` | `OAuthError`, `ApiClientError` |

### Documentation

Add doc comments (`///`) for public APIs:

```rust
/// Login command - performs OAuth authentication
///
/// # Arguments
/// * `config` - OAuth configuration
/// * `profile_name` - Optional profile name (defaults to "default")
///
/// # Returns
/// Returns `Ok(())` on success, or an error if authentication fails
///
/// # Examples
/// ```
/// let config = OAuthConfig::from_env()?;
/// login(config, Some("my-workspace".to_string())).await?;
/// ```
pub async fn login(config: OAuthConfig, profile_name: Option<String>) -> Result<(), OAuthError> {
    // Implementation
}
```

Use module-level docs (`//!`) at the top of each file:

```rust
//! OAuth authentication flow implementation
//!
//! This module provides OAuth 2.0 authentication with PKCE for Slack.
```

### Comments

- Prefer doc comments over regular comments
- Use `//` for implementation notes
- Avoid obvious comments - code should be self-documenting
- Explain *why*, not *what*

```rust
// Good: Explains why
// Use PKCE to prevent authorization code interception
let verifier = generate_pkce_verifier();

// Bad: States the obvious
// Create a new string
let s = String::new();
```

## Submitting Changes

### Before Submitting

1. **Run all checks**:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```

2. **Ensure tests pass**: All existing tests must pass
3. **Add tests**: New features should include tests
4. **Update documentation**: Update README.md or docs/ as needed

### Commit Guidelines

Use conventional commit format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**

```bash
git commit -m "feat(oauth): add support for custom redirect URI"
git commit -m "fix(api): handle rate limit errors correctly"
git commit -m "docs: update installation instructions"
```

### Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch:
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Commit** your changes:
   ```bash
   git commit -m 'feat: add amazing feature'
   ```
4. **Push** to the branch:
   ```bash
   git push origin feature/amazing-feature
   ```
5. **Open** a Pull Request

### Pull Request Checklist

- [ ] Code passes `cargo fmt -- --check`
- [ ] Code passes `cargo clippy -- -D warnings`
- [ ] All tests pass with `cargo test`
- [ ] New features include tests
- [ ] Documentation is updated
- [ ] Commit messages follow conventional format
- [ ] PR description explains the changes

## Additional Resources

- [AGENTS.md](AGENTS.md) - Detailed coding guidelines for AI agents
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [Tokio Documentation](https://tokio.rs/) - Async runtime
- [Slack API Documentation](https://api.slack.com/web) - Slack Web API

## Getting Help

- 🐛 [Report Issues](https://github.com/tumf/slack-rs/issues)
- 💬 [Discussions](https://github.com/tumf/slack-rs/discussions)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
