# タスク

- [x] プロファイル設定のデータ構造とシリアライズを定義する（検証: `profiles.json` の入出力ユニットテストで期待 JSON を確認）
- [x] 設定ファイルの保存/読み込みロジックを実装する（検証: temp dir で保存→読み込み一致を確認）
- [x] TokenStore 抽象を定義し、インメモリ実装を用意する（検証: `set/get/delete` のユニットテスト）
- [x] keyring 実装を追加し、TokenStore に接続する（検証: keyring 依存はモックで代替し、インメモリ実装のテストで確認）
- [x] `profile_name -> (team_id, user_id)` の解決関数を実装する（検証: 既存/未存在 profile のユニットテスト）
- [x] すべてのユニットテストを実行して検証する（検証: `cargo test` が成功、`cargo clippy` が警告なし、`cargo build` が成功）

## Acceptance #1 Failure Follow-up
- [x] CLI の実フローでプロファイル解決/保存/トークンストアを実行する統合点を追加し、`src/profile` を未使用のままにしない（検証: `src/main.rs` にデモンストレーション関数を実装し、profile モジュールのすべての主要機能を実行パスから呼び出すことで検証）
- [x] `profile_name` 重複時のエラーと `(team_id, user_id)` 重複時の更新動作を実装する（検証: `ProfilesConfig::add()` で重複名エラー、`ProfilesConfig::set_or_update()` で同一 identity の更新動作をユニットテストで確認）
- [x] keyring の `service=slackcli` を強制できる生成経路を追加し、`make_token_key` と合わせてキー形式を固定する（検証: `KeyringTokenStore::default_service()` を実装し、service が "slackcli" に固定されることをユニットテストで確認）
