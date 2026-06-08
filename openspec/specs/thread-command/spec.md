# thread-command Specification

## Purpose
TBD - created by archiving change add-thread-get-command. Update Purpose after archive.
## Requirements

### Requirement: `thread get` は `conversations.replies` をラップしてスレッドを取得できる

`thread get` MUST 指定されたチャンネルとスレッドタイムスタンプを用いて `conversations.replies` を実行し、対象スレッドのメッセージを取得する。

#### Scenario: channel と thread_ts を指定してスレッドを取得する
Given `slack-rs thread get C123456 1700000000.000001` を実行する
When CLI が `conversations.replies` を呼び出す
Then `channel=C123456` と `ts=1700000000.000001` が送信される

### Requirement: `thread get` は `--limit` と `--inclusive` を API パラメータとして渡す

`--limit` と `--inclusive` の指定内容 MUST `conversations.replies` のリクエストパラメータに反映される。

#### Scenario: `--limit` と `--inclusive` を指定して取得する
Given `slack-rs thread get C123456 1700000000.000001 --limit=50 --inclusive` を実行する
When CLI が `conversations.replies` を呼び出す
Then `limit=50` と `inclusive=true` が送信される

### Requirement: `thread get` はカーソルページネーションを追従し messages を集約する

`response_metadata.next_cursor` が返る場合、CLI MUST 追加ページを取得し、全ページの `messages` を結合して返す。

#### Scenario: `next_cursor` が返る場合に複数ページを統合する
Given 1 ページ目のレスポンスに `response_metadata.next_cursor` が含まれる
When CLI が追加ページを取得する
Then `messages` は全ページ分の配列として返される

### Requirement: `thread get` は統一エンベロープを既定出力し `--raw` で生レスポンスを返す

既定の `thread get` JSON 出力は、Slack の `messages` 配列を改変せずに統一エンベロープを返さなければならない。(MUST)
既定出力では、スレッド全体で参照されたユーザーを `response.resolved_users` に wrapper-owned metadata として追加してよい。(MUST)
`--raw` が指定された場合は、`resolved_users` を含めず Slack API の生レスポンスを返さなければならない。(MUST)

#### Scenario: 既定出力は message object を改変せず response-level user resolution を含む
- Given `thread get` の取得結果に複数メッセージとユーザー参照が含まれる
- When `--raw` を指定せずに `slack-rs thread get` を実行する
- Then `response.data.messages` の各 message object に CLI が追加した `users` 配列は含まれない
- And `response.resolved_users` に解決済みユーザー情報が格納される

#### Scenario: `--raw` は wrapper-owned user resolution を含まない
- Given `thread get` の取得結果にユーザー参照が含まれる
- When `slack-rs thread get --raw` を実行する
- Then 出力は Slack API の raw レスポンスである
- And `resolved_users` や `unresolved_user_ids` のような wrapper-owned フィールドは含まれない

### Requirement: `thread get` はヘルプとイントロスペクションに表示される

`commands --json` と `--help --json` の結果に `thread get` が含まれることを CLI MUST 保証する。

#### Scenario: `commands --json` と `--help --json` に `thread get` が含まれる
Given `slack-rs commands --json` を実行する
When コマンド一覧を取得する
Then `thread get` が含まれる

### Requirement: `thread get` はスレッド参照ユーザーを response-level で解決する

`thread get` は、スレッド全体で参照された Slack user ID を収集し、既定出力の `response.resolved_users` に解決済みプロフィール情報を格納しなければならない。(MUST)
収集対象には `message.user`, `text` 内の `<@USER_ID>`, `blocks` 内の `<@USER_ID>`, および `reactions[].users[]` を含めなければならない。(MUST)
解決済みでない ID を hydrated profile と誤認させる ID-only object を `resolved_users` に入れてはならない。(MUST)
`conversations.replies` が成功した後の wrapper-owned user resolution 失敗は、`thread get` 全体を失敗させず、解決不能 ID として `response.unresolved_user_ids` に格納しなければならない。(MUST)
`--raw` 指定時は wrapper-owned user resolution を実行せず、`resolved_users` と `unresolved_user_ids` を出力に含めてはならない。(MUST)

#### Scenario: message author, mentions, and reactions are deduplicated into `resolved_users`

- Given あるスレッドのメッセージ本文・blocks・reactions に同じ user ID が複数回出現する
- When `slack-rs thread get` を既定出力で実行する
- Then `response.resolved_users` は user ID ごとに 1 件だけ profile entry を持つ
- And reaction actor の user ID も `resolved_users` の対象に含まれる

#### Scenario: unresolved IDs are not exposed as fake hydrated profiles

- Given 収集された user ID の一部が cache に存在せず `users.info` でも解決できない
- When `slack-rs thread get` を既定出力で実行する
- Then 解決不能な ID は `resolved_users` に `{ "id": ... }` だけの profile object として含まれない
- And 解決不能な ID は `response.unresolved_user_ids` に含まれる
- And `thread get` は `conversations.replies` の成功レスポンスを失敗に変換しない

#### Scenario: CLI wrapper output preserves raw messages while reporting unresolved users

- Given `conversations.replies` がメッセージを返し、参照ユーザーの一部だけが解決できる
- When `--raw` を指定せずに `slack-rs thread get` を実行する
- Then 出力は統一エンベロープである
- And `response.data.messages` の各 message object に CLI-owned `users` 配列は含まれない
- And `response.resolved_users` には実際に解決できた profile だけが含まれる
- And `response.unresolved_user_ids` には解決できなかった user ID が含まれる

#### Scenario: raw output skips wrapper-owned user resolution

- Given `conversations.replies` の取得結果にユーザー参照が含まれる
- When `slack-rs thread get --raw` を実行する
- Then 出力は Slack API の raw レスポンスである
- And `resolved_users` や `unresolved_user_ids` は含まれない
- And user resolution の失敗は raw 出力を失敗させない
