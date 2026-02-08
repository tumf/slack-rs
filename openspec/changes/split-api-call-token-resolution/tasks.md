# 実装タスク

- [x] `run_api_call` からトークン解決ロジックを専用関数へ抽出する（検証: `src/cli/handlers.rs` で `run_api_call` から新関数呼び出しへ置換されていることを確認）。
- [x] `SLACK_TOKEN` 優先・明示指定時エラー・user->bot フォールバックを関数単位でテスト化する（検証: `cargo test --lib` で対象テストが成功する）。
- [x] token store 依存は in-memory store の fixture を使うテストへ統一し、外部資格情報なしで再現可能にする（検証: CI 相当環境で再実行して同一結果）。
- [x] API 呼び出しの既存統合挙動が変わらないことを確認する（検証: `cargo test --test api_integration_tests` と `cargo test --test commands_integration` が成功する）。
