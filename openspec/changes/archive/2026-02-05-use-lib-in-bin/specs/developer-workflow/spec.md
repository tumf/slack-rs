# developer-workflow

## ADDED Requirements

### Requirement: Binary must use library public API
The CLI binary must be composed using public modules from the `slack_rs` library, without holding duplicate implementation of the same functionality. (MUST)

#### Scenario: No duplicate test execution with `cargo test`
- Given `src/main.rs` references modules via `slack_rs::`
- When running `cargo test --quiet`
- Then the same test suite is not output twice
