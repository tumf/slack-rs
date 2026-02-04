# oauth-login Specification

## Purpose
Defines the OAuth 2.0 PKCE authentication flow for slack-rs, enabling secure login to Slack workspaces without exposing client secrets.
## Requirements
### Requirement: Generate authentication URL with PKCE and state
The OAuth authorization URL MUST include `client_id`, `redirect_uri`, `state`, `code_challenge`, and `code_challenge_method=S256`. (MUST)
#### Scenario: Authentication URL contains required parameters
- Given OAuth configuration is loaded
- When generating the authentication URL
- Then all required parameters are included

### Requirement: Do not start if required configuration is missing

When OAuth client information is missing at login start, it MUST be supplemented through interactive input.

#### Scenario: Required configuration is missing
- When `--client-id` is not specified and `client_id` is not in the profile, supplement through interactive input
- `client_secret` is always obtained through interactive input

### Requirement: Validate state in localhost callback
The authorization code MUST NOT be accepted if the callback `state` does not match. (MUST NOT)
#### Scenario: State does not match
- Given callback server is running
- When `code` is sent with mismatched `state`
- Then an error occurs

### Requirement: Callback reception has a timeout
The callback MUST be received within a certain time period. (MUST)
#### Scenario: Code is not received before timeout
- Given callback server is running
- When code does not arrive within the specified time
- Then a timeout error occurs

### Requirement: Exchange authorization code for token and save
`access_token` and profile metadata from `oauth.v2.access` success response MUST be saved. (MUST)
#### Scenario: Save `oauth.v2.access` success response
- Given a valid code exists
- When executing token exchange
- Then access_token and profile metadata are saved

### Requirement: Same `(team_id, user_id)` is treated as update
When the same `(team_id, user_id)` exists, existing token/metadata MUST be updated instead of adding a new profile. (MUST)
#### Scenario: Re-login with existing account
- Given a profile with the same `(team_id, user_id)` exists
- When executing `auth login`
- Then the existing profile is updated

### Requirement: Auth commands can manipulate profiles
`auth status/list/rename/logout` MUST be able to read, update, and delete profiles. (MUST)
#### Scenario: auth list returns profiles.json content
- Given multiple profiles are saved
- When executing `auth list`
- Then a list of profiles is returned

### Requirement: Do not use environment variables for OAuth configuration resolution

OAuth configuration MUST be resolved from CLI arguments or profile configuration files, and MUST NOT reference environment variables.

#### Scenario: Environment variables are ignored even when set
- Environment variables such as `SLACKRS_CLIENT_ID` are set
- `client_id` exists in profile configuration
- When `slack-rs login` is executed, environment variables are not referenced and profile configuration is used

