## MODIFIED Requirements

### Requirement: `thread get` はスレッド参照ユーザーを response-level で解決する

`thread get` は、スレッド全体で参照された Slack user ID を収集し、既定出力の `response.data.resolved_users` に解決済みプロフィール情報を格納しなければならない。(MUST)
収集対象には `message.user`, `text` 内の `<@USER_ID>`, `blocks` 内の `<@USER_ID>`, および `reactions[].users[]` を含めなければならない。(MUST)
解決済みでない ID を hydrated profile と誤認させる ID-only object を `resolved_users` に入れてはならない。(MUST)
`conversations.replies` が成功した後の wrapper-owned user resolution 失敗は、`thread get` 全体を失敗させず、解決不能 ID として `response.data.unresolved_user_ids` に格納しなければならない。(MUST)
`--raw` 指定時は wrapper-owned user resolution を実行せず、`resolved_users` と `unresolved_user_ids` を出力に含めてはならない。(MUST)

#### Scenario: message author, mentions, and reactions are deduplicated into `resolved_users`

- Given あるスレッドのメッセージ本文・blocks・reactions に同じ user ID が複数回出現する
- When `slack-rs thread get` を既定出力で実行する
- Then `response.data.resolved_users` は user ID ごとに 1 件だけ profile entry を持つ
- And reaction actor の user ID も `resolved_users` の対象に含まれる

#### Scenario: unresolved IDs are not exposed as fake hydrated profiles

- Given 収集された user ID の一部が cache に存在せず `users.info` でも解決できない
- When `slack-rs thread get` を既定出力で実行する
- Then 解決不能な ID は `resolved_users` に `{ "id": ... }` だけの profile object として含まれない
- And 解決不能な ID は `response.data.unresolved_user_ids` に含まれる
- And `thread get` は `conversations.replies` の成功レスポンスを失敗に変換しない

#### Scenario: CLI wrapper output preserves raw messages while reporting unresolved users

- Given `conversations.replies` がメッセージを返し、参照ユーザーの一部だけが解決できる
- When `--raw` を指定せずに `slack-rs thread get` を実行する
- Then 出力は統一エンベロープである
- And `response.data.messages` の各 message object に CLI-owned `users` 配列は含まれない
- And `response.data.resolved_users` には実際に解決できた profile だけが含まれる
- And `response.data.unresolved_user_ids` には解決できなかった user ID が含まれる

#### Scenario: raw output skips wrapper-owned user resolution

- Given `conversations.replies` の取得結果にユーザー参照が含まれる
- When `slack-rs thread get --raw` を実行する
- Then 出力は Slack API の raw レスポンスである
- And `resolved_users` や `unresolved_user_ids` は含まれない
- And user resolution の失敗は raw 出力を失敗させない
