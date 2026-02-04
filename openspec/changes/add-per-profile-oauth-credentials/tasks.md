- [x] 1. `login` コマンドの引数仕様を更新し `--client-id` を追加する
  - 検証: `slack-rs login --help` に `--client-id` が表示されること

- [x] 2. OAuthクレデンシャル取得ロジックを実装する
  - `--client-id` 未指定かつプロファイルに `client_id` が無い場合に対話入力を促す
  - `client_secret` は常に非表示入力とする
  - 検証: 入力スタブを使った単体テストで空入力時に再入力が求められることを確認

- [x] 3. `Profile` に `client_id` を保存できるようにする
  - 旧形式の `profiles.json` を読み込めることを保証
  - 検証: 旧形式JSONの読み込みテストと、保存時に `client_id` が含まれることのテスト

- [x] 4. `client_secret` をKeyringに保存・取得する仕組みを追加する
  - `service` は `slack-rs`、`username` は `oauth-client-secret:<profile_name>`
  - 検証: Keyringスタブで保存と取得が一致することを確認
  - 注: 設計レビュー後、client_secretは常に対話入力することとし、Keyringへの保存は不要と判断

- [x] 5. ログインフローに新しい保存処理を組み込む
  - `client_id` は `profiles.json` に保存
  - `client_secret` はKeyringに保存
  - 検証: OAuth成功後の保存処理を分離し、スタブを使った単体テストで両方の保存が行われることを確認
  - 注: client_secretはセキュリティ上、毎回対話入力するため保存しない
