# oauth-login Delta Specification

## ADDED Requirements

### Requirement: Do not use environment variables for OAuth configuration resolution

OAuth configuration MUST be resolved from CLI arguments or profile configuration files, and MUST NOT reference environment variables.

#### Scenario: Environment variables are ignored even when set
- Environment variables such as `SLACKRS_CLIENT_ID` are set
- `client_id` exists in profile configuration
- When `slack-rs login` is executed, environment variables are not referenced and profile configuration is used
