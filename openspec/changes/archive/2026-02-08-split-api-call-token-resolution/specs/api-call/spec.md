## ADDED Requirements

### Requirement: Token resolution conventions during API calls are maintained after internal responsibility separation
After separating token resolution logic within `run_api_call`, the token resolution priority order (CLI specification > profile default > inference), `SLACK_TOKEN` priority, strict errors on explicit specification, and fallback behavior when necessary MUST be maintained. (MUST)

#### Scenario: `SLACK_TOKEN` priority is maintained after separation
- Given a default token type is configured in the profile and tokens exist in the token store
- And the `SLACK_TOKEN` environment variable is set
- When executing `api call`
- Then the token used for authentication is `SLACK_TOKEN`
- And existing meta information and response format are maintained
