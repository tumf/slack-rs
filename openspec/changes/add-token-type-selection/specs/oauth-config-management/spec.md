# oauth-config-management 変更仕様（既定トークン種別設定）

## ADDED Requirements
### Requirement: 既定トークン種別を設定するコマンドを提供する
`config` サブコマンドでプロファイルの `default_token_type` を設定できること。 (MUST)
#### Scenario: `config set default --token-type user` を実行する
- Given 対象プロファイルが存在する
- When `config set default --token-type user` を実行する
- Then プロファイルに `default_token_type=user` が保存される
