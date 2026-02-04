- [x] 1. export/import用のOAuthクレデンシャル取得/保存のヘルパーを追加する
   - Keyringキーは `service=slackcli`、`username=oauth-client-secret:<profile_name>` を使用
   - 検証: Keyringスタブで保存/取得/削除が一致すること
   - 実装: `src/profile/token_store.rs` に `store_oauth_client_secret`, `get_oauth_client_secret`, `delete_oauth_client_secret` を追加

- [x] 2. `ExportProfile` に `client_id`/`client_secret` を追加し、互換的にシリアライズできるようにする
   - `Option` フィールドで欠落を許容
   - 検証: 旧形式ペイロードのデコードが失敗しないこと
   - 実装: `src/auth/format.rs` の `ExportProfile` に `client_id: Option<String>` と `client_secret: Option<String>` を追加

- [x] 3. export処理でOAuthクレデンシャルをペイロードに追加する
   - `client_id` はprofiles設定から取得
   - `client_secret` はKeyringから取得できる場合のみ追加
   - 検証: スタブを使った単体テストでペイロードに含まれること
   - 実装: `src/auth/export_import.rs` の `export_profiles` 関数でOAuthクレデンシャルを含めるように更新

- [x] 4. import処理でOAuthクレデンシャルを復元する
   - `client_id` はprofiles設定に保存
   - `client_secret` はKeyringに保存
   - 競合時の `--yes/--force` に従い上書きされること
   - 検証: スタブを使った単体テストで保存が行われること
   - 実装: `src/auth/export_import.rs` の `import_profiles` 関数でOAuthクレデンシャルを復元

- [x] 5. export/importの暗号化/復号フローに影響がないことを確認する
   - 既存の暗号化/フォーマットテストが通ること
   - 検証: `cargo test auth::`（または該当テスト）で成功すること
   - 結果: すべてのauth::モジュールテスト (40件) が成功

## Acceptance #1 Failure Follow-up
- [x] Import時の`client_secret`保存失敗を握り潰さず、Keyringへの保存が失敗したらエラーとして扱う（`src/auth/export_import.rs`の`import_profiles`で`store_oauth_client_secret`の結果を無視している）
