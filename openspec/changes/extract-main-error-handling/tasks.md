# 実装タスク

- [ ] `src/main.rs` の重複エラー終了箇所を洗い出し、共通ヘルパーの適用対象を明確化する（検証: 対象分岐一覧を差分で確認できる）。
- [ ] 失敗出力と終了処理を共通ヘルパーへ抽出し、対象分岐を置換する（検証: `cargo test --test commands_integration` が成功する）。
- [ ] 非対話エラー時の終了コード 2 を必要とするコマンドの回帰を確認する（検証: `cargo test --test non_interactive_integration` が成功する）。
- [ ] エラー文言互換を主要コマンドで確認する（検証: `cargo test --test output_envelope_tests` と手動確認で stderr フォーマット差分がない）。
