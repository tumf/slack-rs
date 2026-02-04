# タスク

- [ ] `file upload` の CLI ルーティングと引数解析を追加する（検証: `src/main.rs` と `src/cli/mod.rs` に `file upload` の分岐とオプション解析があることを確認）
- [ ] `file upload` コマンド実装で 3 ステップの外部アップロードを行う（検証: `src/commands` 配下の新実装が `files.getUploadURLExternal` → `upload_url` POST → `files.completeUploadExternal` を実行していることを確認）
- [ ] Step2 のアップロードを `reqwest::Client` で生バイト送信する（検証: `Content-Type: application/octet-stream` で送信する実装が存在することを確認）
- [ ] `file upload` の usage/help を更新する（検証: `src/main.rs` と `src/cli/mod.rs` に新コマンドの usage が含まれることを確認）
- [ ] 外部アップロードフローの統合テストを追加する（検証: `tests/` で `files.getUploadURLExternal` / `upload_url` / `files.completeUploadExternal` の順にリクエストが行われることをモックで確認）
