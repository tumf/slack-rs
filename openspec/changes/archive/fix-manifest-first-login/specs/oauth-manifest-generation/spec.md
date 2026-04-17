## MODIFIED Requirements

### Requirement: Generate Slack App Manifest automatically during auth login execution

`auth login` における manifest 生成は、ローカルに解決済みの `redirect_uri`、`bot_scopes`、`user_scopes`、および profile 情報のみで成立しなければならない (MUST)。manifest-first 経路では、Manifest 生成のために `client_id` 入力を前提としてはならない (MUST NOT)。

#### Scenario: tunnel ベースの manifest 生成は client_id 未入力でも成立する
- Given `auth login --cloudflared` または `auth login --ngrok` を実行する
- And tunnel により redirect URI が確定している
- When Manifest を生成する
- Then redirect URI と scopes と profile 情報だけで YAML を生成できる
- And `client_id` の入力が未完了でも Manifest 生成は失敗しない
