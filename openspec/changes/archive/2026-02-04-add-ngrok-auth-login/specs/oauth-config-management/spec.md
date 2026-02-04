# oauth-config-management 仕様（差分）

## MODIFIED Requirements

### Requirement: Generate Manifest automatically during auth login

`auth login` 実行時に、入力された OAuth 設定を基に Slack App Manifest を自動的に生成し、ファイルに保存しなければならない (MUST)。

このとき Manifest の `oauth_config.redirect_urls` は、redirect_uri の解決方法（cloudflared または ngrok 使用有無）に整合していなければならない (MUST)。

#### ADDED Scenario: ngrok 使用時の redirect_urls が保存される
- Given `auth login --ngrok` を実行する
- When OAuth フローが完了し Manifest が保存される
- Then `oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` が含まれる
