# oauth-config-management Specification

## MODIFIED Requirements
### Requirement: Generate Manifest automatically during auth login

`auth login` 実行時に、入力された OAuth 設定を基に Slack App Manifest を自動的に生成し、OAuth フロー開始前にファイルへ保存しなければならない (MUST)。

このとき Manifest の `oauth_config.redirect_urls` は、redirect_uri の解決方法（cloudflared または ngrok 使用有無）に整合していなければならない (MUST)。

#### Scenario: OAuth 開始前に Manifest が保存される
- Given `auth login --ngrok` を実行する
- When OAuth フローを開始する前に Manifest を生成する
- Then Manifest がファイルに保存される
- And `oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` が含まれる
