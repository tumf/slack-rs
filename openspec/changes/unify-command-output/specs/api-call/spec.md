## MODIFIED Requirements
### Requirement: Include meta in output
出力 JSON は `meta.profile_name`, `meta.team_id`, `meta.user_id`, `meta.method` に加えて `meta.command` を含むこと。`--raw` が指定されていない場合は `response`/`meta` のエンベロープで出力すること。`--raw` が指定された場合は Slack API レスポンスをそのまま返し、`meta` を付与しないこと。(MUST)

#### Scenario: 既定出力は `response`/`meta` で返る
- Given 有効な profile と token が存在する
- When `api call conversations.list` を実行する
- Then `meta.command` は `api call` である
- And `response` に Slack API レスポンスが入る

#### Scenario: `--raw` 指定時はエンベロープを省略する
- Given 有効な profile と token が存在する
- When `api call conversations.list --raw` を実行する
- Then 出力は Slack API レスポンスの JSON そのままである
- And `meta` フィールドは含まれない
