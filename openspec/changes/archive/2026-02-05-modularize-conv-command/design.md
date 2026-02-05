# Design: Modularize conv Command Implementation

## Approach
- Preserve existing public API and CLI behavior
- Split modules by responsibility and re-export via `mod.rs`

## Changes Overview
- Split under `src/commands/conv/`
  - `filter.rs`: filtering/matching
  - `sort.rs`: sorting logic
  - `format.rs`: output formatting
  - `api.rs`: API calls
  - `select.rs`: interactive selection
- Re-export existing function names via `src/commands/conv/mod.rs`

## Alternatives Considered
- Reorganize function order within single file
  - Weak separation of concerns, limited improvement in changeability

## Affected Areas
- `src/commands/conv.rs` (becomes `src/commands/conv/mod.rs`)
- `src/commands/mod.rs`
- `src/cli/mod.rs`

## Compatibility
- Existing function names and CLI behavior remain unchanged
