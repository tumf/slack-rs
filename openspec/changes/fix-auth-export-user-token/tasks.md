1. Export フォーマットに `user_token` を追加し、既存フォーマットと互換のまま読み書きできるようにする
   - 対象: `src/auth/format.rs` の `ExportProfile`
   - 検証: `ExportProfile` に `user_token: Option<String>` が追加され、serde の `skip_serializing_if` で既存互換を維持していることを確認

2. `auth export` で bot/user トークンを両方取得し、どちらかが存在すればエクスポート対象とする
   - 対象: `src/auth/export_import.rs` の `export_profiles`
   - 検証: bot または user のいずれかのトークンがある場合に `ExportProfile` が作成されることをテストで確認

3. `auth import` で `user_token` を専用キーへ復元する
   - 対象: `src/auth/export_import.rs` の `import_profiles`
   - 検証: `user_token` 付き export を import したときに `team_id:user_id:user` に保存されることをテストで確認

4. 追加・更新テストを実装する
   - 対象: `src/auth/export_import.rs` のテスト
   - 検証: `cargo test export_import` または該当テスト名で成功すること
