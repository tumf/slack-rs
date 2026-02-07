## MODIFIED Requirements

### Requirement: Tokens are saved in file-based storage and not in configuration file

Tokens (bot/user) and OAuth `client_secret` MUST be saved in FileTokenStore, not in `profiles.json`. (MUST)

The default storage location for FileTokenStore MUST be `~/.local/share/slack-rs/tokens.json`. (MUST)

`profiles.json` and the credential file `tokens.json` MUST NOT be saved in the same file. (MUST NOT)

#### Scenario: Credentials are saved separately from configuration file
- Given `profiles.json` exists in the configuration directory
- When saving bot token and OAuth `client_secret`
- Then credentials are saved in `~/.local/share/slack-rs/tokens.json`
- And credentials are not saved in `profiles.json`
