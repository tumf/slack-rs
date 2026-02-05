## ADDED Requirements

### Requirement: Write operation help text indicates environment variable
Write operation usage/help text MUST clearly indicate control via `SLACKCLI_ALLOW_WRITE` and MUST NOT mislead users into thinking `--allow-write` is required. (MUST)

#### Scenario: Help display shows write control mechanism
- Given displaying `slack-rs` help/usage
- When checking description of write operations like `msg`/`react`
- Then control via `SLACKCLI_ALLOW_WRITE` is indicated
- And no required notation like `requires --allow-write` is included
