# タスク

- [ ] `msg post` の引数解析に `--thread-ts` と `--reply-broadcast` を追加する（検証: `src/cli/mod.rs` で新しいオプションが取得されることを確認）
- [ ] `--reply-broadcast` 単独指定時はエラーにする（検証: `src/cli/mod.rs` の分岐で `--thread-ts` が未指定の場合にエラーを返すことを確認）
- [ ] `msg post` から `thread_ts` / `reply_broadcast` を `chat.postMessage` に渡す（検証: `src/commands/msg.rs` でパラメータが追加されていることを確認）
- [ ] `msg` の usage / help 表示を更新する（検証: `src/cli/mod.rs` と `src/main.rs` の usage 表示に新オプションが含まれることを確認）
- [ ] モックサーバを使って `chat.postMessage` の送信パラメータを検証するテストを追加する（検証: `tests/commands_integration.rs` などで `thread_ts` と `reply_broadcast` がリクエストに含まれることを確認）
