# profiles-and-token-store Delta Specification

## MODIFIED Requirements

### Requirement: Profile configuration can be persisted

Non-sensitive information in a Profile MUST be saved to `profiles.json` and retrievable with the same content after restart.
OAuth non-sensitive information (`client_id`, `redirect_uri`, `scopes`) is also subject to persistence.

#### Scenario: Profiles containing OAuth non-sensitive information can be saved and reloaded
- Save a profile containing `client_id`, `redirect_uri`, `scopes`
- Reload `profiles.json`
- All values can be retrieved with the same content
