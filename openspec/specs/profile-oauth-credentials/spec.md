# profile-oauth-credentials Specification

## Purpose
TBD - created by archiving change add-per-profile-oauth-credentials. Update Purpose after archive.
## Requirements
### Requirement: OAuth credentials retrieval at login prioritizes interactive input

login 開始時のクライアント情報は、プロファイルに保存された設定および Keyring を優先して利用し、不足している項目のみ対話入力で補完しなければならない (MUST)。

redirect_uri はクライアント情報とは別に解決しなければならない (MUST)。`--cloudflared` が指定されない場合、`auth login` は redirect_uri をユーザーにプロンプトして取得しなければならない (MUST)。`--cloudflared [path]` が指定される場合は tunnel 公開 URL から `{public_url}/callback` を解決しなければならない (MUST)。このとき `path` が省略された場合は `cloudflared`（PATH から探索）を実行ファイルとして使用しなければならない (MUST)。

スコープについては、明示的な CLI 引数が指定されていない場合、対話入力してよい (MAY)。その場合、デフォルト入力値は `all` でなければならない (MUST)。

#### Scenario: クライアント情報は不足分のみプロンプトされる
- Given `client_id` が未設定である
- And `client_secret` が Keyring に存在しない
- When `auth login` を実行する
- Then `client_id` と `client_secret` の入力が求められる

#### Scenario: スコープが CLI 引数で指定されていない場合に `all` をデフォルトとしてプロンプトできる
- Given `--bot-scopes` と `--user-scopes` が指定されていない
- When `auth login` を実行する
- Then スコープ入力プロンプトが表示される
- And デフォルト入力値が `all` である

#### Scenario: `--cloudflared` 未指定時に redirect_uri がプロンプトされる
- Given `auth login` を実行する
- And `--cloudflared` が指定されていない
- When OAuth フローを開始する
- Then redirect_uri の入力が求められる

### Requirement: Store `client_id` in profile and `client_secret` in Keyring

Each profile MUST maintain its own OAuth client ID, and secrets MUST NOT remain in configuration files.

#### Scenario: Saved to configuration file and Keyring after successful login
- `client_id` is saved to `profiles.json`
- `client_secret` is saved to Keyring and NOT written to configuration files

### Requirement: Existing profiles can be loaded even without `client_id` set

Previously saved configuration files MUST be loadable as-is.

#### Scenario: Old format `profiles.json` can be loaded without error
- Loading succeeds even if `client_id` is missing

