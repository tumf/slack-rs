## MODIFIED Requirements

### Requirement: CLI behavior is preserved after internal refactoring of top-level command dispatch
After internally splitting and extracting the top-level command dispatch in `main`, the behavior of existing commands regarding argument parsing, stdout/stderr output, and exit codes MUST remain backward compatible. (MUST)

#### Scenario: Existing command behavior is unchanged after main split
- Given executing existing command inputs (`api call`, `auth login/status`, `conv list/history`, `msg post`, `file download`, `doctor`)
- When running with an implementation where `main` dispatch is extracted into handler functions
- Then success/failure determination for each command follows the original behavior
- And exit code conventions including exit code 2 for non-interactive errors are maintained
- And existing JSON envelope and help output compatibility is preserved
