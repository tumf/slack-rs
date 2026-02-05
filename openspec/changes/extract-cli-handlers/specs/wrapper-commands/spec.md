# wrapper-commands 変更提案

## MODIFIED Requirements
### Requirement: CLI ルーティングは挙動を維持したままハンドラー分離される
`slack-rs` の `auth`/`api`/`auth export`/`auth import` のコマンドは、内部の実装がハンドラー分離されても同じ引数と出力で動作しなければならない。(MUST)

#### Scenario:
- Given 既存の CLI 引数と入力を使用する
- When `slack-rs auth login` や `slack-rs api call` を実行する
- Then 以前と同じ成功/失敗判定と出力形式で応答する
