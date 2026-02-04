# oauth-manifest-generation 仕様（差分）

## MODIFIED Requirements

### Requirement: Generate Slack App Manifest automatically during auth login execution

`auth login` 実行時に、ユーザーが入力した `client_id`、`bot_scopes`、`user_scopes`、および解決された `redirect_uri`（cloudflared、ngrok、またはプロンプト入力）を使用して、Slack App Manifest の YAML を自動的に生成しなければならない (MUST)。

#### ADDED Scenario: ngrok 使用時は ngrok のワイルドカード URL が含まれる
- Given `auth login --ngrok` を実行する
- When Manifest が生成される
- Then `oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` が含まれる
