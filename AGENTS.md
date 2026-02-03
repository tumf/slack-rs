# Coding Agent Guidelines for slack-rs

This Rust project is a Slack CLI tool with OAuth authentication, profile management, and API call functionality.

## Build & Test Commands

### Building
```bash
cargo build                    # Build debug version
cargo build --release          # Build optimized release version
cargo build --verbose          # Build with detailed output
```

### Testing
```bash
cargo test                     # Run all tests
cargo test --verbose           # Run tests with detailed output
cargo test <test_name>         # Run single test by name
cargo test -- --nocapture      # Show println! output during tests
cargo test --lib               # Run only library tests
cargo test --test <file>       # Run specific integration test file
```

**Examples:**
```bash
cargo test test_api_call_with_form_data  # Run single test
cargo test oauth                          # Run all tests matching "oauth"
cargo test --test api_integration_tests   # Run tests/api_integration_tests.rs
```

### Linting & Formatting
```bash
cargo fmt                      # Format all code
cargo fmt -- --check           # Check formatting without modifying
cargo clippy                   # Run linter
cargo clippy -- -D warnings    # Fail on warnings (CI standard)
```

### Other Commands
```bash
cargo check                    # Fast compile check without codegen
cargo clean                    # Remove build artifacts
cargo doc --open               # Generate and open documentation
cargo audit                    # Check for security vulnerabilities
```

## Project Structure

```
src/
├── main.rs           # CLI entry point and command routing
├── lib.rs            # Library root with module exports
├── api/              # Slack API client and call handling
├── auth/             # Auth commands (login, logout, status, export/import)
├── cli/              # CLI helpers and usage messages
├── commands/         # Wrapper commands for common operations
├── oauth/            # OAuth flow implementation (PKCE, server)
└── profile/          # Profile and token storage management

tests/                # Integration tests
```

## Code Style Guidelines

### Module Organization
- Each module has a `mod.rs` with documentation and re-exports
- Start files with doc comments: `//! Module description`
- Use `#![allow(dead_code)]` for foundational/future features
- Group related functionality in submodules

### Imports
- Use crate-relative imports: `use crate::oauth::types::OAuthError;`
- Group imports: std, external crates, then crate modules
- Re-export commonly used types in `mod.rs`:
  ```rust
  pub use commands::{list, login, logout};
  pub use types::{Profile, ProfilesConfig};
  ```

### Error Handling
- Use `thiserror` for all custom errors
- Define errors as enums with descriptive variants:
  ```rust
  use thiserror::Error;
  
  #[derive(Debug, Error)]
  pub enum OAuthError {
      #[error("OAuth configuration error: {0}")]
      ConfigError(String),
      
      #[error("Network error: {0}")]
      NetworkError(String),
  }
  ```
- Return `Result<T, CustomError>` from fallible functions
- Use `?` for error propagation

### Types & Structs
- Use `serde` derives for serializable types:
  ```rust
  #[derive(Debug, Serialize, Deserialize)]
  pub struct OAuthResponse { ... }
  ```
- Derive `Debug` for all types
- Derive `Clone` only when needed
- Use builder pattern for complex configs

### Async Code
- Use `tokio` runtime: `#[tokio::main]` or `#[tokio::test]`
- Mark async functions explicitly: `async fn name(...) -> Result<T, E>`
- Use `.await` for async operations

### Naming Conventions
- **Modules**: `snake_case` (e.g., `oauth`, `token_store`)
- **Types**: `PascalCase` (e.g., `OAuthConfig`, `ApiClient`)
- **Functions**: `snake_case` (e.g., `exchange_code`, `run_callback_server`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT`)
- **Error types**: End with `Error` (e.g., `OAuthError`, `ApiClientError`)

### Documentation
- Add doc comments (`///`) for public APIs
- Start with a summary line, then details:
  ```rust
  /// Login command - performs OAuth authentication
  ///
  /// # Arguments
  /// * `config` - OAuth configuration
  /// * `profile_name` - Optional profile name (defaults to "default")
  pub async fn login(config: OAuthConfig, profile_name: Option<String>) { ... }
  ```
- Use module-level docs (`//!`) at the top of each file

### Testing
- Integration tests go in `tests/` directory
- Use `httpmock` or `wiremock` for HTTP mocking
- Test files should mirror source structure
- Name tests descriptively: `test_api_call_with_form_data`
- Group related tests with `#[cfg(test)] mod tests { ... }`

### Formatting
- Run `cargo fmt` before committing
- Use default rustfmt settings (no custom config)
- Line length: default (100 chars)
- Indentation: 4 spaces

### Comments
- Prefer doc comments over regular comments
- Use `//` for implementation notes
- Avoid obvious comments - code should be self-documenting
- Explain *why*, not *what*

## Dependencies

**Core:**
- `tokio` - async runtime (features: "full")
- `reqwest` - HTTP client (features: "json")
- `serde`, `serde_json` - serialization

**Security:**
- `keyring` - secure token storage
- `aes-gcm` - encryption for export/import
- `argon2` - password hashing
- `sha2`, `base64`, `rand` - crypto primitives

**CLI:**
- `rpassword` - secure password input
- `directories` - OS-specific paths

**Testing:**
- `tempfile` - temporary files for tests
- `wiremock`, `httpmock` - HTTP mocking

## Best Practices

1. **Run tests before pushing**: `cargo test && cargo clippy -- -D warnings`
2. **Keep functions focused**: Single responsibility principle
3. **Handle errors explicitly**: Avoid unwrap/expect in library code
4. **Use type safety**: Leverage Rust's type system for correctness
5. **Write integration tests**: Test end-to-end workflows
6. **Keep dependencies minimal**: Only add when necessary
7. **Follow CI checks**: Match the standards in `.github/workflows/ci.yml`
