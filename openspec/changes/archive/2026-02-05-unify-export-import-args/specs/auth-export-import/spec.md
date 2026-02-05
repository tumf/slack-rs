## ADDED Requirements
### Requirement: Unified argument parsing preserves existing behavior
`auth export` and `auth import` MUST maintain existing flag, confirmation, and error behavior even when using a shared argument parser. (MUST)

#### Scenario: Shared parser maintains compatibility
- **GIVEN** existing `auth export`/`auth import` argument sets are used
- **WHEN** each command is executed
- **THEN** the same confirmation flows and error conditions apply as before
