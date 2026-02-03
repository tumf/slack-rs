# タスク

- [ ] プロファイル設定のデータ構造とシリアライズを定義する（検証: `profiles.json` の入出力ユニットテストで期待 JSON を確認）
- [ ] 設定ファイルの保存/読み込みロジックを実装する（検証: temp dir で保存→読み込み一致を確認）
- [ ] TokenStore 抽象を定義し、インメモリ実装を用意する（検証: `set/get/delete` のユニットテスト）
- [ ] keyring 実装を追加し、TokenStore に接続する（検証: keyring 依存はモックで代替し、インメモリ実装のテストで確認）
- [ ] `profile_name -> (team_id, user_id)` の解決関数を実装する（検証: 既存/未存在 profile のユニットテスト）
