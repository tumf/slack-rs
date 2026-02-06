# Proposal: Improve conversation discovery helpers

## Why
`conv list` and `conv search` are used daily, but the handling of private channels and search matching precision often diverge from user expectations. Users need easier access to private channels and more intuitive search behavior. Additionally, `channel_not_found` errors require better guidance to help users diagnose the root cause.

## What Changes
- Add `--include-private` and `--all` flags to `conv list` for simplified type resolution
- Improve `conv search` default matching behavior to use case-insensitive substring matching
- Add guidance output to stderr when `channel_not_found` error is returned

## Out of Scope
- Changes to output format or profile resolution
- Modifications to Slack API specifications themselves

## Success Criteria
- `conv list --include-private` / `--all` resolves types as expected
- `conv search` defaults to case-insensitive substring matching
- `channel_not_found` error displays cause candidates and resolution steps to stderr
