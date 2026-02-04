# oauth-login 変更差分

## ADDED Requirements

### Requirement: OAuth設定の解決に環境変数を用いない

OAuth設定はCLI引数またはプロファイル設定ファイルから解決し、環境変数を参照しないことがMUST。

#### Scenario: 環境変数が設定されていても無視される
- 環境変数に `SLACKRS_CLIENT_ID` 等が設定されている
- プロファイル設定に `client_id` が存在する
- `slack-rs login` 実行時、環境変数は参照されず、プロファイル設定が使用される
