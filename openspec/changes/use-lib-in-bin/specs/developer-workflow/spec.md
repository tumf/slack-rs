# developer-workflow

## MODIFIED Requirements

### Requirement: バイナリはライブラリ公開 API を利用する
CLI バイナリは、同一の機能実装を重複して保持せず、`slack_rs` ライブラリの公開モジュールを利用して構成されなければならない。(MUST)

#### Scenario: `cargo test` の実行で重複が発生しない
- Given `src/main.rs` が `slack_rs::` 経由でモジュールを参照している
- When `cargo test --quiet` を実行する
- Then 同一テスト群が二重に出力されない
