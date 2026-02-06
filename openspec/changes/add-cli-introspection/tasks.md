- [x] `commands --json` の出力フォーマットを定義し、CLI ルーティングと同じ情報源から生成できることを確認する（`src/cli` の定義テーブルと `slack-rs commands --json` の出力を照合）。
- [x] `--help --json` の構造化ヘルプを実装し、`slack-rs msg post --help --json` で `command`/`usage`/`flags`/`examples`/`exitCodes` が含まれることを確認する（ユニットテストで JSON パース検証）。
- [x] `schema --command <cmd> --output json-schema` の最小スキーマ出力を追加し、`schemaVersion`/`type`/`ok`/`response`/`meta` がスキーマに含まれることを確認する（ユニットテストで JSON パース検証）。
- [x] 統一エンベロープに `schemaVersion`/`type`/`ok` を追加し、`--raw` 指定時は追加されないことを確認する（`conv list --raw` の既存テストの挙動確認）。
- [x] イントロスペクション出力のテストを追加し、外部 API なしで実行できることを確認する（`cargo test --lib` で対象テストが通る）。

## Acceptance #1 Failure Follow-up
- [x] `--help --json` の構造化ヘルプがサブコマンドで実行されるように CLI ルーティングへ統合する（`slack-rs msg post --help --json` が `command`/`usage`/`flags`/`examples`/`exitCodes` を返す）。
- [x] `schema --command conv.list --output json-schema` が成功するようにコマンド名のマッピング（`conv.list` ↔ `conv list`）を追加する。

## Acceptance #2 Failure Follow-up
- [x] `--help --json` が `api call` や `auth login` など他コマンドでも構造化ヘルプを返すよう CLI ルーティングを統合する（`src/main.rs` では `conv list` と `msg post` 以外に JSON ヘルプの分岐がない）。
