# 実装タスク

- [ ] `src/commands/file.rs` に `file_download`（`<file_id>` / `--url` 両対応）を追加し、`files.info` による URL 解決と認証付き GET を実装する（完了確認: 追加したテストで `url_private_download` 優先・`url_private` fallback が通る）。
- [ ] `src/commands/file.rs` に出力処理（`--out <path>` 保存 / `--out -` stdout ストリーム）と安全なファイル名決定ロジックを追加する（完了確認: テストで保存先ファイルのバイト一致と `--out -` のバイナリ出力を確認）。
- [ ] `src/commands/file.rs` にエラー処理を追加し、非 2xx と `Content-Type: text/html` を明確な失敗として扱う（完了確認: モック応答を使う失敗系テストが通り、エラーメッセージに原因ヒントが含まれる）。
- [ ] `src/cli/mod.rs` に `run_file_download` を追加し、`--url` / `--out` / `--profile` / `--token-type` を既存規約で解釈して `commands::file_download` を呼び出す（完了確認: `run_file_upload` と同様の経路で `file download` が呼ばれることを CLI テストまたは結合テストで確認）。
- [ ] `src/main.rs` の `file` サブコマンドに `download` を配線し、`print_file_usage` と `src/cli/introspection.rs` のコマンド定義を更新する（完了確認: `slack-rs --help` / `slack-rs commands --json` / `slack-rs --help --json` に `file download` が現れる）。
- [ ] read 操作として `SLACKCLI_ALLOW_WRITE` 非依存であることを検証するテストを追加する（完了確認: `SLACKCLI_ALLOW_WRITE=false` のテスト環境でも `file download` の前段バリデーションが write 拒否で失敗しない）。
- [ ] 回帰確認として関連テストと静的検証を実行する（完了確認: `cargo test` と `cargo clippy -- -D warnings` が成功し、既存 `file upload` テストが維持される）。

## Future Work

- [ ] 実 Slack ワークスペース上での大容量ファイル/多様な MIME の長時間検証は、本提案では外部依存が大きいため将来タスクとして扱う。
