# oauth-config-management Specification

## Purpose
TBD - created by archiving change add-oauth-config-management. Update Purpose after archive.
## Requirements
### Requirement: OAuth configuration can be managed via CLI

OAuth configuration per profile MUST be settable, viewable, and deletable via CLI.

#### Scenario: set/show/delete commands are provided
- `slackrs config oauth set --profile <name>` can save configuration
- `slackrs config oauth show --profile <name>` can view saved content
- `slackrs config oauth delete --profile <name>` can delete configuration

### Requirement: client_secret is stored in Keyring and not displayed

`client_secret` MUST NOT be saved to configuration files but stored in Keyring, and MUST NOT be output by the `show` command.

#### Scenario: Secret is not output
- Input `client_secret` when executing `config oauth set`
- `profiles.json` does not contain `client_secret`
- `config oauth show` output does not contain `client_secret`

