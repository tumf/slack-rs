# タスク

- [ ] write ガードの判定を `SLACKCLI_ALLOW_WRITE` に切り替える（検証: `src/commands/guards.rs` の unit test で `false/0` 時に `WriteNotAllowed` を返すことを確認）
- [ ] `--allow-write` の CLI フラグ利用を削除する（検証: `src/cli/mod.rs` と `src/main.rs` の usage 表示から `--allow-write` が消えることを確認）
- [ ] write 拒否時のエラーメッセージを環境変数に合わせて更新する（検証: `src/api/client.rs` の `ApiError::WriteNotAllowed` の文言が `SLACKCLI_ALLOW_WRITE` を参照していることを確認）
- [ ] write コマンドの判定経路を環境変数基準に接続する（検証: `msg` / `react` の実行パスで `check_write_allowed` が呼ばれることを確認）
- [ ] README と CLI ドキュメントの説明を更新する（検証: `README.md` に `SLACKCLI_ALLOW_WRITE` の説明が追加され、`--allow-write` 記載が削除されていることを確認）
