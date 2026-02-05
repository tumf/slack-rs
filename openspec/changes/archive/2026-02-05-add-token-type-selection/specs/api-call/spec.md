# api-call Change Specification (Token Type Selection)

## MODIFIED Requirements
### Requirement: Include meta in output
Output JSON MUST include `meta.profile_name`, `meta.team_id`, `meta.user_id`, `meta.method`, and `meta.token_type`. (MUST)
#### Scenario: token_type is included in output
- Given executing `api call`
- When calling API with a valid token
- Then `meta.token_type` contains `user` or `bot`

## ADDED Requirements
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
