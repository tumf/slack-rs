# slack-rs setup

One-time or infrequent setup steps for using `slack-rs`.

## Install

Install from crates.io (recommended):

```bash
cargo install slack-rs
```

Build from source:

```bash
git clone https://github.com/tumf/slack-rs.git
cd slack-rs
cargo build --release
./target/release/slack-rs --help
```

Or install from a local checkout:

```bash
cargo install --path .
```

## OAuth Setup (One-time per Slack App)

Create a Slack app and configure OAuth.

Recommended login flow (especially for remote/SSH environments): use `--cloudflared`.
In this mode, `slack-rs auth login` generates a Slack App Manifest YAML for you (and copies it to clipboard, best effort).

1. Go to https://api.slack.com/apps and create an app.
2. Copy your Client ID and Client Secret from "Basic Information" -> "App Credentials".
3. Either:

   - Use the manifest flow (`--cloudflared`) and paste the generated YAML into Slack, or
   - Configure OAuth manually (alternative):

     - Under "OAuth & Permissions", add redirect URL: `http://127.0.0.1:8765/callback`
     - Add required "User Token Scopes" for your use case

Recommended: store OAuth config per profile (client secret is stored securely in file storage).

```bash
slack-rs config oauth set my-workspace \
  --client-id 123456789012.1234567890123 \
  --redirect-uri http://127.0.0.1:8765/callback \
  --scopes "chat:write,users:read,channels:read"
```

You can use `--scopes "all"` for a broad preset, or customize with `--bot-scopes` and `--user-scopes` flags.

Common scopes:

- `chat:write` - post messages
- `users:read` - view users
- `channels:read` - list public channels
- `search:read` - search workspace content
- `reactions:write` - add/remove reactions

Full list: https://api.slack.com/scopes

## Authenticate (Per Profile)

```bash
slack-rs auth login my-workspace
slack-rs auth status my-workspace
slack-rs auth list
```

Remote/SSH environments (recommended):

```bash
slack-rs auth login my-workspace --client-id 123456789012.1234567890123 --cloudflared
```

Note: Check `slack-rs auth login --help` for current tunnel support (e.g., `--cloudflared`, `--ngrok`).

During login, the CLI opens a browser for OAuth authorization and stores:

- Profile metadata in `~/.config/slack-rs/profiles.json`
- OAuth config and tokens in files under `~/.config/slack-rs/` (treat as secrets)

Security note: `~/.config/slack-rs/` contains OAuth credentials (client secrets and access tokens). Treat it as sensitive.
