# タスク

- [x] `search` コマンドを実装する（検証: モックサーバで `search.messages` が呼ばれることを確認）
- [x] `conv list` / `conv history` を実装する（検証: モックサーバで `conversations.list` / `conversations.history` が呼ばれることを確認）
- [x] `users info` を実装する（検証: モックサーバで `users.info` が呼ばれることを確認）
- [x] write ガード（`--allow-write`）を実装する（検証: `--allow-write` なしで write コマンドが失敗するテスト）
- [x] `msg post/update/delete` を実装する（検証: モックサーバで `chat.postMessage` / `chat.update` / `chat.delete` が呼ばれることを確認）
- [x] `react add/remove` を実装する（検証: モックサーバで `reactions.add` / `reactions.remove` が呼ばれることを確認）
- [x] 破壊操作の確認（`--yes`）を実装する（検証: `msg delete` で `--yes` なしの場合に確認が必要なことをテスト）
- [x] CLI ルーティングに各コマンドを接続する（検証: `--help` に各コマンドが表示されることを確認）

## Acceptance #1 Failure Follow-up

- [x] `search` が `search.messages` に `sort` と `sort_dir` を渡していないため、CLI オプションと API パラメータの両方を追加する（`src/cli/mod.rs` の `run_search` と `src/commands/search.rs` の `search` を更新）
