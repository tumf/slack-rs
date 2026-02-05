# wrapper-commands Change Specification (Token Type Selection)

## ADDED Requirements
### Requirement: Wrapper commands accept `--token-type`
Wrapper commands MUST accept `--token-type user|bot` and use the specified token. (MUST)
#### Scenario: Explicitly specify bot in conv list
- Given executing `conv list --token-type bot`
- When calling API
- Then Bot Token is used in Authorization header

### Requirement: Use default token type
When `--token-type` is not specified, MUST follow profile's `default_token_type`. (MUST)
#### Scenario: Default type is bot
- Given `default_token_type` is `bot`
- When executing `msg post`
- Then Bot Token is used
