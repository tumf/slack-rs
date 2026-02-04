# profiles-and-token-store

## MODIFIED Requirements

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
