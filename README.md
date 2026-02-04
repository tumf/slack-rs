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

**Option A: Set environment variables (global credentials):**

```bash
export SLACKRS_CLIENT_ID="your-client-id"
export SLACKRS_CLIENT_SECRET="your-client-secret"
export SLACKRS_REDIRECT_URI="http://127.0.0.1:3000/callback"  # optional
export SLACKRS_SCOPES="chat:write,users:read,search:read"      # optional
```

**Option B: Use per-profile credentials:**

You can save different client IDs for different profiles. During the `auth login` command, the tool will prompt you for credentials if not provided via environment variables or command-line arguments.

### 2. Authenticate

Login to your Slack workspace:

```bash
# Using environment variables (SLACKRS_CLIENT_ID and SLACKRS_CLIENT_SECRET)
slack-rs auth login my-workspace

# Or provide client ID via command-line (you'll be prompted for client secret)
slack-rs auth login my-workspace --client-id your-client-id

# Or let the tool prompt you for both credentials
slack-rs auth login my-workspace
```

This will open a browser for OAuth authorization. After approval, your credentials will be securely stored in your OS keyring.

**Per-Profile Client Keys:**
- Each profile can use a different OAuth client ID
- The client ID is saved with the profile (in `~/.config/slack-rs/profiles.json`)
- The client secret is always prompted for security (never saved)
- On subsequent logins to the same profile, the saved client ID will be reused unless you provide a different one via `--client-id`

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
slack-rs auth login [profile-name] --client-id <your-client-id>

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

#### Per-Profile Client Keys

Each profile can have its own OAuth client ID, allowing you to:
- Use different Slack apps for different workspaces
- Test with development and production OAuth apps
- Manage multiple teams with separate credentials

**How it works:**
1. **First login**: Provide client ID via `--client-id` flag, environment variable, or interactive prompt
2. **Client ID is saved**: Stored in profile metadata (`~/.config/slack-rs/profiles.json`)
3. **Subsequent logins**: Saved client ID is reused automatically
4. **Client Secret**: Always prompted for security (never saved to disk)

**Examples:**

```bash
# Login with specific client ID (will be saved to profile)
slack-rs auth login dev-workspace --client-id 123456.789012

# Login to production workspace with different client ID
slack-rs auth login prod-workspace --client-id 987654.321098

# Subsequent login reuses saved client ID (only prompts for secret)
slack-rs auth login dev-workspace
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
| `SLACKRS_CLIENT_ID` | OAuth Client ID (optional if using per-profile credentials or `--client-id` flag) | - |
| `SLACKRS_CLIENT_SECRET` | OAuth Client Secret (optional - will be prompted if not set) | - |
| `SLACKRS_REDIRECT_URI` | OAuth Redirect URI | `http://127.0.0.1:3000/callback` |
| `SLACKRS_SCOPES` | Comma-separated OAuth scopes | `chat:write,users:read` |
| `SLACKCLI_ALLOW_WRITE` | Allow write operations (set to `false` or `0` to deny) | `true` (allow) |
| `SLACKRS_KEYRING_PASSWORD` | Password for export/import encryption | (required for export/import) |
| `SLACK_OAUTH_BASE_URL` | Custom OAuth base URL (for testing) | - |

### Profile Storage

- **Profile metadata** (includes client ID, team/user info): `~/.config/slack-rs/profiles.json` (Linux/macOS)
- **Sensitive credentials** (access tokens and client secrets): OS keyring (Keychain on macOS, Secret Service on Linux, Credential Manager on Windows)

Each profile stores:
- **In JSON file**: `team_id`, `user_id`, `team_name`, `user_name`, `client_id`
- **In OS keyring**: Access token and client secret (when saved via export/import)

### Write Operation Protection

Write operations (posting, updating, deleting messages, and managing reactions) are controlled by the `SLACKCLI_ALLOW_WRITE` environment variable:

- **Default behavior** (variable not set): Write operations are **allowed**
- **Deny write operations**: Set `SLACKCLI_ALLOW_WRITE=false` or `SLACKCLI_ALLOW_WRITE=0`
- **Explicitly allow**: Set `SLACKCLI_ALLOW_WRITE=true` or `SLACKCLI_ALLOW_WRITE=1`

**Example: Preventing accidental write operations**

```bash
# Deny all write operations
export SLACKCLI_ALLOW_WRITE=false

# This will fail with an error
slack-rs msg post C123456 "Hello"
# Error: Write operation denied. Set SLACKCLI_ALLOW_WRITE=true to enable write operations

# Re-enable write operations
export SLACKCLI_ALLOW_WRITE=true
slack-rs msg post C123456 "Hello"  # Now succeeds
```

## Security

### Credential Storage

**Access Tokens:**
All access tokens are stored securely in your operating system's credential manager:
- **macOS**: Keychain
- **Linux**: Secret Service (GNOME Keyring, KWallet)
- **Windows**: Credential Manager

Tokens are never stored in plain text files or logged to the console.

**Client Keys:**
- **Client IDs**: Stored in profile metadata file (`~/.config/slack-rs/profiles.json`). These are not considered sensitive as they're part of OAuth public flow.
- **Client Secrets**: Never saved to disk during normal operation. Always prompted when needed for authentication. Only stored in OS keyring when explicitly saved via export/import for backup purposes.

### Profile Export/Import

Profile export creates an encrypted file containing your credentials:

- **Encryption**: AES-256-GCM
- **Key Derivation**: Argon2id with salt
- **File Permissions**: Automatically set to `0600` (owner read/write only)
- **Passphrase**: Must be provided via `SLACKRS_KEYRING_PASSWORD` environment variable or `--passphrase-prompt`

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
