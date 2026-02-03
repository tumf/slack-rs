# タスク

- [ ] `search` コマンドを実装する（検証: モックサーバで `search.messages` が呼ばれることを確認）
- [ ] `conv list` / `conv history` を実装する（検証: モックサーバで `conversations.list` / `conversations.history` が呼ばれることを確認）
- [ ] `users info` を実装する（検証: モックサーバで `users.info` が呼ばれることを確認）
- [ ] write ガード（`--allow-write`）を実装する（検証: `--allow-write` なしで write コマンドが失敗するテスト）
- [ ] `msg post/update/delete` を実装する（検証: モックサーバで `chat.postMessage` / `chat.update` / `chat.delete` が呼ばれることを確認）
- [ ] `react add/remove` を実装する（検証: モックサーバで `reactions.add` / `reactions.remove` が呼ばれることを確認）
- [ ] 破壊操作の確認（`--yes`）を実装する（検証: `msg delete` で `--yes` なしの場合に確認が必要なことをテスト）
- [ ] CLI ルーティングに各コマンドを接続する（検証: `--help` に各コマンドが表示されることを確認）
