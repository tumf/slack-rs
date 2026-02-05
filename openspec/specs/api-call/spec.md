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
Output JSON MUST include `meta.profile_name`, `meta.team_id`, `meta.user_id`, and `meta.method`. (MUST)
#### Scenario: Execution context is included in JSON
- Given a profile is selected
- When executing `api call`
- Then meta contains required fields

### Requirement: Return Slack API response as-is
Slack API response MUST be preserved in `response`, even when `ok=false`. (MUST)
#### Scenario: Return `ok=false` response
- Given Slack API returns `ok=false`
- When executing `api call`
- Then `response.ok` is returned as `false`

### Requirement: Prioritize User Token when private_channel is specified
When `types` includes `private_channel` in `api call conversations.list`, User Token MUST be prioritized if `--token-type` is not specified. (MUST)
#### Scenario: Prioritize user token when private_channel is specified
- Given executing `api call conversations.list` with `types=private_channel`
- And `--token-type` is not specified
- And User Token exists
- When calling the API
- Then User Token is used

