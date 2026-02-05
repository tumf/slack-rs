# Proposal: Modularize conv Command Implementation

## Why
`src/commands/conv.rs` has grown large, mixing filtering, sorting, formatting, and API calls. Maintainability has decreased.

## What Changes
- Split `conv`-related functions by responsibility (e.g., filter/sort/format/api)
- Maintain existing public API through re-exports
- Preserve CLI behavior without changing specifications or output

## Scope
- Split functions in `conv` module by responsibility
- Maintain existing public API paths via re-exports

## Non-Goals
- Changing `conv` command specifications
- Modifying output or filter conditions

## Impact & Risks
- Potential regression due to visibility or module path changes in public functions

## Acceptance Criteria
- `conv list`/`conv select`/`conv history` maintain existing behavior
- Calls from `commands::` work with same paths

## Dependencies
- None
