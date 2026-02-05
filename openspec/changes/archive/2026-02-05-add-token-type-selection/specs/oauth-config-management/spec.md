# oauth-config-management Change Specification (Default Token Type Configuration)

## ADDED Requirements
### Requirement: Provide command to configure default token type
The `config` subcommand MUST allow setting the profile's `default_token_type`. (MUST)
#### Scenario: Execute `config set default --token-type user`
- Given target profile exists
- When executing `config set default --token-type user`
- Then `default_token_type=user` is saved to profile
