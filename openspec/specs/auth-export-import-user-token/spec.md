# auth-export-import-user-token Specification

## Purpose
Ensure that `auth export` and `auth import` commands handle both bot tokens and user tokens correctly. This specification addresses the issue where user tokens stored under the `team_id:user_id:user` key were not being exported and therefore could not be restored during import operations.

## Requirements
### Requirement: Export/import must handle both bot and user tokens
`auth export` MUST include user tokens in the encrypted payload in addition to bot tokens. (MUST)
`auth import` MUST restore user tokens to their dedicated key when they are present in the import file. (MUST)

#### Scenario: Export includes user token
- **GIVEN** a profile has both bot token and user token stored
- **WHEN** executing `slack-rs auth export --profile <name> --out <path>`
- **THEN** the encrypted payload includes the user token

#### Scenario: Import restores user token
- **GIVEN** an export file containing a user token
- **WHEN** executing `slack-rs auth import --in <path> --yes`
- **THEN** the user token is stored under the `team_id:user_id:user` key

