## MODIFIED Requirements

### Requirement: Do not start if required configuration is missing

ログイン開始時にOAuthクライアント情報が不足している場合、対話入力で補完できることがMUST。

#### Scenario: Required configuration is missing
- `--client-id` が未指定かつプロファイルに `client_id` が無い場合は対話入力で補完する
- `client_secret` は常に対話入力で取得する
