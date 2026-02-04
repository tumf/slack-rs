1. export/import用のOAuthクレデンシャル取得/保存のヘルパーを追加する
   - Keyringキーは `service=slack-rs`、`username=oauth-client-secret:<profile_name>` を使用
   - 検証: Keyringスタブで保存/取得/削除が一致すること

2. `ExportProfile` に `client_id`/`client_secret` を追加し、互換的にシリアライズできるようにする
   - `Option` フィールドで欠落を許容
   - 検証: 旧形式ペイロードのデコードが失敗しないこと

3. export処理でOAuthクレデンシャルをペイロードに追加する
   - `client_id` はprofiles設定から取得
   - `client_secret` はKeyringから取得できる場合のみ追加
   - 検証: スタブを使った単体テストでペイロードに含まれること

4. import処理でOAuthクレデンシャルを復元する
   - `client_id` はprofiles設定に保存
   - `client_secret` はKeyringに保存
   - 競合時の `--yes/--force` に従い上書きされること
   - 検証: スタブを使った単体テストで保存が行われること

5. export/importの暗号化/復号フローに影響がないことを確認する
   - 既存の暗号化/フォーマットテストが通ること
   - 検証: `cargo test auth::`（または該当テスト）で成功すること
