# wrapper-commands Change Proposal

## ADDED Requirements
### Requirement: conv command behavior is preserved after internal module split
`conv list`/`conv select`/`conv history` MUST maintain existing argument, output, and error behavior even after internal restructuring. (MUST)

#### Scenario: Behavior is preserved after module split
- Given using existing `conv` command arguments
- When executing `conv list` or `conv history`
- Then the same API calls and output format as before are maintained
