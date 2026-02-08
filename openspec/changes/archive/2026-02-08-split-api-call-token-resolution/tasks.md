# 実装タスク

- [x] `run_api_call` からトークン解決ロジックを専用関数へ抽出する（検証: `src/cli/handlers.rs` で `run_api_call` から新関数呼び出しへ置換されていることを確認）。
- [x] `SLACK_TOKEN` 優先・明示指定時エラー・user->bot フォールバックを関数単位でテスト化する（検証: `cargo test --lib` で対象テストが成功する）。
- [x] token store 依存は in-memory store の fixture を使うテストへ統一し、外部資格情報なしで再現可能にする（検証: CI 相当環境で再実行して同一結果）。
- [x] API 呼び出しの既存統合挙動が変わらないことを確認する（検証: `cargo test --test api_integration_tests` と `cargo test --test commands_integration` が成功する）。

## Acceptance #1 Failure Follow-up

- [x] `resolve_token` が token store 取得を `SLACK_TOKEN` より先に評価しており仕様の「SLACK_TOKEN 優先」に違反しているため、`src/cli/handlers.rs` の `resolve_token` で `SLACK_TOKEN` を最優先で返す実装に修正する（証拠: `src/cli/handlers.rs:289`-`src/cli/handlers.rs:295`）。
- [x] `SLACK_TOKEN` 優先を検証するテストが実質的に未実装のため、環境変数と store トークンが同時に存在する条件で env が選ばれることを厳密に検証するテストへ修正する（証拠: `test_resolve_token_slack_token_prioritized_over_store` in `src/cli/handlers.rs:1167` は `SLACK_TOKEN` を設定していない）。
