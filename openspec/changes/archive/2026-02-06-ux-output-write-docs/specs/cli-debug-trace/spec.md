# cli-debug-trace Specification

## Purpose
実行時の解決結果を可視化し、問題切り分けを容易にする。

## ADDED Requirements

### Requirement: `--debug` / `--trace` shows resolved context
`--debug` または `--trace` が指定された場合、stderr に以下を出力しなければならない。(MUST)
- 解決済み profile 名
- token store backend
- 解決済み token type
- API method と endpoint
- Slack error code（存在する場合）
秘密情報（token, client_secret）は出力してはならない。(MUST NOT)

#### Scenario: `--debug` で解決情報を表示する
- Given `--debug` が指定されている
- When `api call auth.test` を実行する
- Then stderr に解決済み profile 名と token store backend が出力される
- And token の値は出力されない
