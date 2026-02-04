# profile-oauth-credentials 変更差分

## MODIFIED Requirements

### Requirement: ログイン時のOAuthクレデンシャル取得は保存済み設定を優先し、不足時のみ対話入力する

ログイン時のクライアント情報は保存済みのプロファイル設定とKeyringを優先し、欠落している項目のみ対話入力で補うことがMUST。

#### Scenario: 保存済み設定がある場合はプロンプトが省略される
- `profiles.json` に `client_id`/`redirect_uri`/`scopes` が保存されている
- Keyringに `client_secret` が保存されている
- `slack-rs login` 実行時、各値は保存済みから解決され、対話入力は行われない
- いずれかが欠落している場合のみ、その項目の入力が促される
