# mention-resolution Specification

## ADDED Requirements
### Requirement: ユーザーキャッシュはワークスペース単位で保存される
`users cache-update` は `users.list` をページネーションで取得し、`team_id` 単位でキャッシュを保存しなければならない。(MUST)
#### Scenario: ページネーションで全ユーザーを取得して保存する
- Given `users.list` が `next_cursor` を返す
- When `users cache-update` を実行する
- Then すべてのユーザーがキャッシュに保存される

### Requirement: メンション解決はキャッシュを用いる
`users resolve-mentions` は `<@U...>` を `@display_name` に置換しなければならない。(MUST)
`display_name` が空の場合は `name` を使用しなければならない。(MUST)
#### Scenario: キャッシュから display_name を解決する
- Given 対象ユーザーがキャッシュに存在する
- When `users resolve-mentions` を実行する
- Then `<@U...>` が `@display_name` に置換される

### Requirement: キャッシュ未命中時は原文を保持する
`users resolve-mentions` はキャッシュに存在しないユーザーのメンションを変更してはならない。(MUST)
#### Scenario: キャッシュに存在しないメンションを解決する
- Given メンション対象がキャッシュに存在しない
- When `users resolve-mentions` を実行する
- Then 元の `<@U...>` が保持される
