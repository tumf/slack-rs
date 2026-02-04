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

### Requirement: Tokens are saved in keyring and not in configuration file
Tokens MUST be saved in the OS keyring and MUST NOT be saved in `profiles.json`. (MUST NOT)
#### Scenario: Save token and check configuration file
- Given a token has been saved
- When `profiles.json` is loaded
- Then the token is not included

### Requirement: keyring key format is stable

bot トークンの Keyring 保存キーは `service=slackcli`、`username={team_id}:{user_id}` でなければならない (MUST)。

user トークンは bot トークンとは異なる、安定した別キーで保存しなければならない (MUST)。

#### Scenario: bot と user のトークンが別キーで保存される
- Given `team_id=T123` と `user_id=U456` がある
- When bot トークンと user トークンの両方を保存する
- Then 2 つのキーは互いに異なり、かつ安定している

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

