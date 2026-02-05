## ADDED Requirements
### Requirement: Wrapper commands output is normalized
ラッパーコマンドの JSON 出力は `response`/`meta` のエンベロープで返し、`meta.profile_name`, `meta.team_id`, `meta.user_id`, `meta.method`, `meta.command` を含むこと。`--raw` が指定された場合は Slack API レスポンスをそのまま返すこと。(MUST)

#### Scenario: `conv list` は統一フォーマットで返る
- Given 有効な profile と token が存在する
- When `conv list` を実行する
- Then `meta.command` は `conv list` である
- And `meta.method` は `conversations.list` である
- And `response` に Slack API レスポンスが入る

#### Scenario: `--raw` 指定時は Slack API レスポンスを返す
- Given 有効な profile と token が存在する
- When `conv list --raw` を実行する
- Then 出力は Slack API レスポンスの JSON そのままである
- And `meta` フィールドは含まれない
