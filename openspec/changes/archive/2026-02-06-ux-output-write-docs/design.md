# Design: Output, write safety, debug, recipes

## 出力既定の切替
- `SLACKRS_OUTPUT=raw|envelope` を導入し、`--raw` 未指定時の既定を切り替える。
- `--raw` は常に最優先とし、env より強い。

## write 操作の確認
- `SLACKCLI_ALLOW_WRITE` の許可/拒否は維持する。
- write 操作はデフォルトで確認を要求する（TTY のみ）。
- `--yes` で確認をスキップする。
- 非対話（`--non-interactive` または stdin 非TTY）では `--yes` が無ければ即時エラー。
- 対象コマンド: `msg post/update/delete`, `react add/remove`, `file upload`。

## debug/trace
- `--debug` / `--trace` を追加し、stderr に以下を出す:
  - 解決済み profile 名
  - token store backend
  - 解決済み token type
  - API method + endpoint
  - Slack error code（存在する場合）
- 秘密情報（token / client_secret）は出力しない。
- 既存の `SLACK_RS_DEBUG` は互換として残し、フラグ指定時に有効化する。

## help/recipes
- help に copy/paste 可能な例を追加する。
- `docs/recipes.md` を新設し、profile 選択・出力切替・会話/スレッド取得・エラー対処の項目を含める。
