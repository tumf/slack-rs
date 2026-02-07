# profiles-and-token-store Specification

## Purpose
Defines how slack-rs persists profile configurations and manages access tokens securely using OS-level storage mechanisms.
## Requirements
### Requirement: Profile configuration can be persisted

Non-sensitive information contained in Profile MUST be saved in `profiles.json` and retrievable with the same content after restart. (MUST)

OAuth non-sensitive information (`client_id`, `redirect_uri`, `bot_scopes`, `user_scopes`) MUST also be subject to persistence. (MUST)

#### Scenario: Profile containing bot/user scopes can be saved and reloaded
- Given saving a profile containing `client_id`, `redirect_uri`, `bot_scopes`, `user_scopes`
- When reloading `profiles.json`
- Then all values can be retrieved with identical content

### Requirement: Configuration file has a version field
`profiles.json` MUST contain a `version` field. (MUST)
#### Scenario: Version is included when saving
- Given a newly created configuration file exists
- When the configuration is saved
- Then `version` is included

### Requirement: profile_name is unique
The same `profile_name` MUST NOT be registered multiple times. (MUST NOT)
#### Scenario: Attempting to add a profile with the same name
- Given `profile_name` already exists
- When saving the same `profile_name`
- Then a duplicate error occurs

### Requirement: `(team_id, user_id)` is unique as a stable key
Profiles with the same `(team_id, user_id)` MUST NOT be duplicated. (MUST NOT)
#### Scenario: Re-registering the same `(team_id, user_id)`
- Given `(team_id, user_id)` already exists
- When saving a profile with the same `(team_id, user_id)`
- Then the existing entry is updated and not added as new

### Requirement: Tokens are saved in file-based storage and not in configuration file

Tokens (bot/user) and OAuth `client_secret` MUST be saved in FileTokenStore, not in `profiles.json`. (MUST)

The default storage location for FileTokenStore MUST be `~/.local/share/slack-rs/tokens.json`. (MUST)

`profiles.json` and the credential file `tokens.json` MUST NOT be saved in the same file. (MUST NOT)

#### Scenario: Credentials are saved separately from configuration file
- Given `profiles.json` exists in the configuration directory
- When saving bot token and OAuth `client_secret`
- Then credentials are saved in `~/.local/share/slack-rs/tokens.json`
- And credentials are not saved in `profiles.json`

### Requirement: file-based token storage key format is stable

The file-based storage key for bot tokens MUST be `{team_id}:{user_id}`. (MUST)

User tokens MUST be saved with a stable separate key different from bot tokens. (MUST)

OAuth client secrets MUST be saved in the key format `oauth-client-secret:{profile_name}`. (MUST)

#### Scenario: bot and user tokens are saved with separate keys
- **WHEN** there are `team_id=T123` and `user_id=U456`
- **THEN** the bot token key is `T123:U456` and the user token is saved with a separate stable key

#### Scenario: OAuth client secret is saved with correct key format
- **WHEN** saving OAuth client secret for profile name `default`
- **THEN** the key is saved as `oauth-client-secret:default`

### Requirement: Stable key can be resolved from profile_name
`(team_id, user_id)` MUST be uniquely resolvable from `profile_name`. (MUST)
#### Scenario: Resolve `(team_id, user_id)` from profile_name
- Given profile_name exists in the configuration
- When profile_name is specified
- Then `(team_id, user_id)` is returned

### Requirement: Profile configuration file uses slack-rs as default path
Profile non-secret information MUST be stored in `profiles.json` under the `slack-rs` configuration directory. (MUST)
#### Scenario: Resolve default path
- Given retrieving the default configuration path
- When referencing the OS configuration directory
- Then the path contains `slack-rs` and `profiles.json`

### Requirement: Legacy path configuration file is migrated to new path
When the new path does not exist and `profiles.json` exists in the legacy path (`slack-cli`), the configuration file MUST be migrated to the new path. (MUST)
#### Scenario: Loading when only legacy path exists
- Given `profiles.json` exists in the legacy path and does not exist in the new path
- When loading the configuration file
- Then `profiles.json` is created in the new path and the same content is loaded

### Requirement: Store default token type in profile
Profile MUST optionally hold `default_token_type` and persist it. (MUST)
#### Scenario: Save and reload default type
- Given setting `default_token_type=user`
- When saving and reloading profile
- Then `default_token_type` is retained

### Requirement: FileTokenStore mode reuses tokens.json path and stable keys

In file mode (`SLACKRS_TOKEN_STORE=file`), the existing `FileTokenStore` storage path `~/.config/slack-rs/tokens.json` MUST be reused. (MUST)

In file mode, the key format MUST also maintain the existing specification. (MUST) At minimum, the following keys must be in the same format:
- bot token: `{team_id}:{user_id}`
- OAuth `client_secret`: `oauth-client-secret:{profile_name}`

#### Scenario: file mode uses tokens.json and existing key format
- Given `SLACKRS_TOKEN_STORE=file` is set
- When saving bot token for `team_id=T123` and `user_id=U456`
- Then it is saved in `~/.config/slack-rs/tokens.json` with key `T123:U456`
- And `client_secret` for `profile_name=default` is saved with key `oauth-client-secret:default`

