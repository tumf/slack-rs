- [x] `src/cli/mod.rs` のトークン解決に `SLACK_TOKEN` 優先ロジックを追加し、`--token-type`/`default_token_type` の解決順は維持する（確認: `conv list` などが `SLACK_TOKEN` を使うことをテストで検証）
- [x] ラッパーコマンド用のテストを追加し、`SLACK_TOKEN` 設定時に Authorization ヘッダーが環境変数のトークンになることを `httpmock`/`wiremock` で確認する（確認: 新規テストがパス）
- [x] `auth status` の表示を `profile.default_token_type` 優先に修正し、未設定時のみ従来の推測ロジックを使用する（確認: `tests/auth_integration.rs` などの期待値更新）
- [x] 既存テストの調整と `cargo test` の実行（確認: `cargo test` が成功）

## Acceptance #1 Failure Follow-up
- [x] `SLACK_TOKEN` 使用時でも `default_token_type` を解決してメタ情報として保持できるようにし、ラッパーコマンドの出力/内部メタに `token_type` を反映する（例: `CommandMeta` に `token_type` を追加し `wrap_with_envelope` で設定）
- [x] 作業ツリーをクリーンにする（未コミット変更: `openspec/changes/align-token-resolution/tasks.md`）
