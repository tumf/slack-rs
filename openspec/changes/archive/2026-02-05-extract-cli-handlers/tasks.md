# タスク

- [x] `src/main.rs` の `run_auth_login`/`run_api_call`/`handle_export_command`/`handle_import_command` を新しいハンドラーモジュールへ移動する（確認: `src/main.rs` がディスパッチ中心になり、関数本体が移動している）
- [x] `main.rs` から新ハンドラーを呼び出す配線を追加する（確認: `src/main.rs` で各コマンド分岐が新関数を呼ぶ）
- [x] 既存の公開 API 参照を維持するための re-export/モジュール宣言を更新する（確認: `cargo check` が通る）
- [x] CLI の回帰を防ぐために既存テストを実行する（確認: `cargo test`）
