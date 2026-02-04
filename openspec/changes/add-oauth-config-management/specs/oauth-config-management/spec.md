# oauth-config-management 変更差分

## ADDED Requirements

### Requirement: OAuth設定をCLIで管理できる

プロファイルごとのOAuth設定をCLIで設定・確認・削除できることがMUST。

#### Scenario: set/show/delete が提供される
- `slackrs config oauth set --profile <name>` で設定を保存できる
- `slackrs config oauth show --profile <name>` で保存内容を確認できる
- `slackrs config oauth delete --profile <name>` で設定を削除できる

### Requirement: client_secret はKeyringに保存し、表示しない

`client_secret` は設定ファイルに保存せずKeyringに保存し、`show` コマンドでも出力しないことがMUST。

#### Scenario: secretが出力されない
- `config oauth set` 実行時に `client_secret` を入力する
- `profiles.json` には `client_secret` が含まれない
- `config oauth show` の出力に `client_secret` が含まれない
