# api-call Delta Specification (private_channel priority)

## ADDED Requirements
### Requirement: Prioritize User Token when private_channel is specified
When `types` includes `private_channel` in `api call conversations.list`, User Token MUST be prioritized if `--token-type` is not specified. (MUST)
#### Scenario: Prioritize user token when private_channel is specified
- Given executing `api call conversations.list` with `types=private_channel`
- And `--token-type` is not specified
- And User Token exists
- When calling the API
- Then User Token is used
