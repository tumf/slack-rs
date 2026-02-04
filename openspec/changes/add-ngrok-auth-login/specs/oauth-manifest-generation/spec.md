# oauth-manifest-generation 仕様（差分）

## MODIFIED Requirements

### Requirement: ngrok 使用時の redirect_urls を Manifest に反映する

ngrok を使用した `auth login` の場合、Manifest の `oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` を含めなければならない (MUST)。

#### Scenario: ngrok 使用時は ngrok のワイルドカード URL が含まれる
- Given `auth login --ngrok` を実行する
- When Manifest が生成される
- Then `oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` が含まれる
