## ADDED Requirements
### Requirement: client_secret can be obtained securely

`config oauth set` MUST be able to obtain `client_secret` via the following input sources:
1) Environment variable specified with `--client-secret-env <VAR>`
2) `SLACKRS_CLIENT_SECRET`
3) `--client-secret-file <PATH>`
4) `--client-secret <SECRET>` (requires explicit consent via `--yes`)
5) Interactive input (if none of the above are provided)

If `--client-secret` is specified without `--yes`, it MUST be rejected for security reasons and alternative methods MUST be suggested.

#### Scenario: Obtain client_secret from environment variable
- Given `SLACKRS_CLIENT_SECRET` is set
- When `config oauth set <profile> --client-id ... --redirect-uri ... --scopes ...` is executed
- Then `client_secret` is obtained without interactive input
- And `client_secret` is stored in the token store backend

#### Scenario: `--client-secret` requires `--yes`
- Given `--client-secret` is specified
- And `--yes` is not specified
- When `config oauth set` is executed
- Then the command MUST fail
- And alternative methods (environment variable/file/interactive input) MUST be suggested
