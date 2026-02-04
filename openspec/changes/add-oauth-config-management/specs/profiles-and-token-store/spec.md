# profiles-and-token-store 変更差分

## MODIFIED Requirements

### Requirement: Profile configuration can be persisted

Profileの非機密情報は `profiles.json` に保存され、再起動後に同一内容で取得できることがMUST。
OAuthの非機密情報（`client_id`, `redirect_uri`, `scopes`）も同様に保存対象とする。

#### Scenario: OAuth非機密情報を含むプロファイルを保存・再読み込みできる
- `client_id`, `redirect_uri`, `scopes` を含むプロファイルを保存する
- `profiles.json` を再読み込みする
- すべての値が同一内容で取得できる
