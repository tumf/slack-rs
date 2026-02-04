# oauth-config-management 仕様（差分）

## MODIFIED Requirements

### Requirement: Manifest の redirect_urls は ngrok 使用時の値に整合する

`auth login --ngrok` の場合、生成された Manifest の `oauth_config.redirect_urls` は ngrok のワイルドカード URL を含めなければならない (MUST)。

#### Scenario: ngrok 使用時の redirect_urls が保存される
- Given `auth login --ngrok` を実行する
- When OAuth フローが完了し Manifest が保存される
- Then `oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` が含まれる
