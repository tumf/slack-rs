# api-call Specification

## Purpose
Provides a generic API call interface to invoke any Slack Web API method with proper authentication, retry logic, and flexible parameter handling.
## Requirements
### Requirement: Can call any Slack API method
Any Slack Web API method MUST be callable via `https://slack.com/api/{method}`. (MUST)
#### Scenario: Execute `api call search.messages`
- Given a valid profile and token exist
- When executing `api call search.messages`
- Then a request is sent to `https://slack.com/api/search.messages`

### Requirement: Add authentication header
Requests MUST include `Authorization: Bearer <token>`. (MUST)
#### Scenario: Add Bearer token
- Given a token can be retrieved
- When executing `api call`
- Then the Authorization header is included

### Requirement: Can switch between form and JSON for sending
`key=value` MUST be sent as form-urlencoded, `--json` MUST be sent as JSON body. (MUST)
#### Scenario: Send JSON body when `--json` is specified
- Given `--json` is specified
- When executing `api call chat.postMessage`
- Then Content-Type is `application/json` and sent

### Requirement: Can switch HTTP method
Default MUST be POST, and GET MUST be used when `--get` is specified. (MUST)
#### Scenario: Specify `--get`
- Given `--get` is specified
- When executing `api call users.info`
- Then a GET request is sent

### Requirement: Retry 429 following Retry-After
On 429 response, MUST wait according to `Retry-After` and retry up to a maximum number of times. (MUST)
#### Scenario: Wait and retry after 429 response
- Given returning 429 and Retry-After
- When executing `api call`
- Then wait for the specified seconds and retry

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

### Requirement: Return Slack API response as-is
Slack API response MUST be preserved in `response`, even when `ok=false`. (MUST)
#### Scenario: Return `ok=false` response
- Given Slack API returns `ok=false`
- When executing `api call`
- Then `response.ok` is returned as `false`

### Requirement: Can explicitly specify token type with `--token-type`
`api call` MUST accept `--token-type user|bot` and use the specified token type. (MUST)
#### Scenario: Explicitly specify user token
- Given `--token-type user` is specified
- When executing `api call conversations.list`
- Then User Token is used in Authorization header

### Requirement: Resolve with default token type
`--token-type` が未指定の場合、プロフィールの `default_token_type` が設定されていればその値を使用しなければならない。(MUST)
`default_token_type` が未設定の場合は、トークンストアに User token が存在するなら User を既定として選択し、存在しない場合は Bot を既定として選択しなければならない。(MUST)

#### Scenario: default_token_type 未設定で User token が存在する
- Given `default_token_type` が未設定である
- And トークンストアに User token が保存されている
- When `api call` を実行する
- Then User Token が使用される

### Requirement: Error when token does not exist
When the specified token type does not exist, MUST fail with a clear error. (MUST)
#### Scenario: User token is not saved
- Given `--token-type user` is specified
- And User Token is not saved
- When executing `api call`
- Then an error indicating missing token is returned

### Requirement: Provide guidance for known Slack error codes
`api call` の実行結果が `ok=false` かつ `error` が既知のコード（`not_allowed_token_type`, `missing_scope`, `invalid_auth` など）に一致する場合、標準エラー出力に原因と解決策のガイダンスを表示すること。JSON 出力の `response` は変更せず、追加情報は標準エラー出力に限定すること。(MUST)

#### Scenario: `not_allowed_token_type` のガイダンスが表示される
- Given Slack API が `ok=false` と `error=not_allowed_token_type` を返す
- When `api call search.messages` を実行する
- Then 標準エラー出力に原因と解決策が表示される
- And JSON 出力の `response` は Slack のレスポンスのままである

### Requirement: Token resolution conventions during API calls are maintained after internal responsibility separation
After separating token resolution logic within `run_api_call`, the token resolution priority order (CLI specification > profile default > inference), `SLACK_TOKEN` priority, strict errors on explicit specification, and fallback behavior when necessary MUST be maintained. (MUST)

#### Scenario: `SLACK_TOKEN` priority is maintained after separation
- Given a default token type is configured in the profile and tokens exist in the token store
- And the `SLACK_TOKEN` environment variable is set
- When executing `api call`
- Then the token used for authentication is `SLACK_TOKEN`
- And existing meta information and response format are maintained

### Requirement: Send key=value as query params for GET requests
When `--get` is specified, `key=value` parameters MUST be sent as URL query parameters. GET requests MUST NOT send a request body. Even if `--json` is also specified, GET requests prioritize query parameters and do not send a JSON body. (MUST)

#### Scenario: Pass required parameters to conversations.replies with `--get`
- Given `--get` is specified
- And `channel=C123` and `ts=12345.6789` are specified as `key=value` parameters
- When executing `api call conversations.replies`
- Then the GET request query includes `channel=C123` and `ts=12345.6789`
- And no request body is sent

