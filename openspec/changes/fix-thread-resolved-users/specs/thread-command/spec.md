## MODIFIED Requirements

### Requirement: `thread get` は統一エンベロープを既定出力し `--raw` で生レスポンスを返す

既定の `thread get` JSON 出力は、Slack の `messages` 配列を改変せずに統一エンベロープを返さなければならない。(MUST)
既定出力では、スレッド全体で参照されたユーザーを `response.data.resolved_users` に wrapper-owned metadata として追加してよい。(MUST)
`--raw` が指定された場合は、`resolved_users` を含めず Slack API の生レスポンスを返さなければならない。(MUST)

#### Scenario: 既定出力は message object を改変せず response-level user resolution を含む
- Given `thread get` の取得結果に複数メッセージとユーザー参照が含まれる
- When `--raw` を指定せずに `slack-rs thread get` を実行する
- Then `response.data.messages` の各 message object に CLI が追加した `users` 配列は含まれない
- And `response.data.resolved_users` に解決済みユーザー情報が格納される

#### Scenario: `--raw` は wrapper-owned user resolution を含まない
- Given `thread get` の取得結果にユーザー参照が含まれる
- When `slack-rs thread get --raw` を実行する
- Then 出力は Slack API の raw レスポンスである
- And `resolved_users` や `unresolved_user_ids` のような wrapper-owned フィールドは含まれない

## ADDED Requirements

### Requirement: `thread get` はスレッド参照ユーザーを response-level で解決する

`thread get` は、スレッド全体で参照された Slack user ID を収集し、既定出力の `response.data.resolved_users` に解決済みプロフィール情報を格納しなければならない。(MUST)
収集対象には `message.user`, `text` 内の `<@USER_ID>`, `blocks` 内の `<@USER_ID>`, および `reactions[].users[]` を含めなければならない。(MUST)
解決済みでない ID を hydrated profile と誤認させる ID-only object を `resolved_users` に入れてはならない。(MUST)

#### Scenario: message author, mentions, and reactions are deduplicated into `resolved_users`
- Given あるスレッドのメッセージ本文・blocks・reactions に同じ user ID が複数回出現する
- When `slack-rs thread get` を既定出力で実行する
- Then `response.data.resolved_users` は user ID ごとに 1 件だけ profile entry を持つ
- And reaction actor の user ID も `resolved_users` の対象に含まれる

#### Scenario: unresolved IDs are not exposed as fake hydrated profiles
- Given 収集された user ID の一部が cache に存在せず `users.info` でも解決できない
- When `slack-rs thread get` を既定出力で実行する
- Then 解決不能な ID は `resolved_users` に `{ "id": ... }` だけの profile object として含まれない
- And 必要に応じて `unresolved_user_ids` のような別フィールドで区別される
