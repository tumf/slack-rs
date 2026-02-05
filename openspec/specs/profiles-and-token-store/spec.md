# profiles-and-token-store Specification

## Purpose
Defines how slack-rs persists profile configurations and manages access tokens securely using OS-level storage mechanisms.
## Requirements
### Requirement: Profile configuration can be persisted

Profile に含まれる非機密情報は `profiles.json` に保存され、再起動後も同じ内容で取得できなければならない (MUST)。

OAuth の非機密情報（`client_id`、`redirect_uri`、`bot_scopes`、`user_scopes`）も永続化の対象としなければならない (MUST)。

#### Scenario: bot/user スコープを含むプロファイルを保存して再読み込みできる
- Given `client_id`、`redirect_uri`、`bot_scopes`、`user_scopes` を含むプロファイルを保存する
- When `profiles.json` を再読み込みする
- Then すべての値が同一内容で取得できる

### Requirement: Configuration file has a version field
`profiles.json` MUST contain a `version` field. (MUST)
#### Scenario: Version is included when saving
- Given a newly created configuration file exists
- When the configuration is saved
- Then `version` is included

### Requirement: profile_name is unique
The same `profile_name` MUST NOT be registered multiple times. (MUST NOT)
#### Scenario: Attempting to add a profile with the same name
- Given `profile_name` already exists
- When saving the same `profile_name`
- Then a duplicate error occurs

### Requirement: `(team_id, user_id)` is unique as a stable key
Profiles with the same `(team_id, user_id)` MUST NOT be duplicated. (MUST NOT)
#### Scenario: Re-registering the same `(team_id, user_id)`
- Given `(team_id, user_id)` already exists
- When saving a profile with the same `(team_id, user_id)`
- Then the existing entry is updated and not added as new

### Requirement: Tokens are saved in file-based storage and not in configuration file

トークン（bot/user）および OAuth `client_secret` は `profiles.json` ではなく、token store backend に保存されなければならない (MUST)。

デフォルトの token store backend は Keyring でなければならない (MUST)。

Keyring が利用不能な場合、`SLACKRS_TOKEN_STORE` による明示指定がない限り、関連コマンドは MUST で失敗し、対処方法を提示しなければならない。静かな file へのフォールバックはしてはならない (MUST NOT)。

ここで「Keyring が利用不能」には、初期化/アクセスの失敗に加えて、Keyring がロックされていてユーザー操作（対話的アンロック）を要求するケース（interaction required 等）も含む。slack-rs は Keyring backend のために独自のパスワード/パスフレーズ入力プロンプトを導入してはならない (MUST NOT)。そのようなエラーは Keyring 利用不能として扱い、OS の Keyring をアンロックして再実行するか、`SLACKRS_TOKEN_STORE=file` を設定して file backend を明示的に選択するよう案内しなければならない。

ファイルベースの token store は `SLACKRS_TOKEN_STORE=file` により明示的に選択された場合にのみ使用してよい (MAY)。

`SLACKRS_KEYRING_PASSWORD` は export/import の暗号化パスワードであり、OS の Keyring アンロック要求とは無関係である。

#### Scenario: 既定では Keyring が使用される
- Given `SLACKRS_TOKEN_STORE` が設定されていない
- And Keyring が利用可能である
- When トークンストレージを初期化する
- Then Keyring backend が選択される

#### Scenario: Keyring が利用不能な場合は失敗しガイダンスを出す
- Given `SLACKRS_TOKEN_STORE` が設定されていない
- And Keyring が利用不能である
- When 認証情報が必要なコマンドを実行する
- Then コマンドは失敗する
- And エラーに「OS の Keyring をアンロックして再実行」または `SLACKRS_TOKEN_STORE=file` を含む対処方法が含まれる
- And slack-rs は Keyring アンロックのための追加のパスワード入力を求めない

#### Scenario: file mode では FileTokenStore が使用される
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- When トークンストレージを初期化する
- Then `FileTokenStore` backend が選択される

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

### Requirement: Stable key can be resolved from profile_name
`(team_id, user_id)` MUST be uniquely resolvable from `profile_name`. (MUST)
#### Scenario: Resolve `(team_id, user_id)` from profile_name
- Given profile_name exists in the configuration
- When profile_name is specified
- Then `(team_id, user_id)` is returned

### Requirement: Profile configuration file uses slack-rs as default path
Profile non-secret information MUST be stored in `profiles.json` under the `slack-rs` configuration directory. (MUST)
#### Scenario: Resolve default path
- Given retrieving the default configuration path
- When referencing the OS configuration directory
- Then the path contains `slack-rs` and `profiles.json`

### Requirement: Legacy path configuration file is migrated to new path
When the new path does not exist and `profiles.json` exists in the legacy path (`slack-cli`), the configuration file MUST be migrated to the new path. (MUST)
#### Scenario: Loading when only legacy path exists
- Given `profiles.json` exists in the legacy path and does not exist in the new path
- When loading the configuration file
- Then `profiles.json` is created in the new path and the same content is loaded

### Requirement: Store default token type in profile
Profile MUST optionally hold `default_token_type` and persist it. (MUST)
#### Scenario: Save and reload default type
- Given setting `default_token_type=user`
- When saving and reloading profile
- Then `default_token_type` is retained

### Requirement: FileTokenStore mode reuses tokens.json path and stable keys

file mode（`SLACKRS_TOKEN_STORE=file`）では、既存の `FileTokenStore` の保存パス `~/.config/slack-rs/tokens.json` を再利用しなければならない (MUST)。

file mode ではキー形式も既存仕様を維持しなければならない (MUST)。少なくとも以下のキーは同一形式であること:
- bot token: `{team_id}:{user_id}`
- OAuth `client_secret`: `oauth-client-secret:{profile_name}`

#### Scenario: file mode で tokens.json と既存キー形式を使用する
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- When `team_id=T123` と `user_id=U456` の bot token を保存する
- Then `~/.config/slack-rs/tokens.json` にキー `T123:U456` で保存される
- And `profile_name=default` の `client_secret` はキー `oauth-client-secret:default` で保存される

