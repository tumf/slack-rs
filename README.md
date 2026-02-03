# slack-rs

A Slack CLI tool written in Rust that provides comprehensive access to the Slack Web API using OAuth authentication.

[![CI](https://github.com/tumf/slack-rs/workflows/CI/badge.svg)](https://github.com/tumf/slack-rs/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Overview

`slack-rs` is a command-line tool designed for interacting with Slack workspaces using your personal OAuth credentials. It supports multiple workspace profiles, secure token storage, and provides both generic API access and convenient wrapper commands for common operations.

### Key Features

- 🔐 **OAuth Authentication** with PKCE flow
- 🏢 **Multiple Workspace Support** via profiles
- 🔒 **Secure Token Storage** using OS keyring (Keychain/Secret Service)
- 🔄 **Profile Import/Export** with encryption
- 📡 **Generic API Access** - call any Slack Web API method
- 🛠️ **Wrapper Commands** for common operations
- 🔁 **Smart Retry Logic** with exponential backoff and rate limit handling

## Installation

### Prerequisites

- Rust 1.70+ (tested with 1.92.0)
- A Slack app with OAuth credentials ([create one here](https://api.slack.com/apps))

### Build from Source

```bash
git clone https://github.com/tumf/slack-rs.git
cd slack-rs
cargo build --release
```

The binary will be available at `target/release/slack-rs`.

### Install via Cargo

```bash
cargo install --path .
```

## Quick Start

### 1. Set Up OAuth Credentials

Create a Slack app and configure OAuth:

1. Go to https://api.slack.com/apps
2. Create a new app or select an existing one
3. Add OAuth scopes (e.g., `chat:write`, `users:read`, `search:read`)
4. Note your **Client ID** and **Client Secret**

Set environment variables:

```bash
export SLACKCLI_CLIENT_ID="your-client-id"
export SLACKCLI_CLIENT_SECRET="your-client-secret"
export SLACKCLI_REDIRECT_URI="http://127.0.0.1:3000/callback"  # optional
export SLACKCLI_SCOPES="chat:write,users:read,search:read"      # optional
```

### 2. Authenticate

Login to your Slack workspace:

```bash
slack-rs auth login my-workspace
```

This will open a browser for OAuth authorization. After approval, your credentials will be securely stored in your OS keyring.

### 3. Make API Calls

**Generic API call:**

```bash
slack-rs api call chat.postMessage channel=C123456 text="Hello from slack-rs!"
```

**Check authentication status:**

```bash
slack-rs auth status my-workspace
```

**List all profiles:**

```bash
slack-rs auth list
```

## Usage

### Authentication Commands

```bash
# Login to a workspace
slack-rs auth login [profile-name]

# Show authentication status
slack-rs auth status [profile-name]

# List all profiles
slack-rs auth list

# Rename a profile
slack-rs auth rename <old-name> <new-name>

# Logout (remove credentials)
slack-rs auth logout <profile-name>

# Export profiles (encrypted)
slack-rs auth export --profile <name> --out <file> --yes
slack-rs auth export --all --out <file> --yes

# Import profiles
slack-rs auth import --profile <name> --in <file>
slack-rs auth import --all --in <file>
```

### API Calls

**Generic API access:**

```bash
slack-rs api call <method> [key=value...]

# Examples:
slack-rs api call users.info user=U123456
slack-rs api call conversations.history channel=C123456 limit=50
slack-rs api call search.messages query="important" count=20
```

**Form-encoded arguments:**
```bash
slack-rs api call chat.postMessage channel=C123 text="Hello" thread_ts=1234567.123
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SLACKCLI_CLIENT_ID` | OAuth Client ID | (required) |
| `SLACKCLI_CLIENT_SECRET` | OAuth Client Secret | (required) |
| `SLACKCLI_REDIRECT_URI` | OAuth Redirect URI | `http://127.0.0.1:3000/callback` |
| `SLACKCLI_SCOPES` | Comma-separated OAuth scopes | `chat:write,users:read` |
| `SLACKCLI_KEYRING_PASSWORD` | Password for export/import encryption | (required for export/import) |
| `SLACK_OAUTH_BASE_URL` | Custom OAuth base URL (for testing) | - |

### Profile Storage

- **Profile metadata** (non-sensitive): `~/.config/slack-cli/profiles.json` (Linux/macOS)
- **Access tokens** (sensitive): OS keyring (Keychain on macOS, Secret Service on Linux, Credential Manager on Windows)

## Security

### Token Storage

All access tokens are stored securely in your operating system's credential manager:
- **macOS**: Keychain
- **Linux**: Secret Service (GNOME Keyring, KWallet)
- **Windows**: Credential Manager

Tokens are never stored in plain text files or logged to the console.

### Profile Export/Import

Profile export creates an encrypted file containing your credentials:

- **Encryption**: AES-256-GCM
- **Key Derivation**: Argon2id with salt
- **File Permissions**: Automatically set to `0600` (owner read/write only)
- **Passphrase**: Must be provided via `SLACKCLI_KEYRING_PASSWORD` environment variable or `--passphrase-prompt`

**⚠️ Warning**: Exported files contain sensitive credentials. Store them securely and never commit them to version control.

## Development

### Building

```bash
cargo build              # Debug build
cargo build --release    # Optimized build
```

### Testing

```bash
cargo test                                    # Run all tests
cargo test test_api_call_with_form_data      # Run specific test
cargo test --test api_integration_tests       # Run integration tests
cargo test -- --nocapture                     # Show println! output
```

### Linting and Formatting

```bash
cargo fmt                    # Format code
cargo fmt -- --check         # Check formatting
cargo clippy                 # Run linter
cargo clippy -- -D warnings  # Fail on warnings (CI standard)
```

### Code Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --verbose --all-features --workspace --timeout 120
```

## Project Structure

```
slack-rs/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library root
│   ├── api/              # Slack API client
│   ├── auth/             # Auth commands
│   ├── cli/              # CLI helpers
│   ├── commands/         # Wrapper commands
│   ├── oauth/            # OAuth flow (PKCE)
│   └── profile/          # Profile management
├── tests/                # Integration tests
├── Cargo.toml            # Dependencies
└── AGENTS.md             # Developer guidelines
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure:
- Code passes `cargo fmt -- --check` and `cargo clippy -- -D warnings`
- All tests pass with `cargo test`
- New features include tests

See [AGENTS.md](AGENTS.md) for detailed coding guidelines.

## Roadmap

- [ ] Enhanced wrapper commands for common operations
- [ ] Support for slash commands
- [ ] Interactive mode for profile management
- [ ] Improved error messages with suggestions
- [ ] Internationalization (i18n) for English and Japanese

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [reqwest](https://github.com/seanmonstar/reqwest) for HTTP
- Secure storage with [keyring](https://github.com/hwchen/keyring-rs)
- OAuth implementation inspired by [oauth2-rs](https://github.com/ramosbugs/oauth2-rs)

## Support

- 🐛 [Report Issues](https://github.com/tumf/slack-rs/issues)
- 💬 [Discussions](https://github.com/tumf/slack-rs/discussions)
- 📖 [Slack API Documentation](https://api.slack.com/web)

---

**Note**: This is an unofficial tool and is not affiliated with or endorsed by Slack Technologies, Inc.
