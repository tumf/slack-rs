# profiles-and-token-store 仕様（差分）

## MODIFIED Requirements

### Requirement: Profile configuration can be persisted

Profile に含まれる非機密情報は `profiles.json` に保存され、再起動後も同じ内容で取得できなければならない (MUST)。

OAuth の非機密情報（`client_id`、`redirect_uri`、`bot_scopes`、`user_scopes`）も永続化の対象としなければならない (MUST)。

#### Scenario: bot/user スコープを含むプロファイルを保存して再読み込みできる
- Given `client_id`、`redirect_uri`、`bot_scopes`、`user_scopes` を含むプロファイルを保存する
- When `profiles.json` を再読み込みする
- Then すべての値が同一内容で取得できる

### Requirement: keyring key format is stable

bot トークンの Keyring 保存キーは `service=slackcli`、`username={team_id}:{user_id}` でなければならない (MUST)。

user トークンは bot トークンとは異なる、安定した別キーで保存しなければならない (MUST)。

#### Scenario: bot と user のトークンが別キーで保存される
- Given `team_id=T123` と `user_id=U456` がある
- When bot トークンと user トークンの両方を保存する
- Then 2 つのキーは互いに異なり、かつ安定している
