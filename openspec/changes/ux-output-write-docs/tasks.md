- [x] `SLACKRS_OUTPUT=raw|envelope` を導入し、`--raw` 未指定時の既定出力を切り替える（確認: `api call` と wrapper コマンドの出力分岐に env を反映）
- [x] write 操作に統一の確認フローを追加する（確認: `msg post/update/delete`、`react add/remove`、`file upload` が `--yes` でスキップ可能）
- [x] 非対話時に `--yes` がない write 操作を即時エラーにする（確認: `--non-interactive` でのエラーメッセージと終了経路）
- [x] `--debug` / `--trace` を追加し、解決済み情報を stderr に出す（確認: debug ログの出力項目と secret が出ないこと）
- [x] help 出力に代表例を追加する（確認: `slack-rs --help` および主要コマンドの `--help` 表示）
- [x] `docs/recipes.md` を追加し、profile/出力/会話/エラーの項目を含める（確認: ファイル内容に指定見出しがある）

## Acceptance #1 Failure Follow-up
- [ ] Clean git working tree: commit or revert `docs/recipes.md`, `openspec/changes/ux-output-write-docs/tasks.md`, `src/api/args.rs`, `src/cli/handlers.rs`, `src/cli/mod.rs`, `src/debug.rs`, `src/main.rs`
- [ ] Add confirmation + non-interactive `--yes` enforcement for `msg post` in `src/commands/msg.rs` and CLI flow
- [ ] Add confirmation + non-interactive `--yes` enforcement for `react add` in `src/commands/react.rs` and CLI flow
- [ ] Add confirmation + non-interactive `--yes` enforcement for `file upload` in `src/commands/file.rs` and CLI flow
- [ ] Add profile selection example to top-level help output in `src/main.rs` `print_help`
