# profiles-and-token-store

## MODIFIED Requirements

### Requirement: プロファイル設定ファイルの既定パスは slack-rs を使用する
プロファイルの非秘密情報は `slack-rs` の設定ディレクトリ配下の `profiles.json` に保存されなければならない。(MUST)

#### Scenario: 既定パスを解決する
- Given 既定の設定パスを取得する
- When OS の設定ディレクトリを参照する
- Then パスに `slack-rs` と `profiles.json` が含まれる

### Requirement: 旧パスの設定ファイルは新パスへ移行される
新パスが存在せず旧パス（`slack-cli`）の `profiles.json` が存在する場合、設定ファイルは新パスへ移行されなければならない。(MUST)

#### Scenario: 旧パスのみ存在する場合に読み込みを行う
- Given 旧パスに `profiles.json` が存在し、新パスには存在しない
- When 設定ファイルを読み込む
- Then 新パスに `profiles.json` が作成され、同一の内容が読み込まれる
