# 実装タスク

- [x] `auth login` の引数解析を専用関数/構造体へ抽出し、`run_auth_login` から責務分離する（検証: `run_auth_login` が解析済み結果を受ける構造になっている）。
- [x] `--cloudflared` と `--ngrok` の排他、必須値欠落、未知オプションを解析ユニットテストで固定化する（検証: `cargo test --lib` の対象テスト成功）。
- [x] 非対話モード制約（必須オプション、client secret 扱い）をケース別テストで固定化する（検証: `cargo test --test non_interactive_integration` が成功する）。
- [x] 標準ログインと拡張ログインで重複する OAuth 実行手順を単一コア関数へ統合する（検証: 両経路が同一コア呼び出しを利用するコード経路を確認）。
- [x] OAuth 通信部分は mock/stub で検証可能な単位に分け、外部クレデンシャル不要でテスト可能にする（検証: fixture ベーステストが `cargo test --lib` で成功）。
- [x] 既存ログインコマンド互換を確認する（検証: `cargo test --test auth_integration` と `cargo test --test oauth_integration` と `cargo test --test manifest_generation_integration` が成功する）。
