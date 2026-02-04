## 設計方針

### データモデル
- `Profile` に `client_id: Option<String>` を追加する。
- 既存の `profiles.json` との互換性を保つため、`client_id` はOptionalにする。
- `ProfilesConfig.version` は据え置き（`1`）とし、将来の移行が必要になった場合にのみ更新する。

### シークレット保存
- `client_secret` は設定ファイルに保存しない。
- Keyringに保存し、プロファイル名に紐づくキーで取得できるようにする。
- `keyring` の `service` は `slack-rs`、`username` は `oauth-client-secret:<profile_name>` とする。

### 入力フロー
- `login` で `--client-id` が指定されていない場合、対話的に入力を求める。
- 既存プロファイルに `client_id` が保存されている場合はそれを優先する。
- `client_secret` は常に対話的入力（`rpassword` で非表示）とする。
- `client_secret` のCLI引数や環境変数フォールバックは実装しない。

### 既存プロファイルの扱い
- 既存の `profiles.json` に `client_id` が無くても読み込み可能とする。
- 既存プロファイルでログインを再実行した場合に `client_id` を保存する。

### エラーハンドリング
- `client_id`/`client_secret` が空のまま入力された場合は再入力を促す。
- Keyring保存に失敗した場合はログインを失敗として終了する。

### テスト容易性
- 対話入力を抽象化した関数に分離し、テスト時は入力スタブを注入できるようにする。
- Keyring操作は既存の `TokenStore` と同様にスタブ実装で検証できるようにする。
