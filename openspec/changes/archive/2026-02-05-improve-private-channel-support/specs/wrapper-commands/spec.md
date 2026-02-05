# wrapper-commands Delta Specification (private_channel guidance)

## ADDED Requirements
### Requirement: Display guidance when private_channel retrieval fails
When `types=private_channel` is specified in `conv list`, guidance MUST be displayed if the result is empty and Bot Token is being used. (MUST)
#### Scenario: Empty private channels with Bot Token
- Given executing `conv list --types private_channel` using Bot Token
- When the result is empty
- Then guidance about using User Token or inviting the bot is displayed

### Requirement: Explicit error when User Token does not exist
When `private_channel` is requested without User Token available, an error MUST be returned indicating User Token is required. (MUST)
#### Scenario: User Token unavailable
- Given User Token is not stored
- When executing `conv list --types private_channel`
- Then an error indicating User Token is required is returned
