# profiles-and-token-store Specification

## Purpose
Defines how slack-rs persists profile configurations and manages access tokens securely using OS-level storage mechanisms.

## MODIFIED Requirements

### Requirement: Tokens are saved in file-based storage and not in configuration file

トークンはファイルベースのストレージ（`~/.config/slack-rs/tokens.json`）に保存されなければならない (MUST)。

トークンは `profiles.json` に保存してはならない (MUST NOT)。

デフォルトのトークンストレージは `FileTokenStore` でなければならない (MUST)。

#### Scenario: トークンを保存して設定ファイルに含まれないことを確認
- **WHEN** トークンを保存する
- **THEN** `profiles.json` にトークンが含まれない

#### Scenario: トークンは tokens.json に保存される
- **WHEN** トークンを保存する
- **THEN** `~/.config/slack-rs/tokens.json` にトークンが保存される

#### Scenario: FileTokenStore がデフォルトで使用される
- **WHEN** トークンストレージを初期化する（明示的な指定なし）
- **THEN** `FileTokenStore` が使用される

### Requirement: file-based token storage key format is stable

bot トークンのファイルベースストレージ保存キーは `{team_id}:{user_id}` でなければならない (MUST)。

user トークンは bot トークンとは異なる、安定した別キーで保存しなければならない (MUST)。

OAuth クライアントシークレットは `oauth-client-secret:{profile_name}` のキー形式で保存しなければならない (MUST)。

#### Scenario: bot と user のトークンが別キーで保存される
- **WHEN** `team_id=T123` と `user_id=U456` がある
- **THEN** bot トークンのキーは `T123:U456` で、user トークンは別の安定したキーで保存される

#### Scenario: OAuth クライアントシークレットが正しいキー形式で保存される
- **WHEN** プロファイル名 `default` の OAuth クライアントシークレットを保存する
- **THEN** キーは `oauth-client-secret:default` で保存される
