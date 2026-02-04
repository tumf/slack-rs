# profile-oauth-credentials Delta Specification

## MODIFIED Requirements

### Requirement: OAuth credentials retrieval at login prioritizes interactive input

Client information at login MUST prioritize saved profile configuration and Keyring, prompting for interactive input only for missing items.

#### Scenario: Prompts are skipped when saved configuration exists
- `client_id`/`redirect_uri`/`scopes` are saved in `profiles.json`
- `client_secret` is saved in Keyring
- When `slack-rs login` is executed, each value is resolved from saved configuration without interactive input
- Input is prompted only for items that are missing
