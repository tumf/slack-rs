## 設計方針

### データモデル拡張
- `ExportProfile` に `client_id: Option<String>` と `client_secret: Option<String>` を追加する。
- 既存エクスポートとの互換性のため、追加フィールドはOptionalにする。

### 保存先の扱い
- `client_id` はprofiles設定に保存される非機密情報として扱う。
- `client_secret` はKeyringに保存される機密情報として扱い、設定ファイルには保存しない。
- Keyringの識別子は `service = "slack-rs"`、`username = "oauth-client-secret:<profile_name>"` を用いる。

### exportの挙動
- `client_id` が存在する場合のみペイロードに含める。
- `client_secret` はKeyringから取得できる場合のみペイロードに含める。
- 取得できない場合はペイロードから省略し、エクスポート自体は継続する。

### importの挙動
- ペイロードに `client_id` があればprofiles設定に保存する。
- ペイロードに `client_secret` があればKeyringへ保存する。
- 競合時の `--yes/--force` の振る舞いは既存仕様に従い、上書き時はOAuthクレデンシャルも更新する。

### 互換性
- 既存のexportファイル（OAuthクレデンシャル無し）を読み込めること。
- 既存profiles.json（`client_id`無し）を読み込めること。

### テスト容易性
- Keyringへの保存/取得はスタブ実装で検証できるようにする。
- export/importのコア処理は副作用を分離し、単体テストで確認できるようにする。
