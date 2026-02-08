# 実装タスク

- [ ] `auth login` の引数解析を専用関数/構造体へ抽出し、`run_auth_login` から責務分離する（検証: `run_auth_login` が解析済み結果を受ける構造になっている）。
- [ ] `--cloudflared` と `--ngrok` の排他、必須値欠落、未知オプションを解析ユニットテストで固定化する（検証: `cargo test --lib` の対象テスト成功）。
- [ ] 非対話モード制約（必須オプション、client secret 扱い）をケース別テストで固定化する（検証: `cargo test --test non_interactive_integration` が成功する）。
- [ ] 外部 OAuth 実通信に依存しないよう、引数解析・分岐判定は mock/fixture 前提で検証する（検証: 認証情報なしで全テスト再実行可能）。
- [ ] 既存ログインコマンド互換を確認する（検証: `cargo test --test auth_integration` と `cargo test --test oauth_integration` が成功する）。
