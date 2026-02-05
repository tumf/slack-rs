# oauth-login Delta Specification (auth status enhancement)

## ADDED Requirements
### Requirement: auth status displays token information
`auth status` MUST display available token types, their scopes, and the default token type. (MUST)
#### Scenario: Both User and Bot tokens exist
- Given User Token and Bot Token are saved
- When executing `auth status`
- Then both token types and their scopes are displayed

### Requirement: Display Bot ID when Bot Token exists
When a Bot Token is saved, `auth status` MUST display the Bot ID. (MUST)
#### Scenario: Bot ID is displayed for Bot Token
- Given Bot Token and Bot ID are saved
- When executing `auth status`
- Then Bot ID is displayed
