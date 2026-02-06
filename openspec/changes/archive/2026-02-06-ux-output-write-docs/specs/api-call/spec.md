# api-call Specification (Delta)

## MODIFIED Requirements

### Requirement: Include meta in output
出力 JSON は `meta.profile_name`, `meta.team_id`, `meta.user_id`, `meta.method` に加えて `meta.command` を含むこと。`--raw` が指定されていない場合は `response`/`meta` のエンベロープで出力すること。`--raw` が指定された場合は Slack API レスポンスをそのまま返し、`meta` を付与しないこと。(MUST)
`SLACKRS_OUTPUT=raw` が設定されている場合、`--raw` が指定されていなくても raw 出力を既定とすること。(MUST)
`SLACKRS_OUTPUT=envelope` は既存の既定動作を維持すること。(MUST)

#### Scenario: `SLACKRS_OUTPUT=raw` で既定出力が raw になる
- Given `SLACKRS_OUTPUT=raw` が設定されている
- And `--raw` が指定されていない
- When `api call conversations.list` を実行する
- Then 出力は Slack API レスポンスの JSON そのままである
- And `meta` フィールドは含まれない
