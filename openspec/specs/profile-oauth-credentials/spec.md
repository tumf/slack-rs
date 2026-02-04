# profile-oauth-credentials Specification

## Purpose
TBD - created by archiving change add-per-profile-oauth-credentials. Update Purpose after archive.
## Requirements
### Requirement: OAuth credentials retrieval at login prioritizes interactive input

Client information at login MUST prioritize saved profile configuration and Keyring, prompting for interactive input only for missing items.

#### Scenario: Prompts are skipped when saved configuration exists
- `client_id`/`redirect_uri`/`scopes` are saved in `profiles.json`
- `client_secret` is saved in Keyring
- When `slack-rs login` is executed, each value is resolved from saved configuration without interactive input
- Input is prompted only for items that are missing

### Requirement: Store `client_id` in profile and `client_secret` in Keyring

Each profile MUST maintain its own OAuth client ID, and secrets MUST NOT remain in configuration files.

#### Scenario: Saved to configuration file and Keyring after successful login
- `client_id` is saved to `profiles.json`
- `client_secret` is saved to Keyring and NOT written to configuration files

### Requirement: Existing profiles can be loaded even without `client_id` set

Previously saved configuration files MUST be loadable as-is.

#### Scenario: Old format `profiles.json` can be loaded without error
- Loading succeeds even if `client_id` is missing

