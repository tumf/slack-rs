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
When `--token-type` is not specified, MUST use the profile's `default_token_type`. (MUST)
#### Scenario: Default type is user
- Given profile's `default_token_type` is `user`
- When executing `api call`
- Then User Token is used

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

