# profiles-and-token-store Change Specification (Default Token Type)

## ADDED Requirements
### Requirement: Store default token type in profile
Profile MUST optionally hold `default_token_type` and persist it. (MUST)
#### Scenario: Save and reload default type
- Given setting `default_token_type=user`
- When saving and reloading profile
- Then `default_token_type` is retained
