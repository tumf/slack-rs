# Implementation Tasks for add-conv-history-users-field

## Implementation Tasks

- [ ] Task 1: Create a helper function `enrich_message_with_users` that extracts the `user` field and all `<@Uxxxx>` mentions from `text`/`blocks`, deduplicates, sorts, and builds a `users` array using the existing `WorkspaceCache`. Place the helper in `src/commands/conv/format.rs` (or a new small module). Verification: `unit` - add a unit test that passes a sample message Value and asserts the `users` array is correctly populated (cargo test).

- [ ] Task 2: Modify `run_conv_history` in `src/cli/mod.rs` to call the enrichment helper on every message before wrapping. Verification: `integration` - run `cargo test --test cli_conv_history_users` (or equivalent) exercising the full history path with a mocked cache.

- [ ] Task 3: Modify `run_thread_get` in `src/cli/mod.rs` with the same enrichment step. Verification: `integration` - run `cargo test --test cli_thread_get_users` exercising the thread path.

- [ ] Task 4: Ensure the enrichment also works when `SLACKRS_OUTPUT=raw`. Verification: `manual` - execute `SLACKRS_OUTPUT=raw slack-rs conv history Cxxxx --limit=1` and confirm `users` appears (intentional manual coverage for the exact CLI flag path).

- [ ] Task 5: Run full lint and format checks. Verification: `manual` - `cargo clippy -- -D warnings && cargo fmt -- --check` (required per AGENTS.md).

## Future Work

- Expose a `--refresh-users-cache` flag on history/thread commands (would require network call during CLI execution).
- Add a dedicated `conv history --format=mentions` that only outputs the users array (separate proposal).
