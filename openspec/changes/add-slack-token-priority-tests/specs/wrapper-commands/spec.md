## MODIFIED Requirements
### Requirement: Use default token type
`--token-type` が指定されていない場合、ラッパーコマンドはプロフィールの `default_token_type` を解決結果として扱わなければならない。(MUST)
`SLACK_TOKEN` が設定されている場合は、トークンソースとしてそれを優先し、token store の内容に関わらず `SLACK_TOKEN` を使用しなければならない。(MUST)

#### Scenario: default_token_type と SLACK_TOKEN の併用
- Given `default_token_type=user` が設定されている
- And `SLACK_TOKEN` が設定されている
- When `msg post` を実行する
- Then リクエストのトークンは `SLACK_TOKEN` が使用される
- And メタ情報上の `token_type` は `user` として扱われる
