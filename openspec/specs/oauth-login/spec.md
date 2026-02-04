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
OAuth flow MUST NOT start if `SLACKRS_CLIENT_ID` or `SLACKRS_CLIENT_SECRET` is not set. (MUST NOT)
#### Scenario: Required environment variables are missing
- Given `SLACKRS_CLIENT_ID` is not set
- When executing `auth login`
- Then it exits with an error

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
