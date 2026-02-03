# タスク

- [x] プロファイル設定のデータ構造とシリアライズを定義する（検証: `profiles.json` の入出力ユニットテストで期待 JSON を確認）
- [x] 設定ファイルの保存/読み込みロジックを実装する（検証: temp dir で保存→読み込み一致を確認）
- [x] TokenStore 抽象を定義し、インメモリ実装を用意する（検証: `set/get/delete` のユニットテスト）
- [x] keyring 実装を追加し、TokenStore に接続する（検証: keyring 依存はモックで代替し、インメモリ実装のテストで確認）
- [x] `profile_name -> (team_id, user_id)` の解決関数を実装する（検証: 既存/未存在 profile のユニットテスト）
- [x] すべてのユニットテストを実行して検証する（検証: `cargo test` が成功、`cargo clippy` が警告なし、`cargo build` が成功）
