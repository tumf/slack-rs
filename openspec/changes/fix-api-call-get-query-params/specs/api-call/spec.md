# api-call Specification (Delta)

## MODIFIED Requirements

### Requirement: GET 時は key=value を query params で送信する
`--get` が指定された場合、`key=value` は URL の query params として送信されなければならない。GET 時はリクエストボディを送信してはならない。`--json` が併用されていても GET では query を優先し、JSON body は送らない。(MUST)

#### Scenario: `--get` で conversations.replies に必須パラメータを渡す
- Given `--get` が指定されている
- And `channel=C123` と `ts=12345.6789` が `key=value` で指定されている
- When `api call conversations.replies` を実行する
- Then GET リクエストの query に `channel=C123` と `ts=12345.6789` が含まれる
- And リクエストボディは送信されない
