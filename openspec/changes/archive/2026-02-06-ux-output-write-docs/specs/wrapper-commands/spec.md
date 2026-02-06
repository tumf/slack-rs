# wrapper-commands Specification (Delta)

## MODIFIED Requirements

### Requirement: Wrapper commands output is normalized
ラッパーコマンドの JSON 出力は `response`/`meta` のエンベロープで返し、`meta.profile_name`, `meta.team_id`, `meta.user_id`, `meta.method`, `meta.command` を含むこと。`--raw` が指定された場合は Slack API レスポンスをそのまま返すこと。(MUST)
`SLACKRS_OUTPUT=raw` が設定されている場合、`--raw` が指定されていなくても raw 出力を既定とすること。(MUST)

#### Scenario: `SLACKRS_OUTPUT=raw` で既定出力が raw になる
- Given `SLACKRS_OUTPUT=raw` が設定されている
- And `--raw` が指定されていない
- When `conv list` を実行する
- Then 出力は Slack API レスポンスの JSON そのままである
- And `meta` フィールドは含まれない

### Requirement: Destructive operations require confirmation without --yes flag
write 操作（`msg post/update/delete`, `react add/remove`, `file upload`）はデフォルトで確認を求めなければならない。(MUST)
`--yes` が指定されている場合は確認を省略しなければならない。(MUST)
非対話モードでは `--yes` が無い場合に即時エラーにしなければならない。(MUST)

#### Scenario: `msg post` は確認を求める
- Given `SLACKCLI_ALLOW_WRITE` が許可されている
- And TTY で実行している
- When `msg post` を `--yes` なしで実行する
- Then 実行前に確認プロンプトが表示される

#### Scenario: 非対話モードで `--yes` が無い場合は失敗する
- Given `--non-interactive` が指定されている
- When `file upload` を `--yes` なしで実行する
- Then 確認プロンプトを出さずにエラーになる
