---
change_type: implementation
priority: medium
dependencies: []
references: ["src/cli/mod.rs", "src/commands/conv/api.rs", "src/commands/users_cache.rs", "mention-resolution"]
---

**Change Type**: implementation

# Add `users` field to conv history and thread get output

## Problem / Context

`conv history` and `thread get` currently return raw Slack API message payloads. Each message contains user IDs (in `user` field and `<@Uxxxx>` mentions in text/blocks), but no resolved user information. Users must run separate commands or maintain their own mapping to understand who is involved in a conversation. The existing `users_cache` module already provides name resolution, but it is not applied to history/thread output.

## Proposed Solution

After fetching messages via `conv_history` / `thread_get`, post-process each message to:
1. Collect all user IDs: the `user` field (poster) + any `<@Uxxxx>` mentions found in `text` or `blocks`.
2. Deduplicate and sort the IDs.
3. For each ID, look up `name`, `display_name`, and `real_name` from the workspace users cache.
4. Attach a `users` array to the message object before returning/wrapping the response.

The `users` array will have the shape:
```json
"users": [
  { "id": "U12345678", "name": "tanaka", "display_name": "田中", "real_name": "田中太郎" },
  ...
]
```

This change affects only the CLI output path (`run_conv_history`, `run_thread_get` in `src/cli/mod.rs`). The underlying API client and cache remain unchanged.

## Acceptance Criteria

- Every message returned by `slack-rs conv history <channel>` contains a `users` array with all involved user IDs and their names.
- Every message returned by `slack-rs thread get <channel> <ts>` contains the same `users` array.
- The `users` array includes the poster (`user` field) even when no explicit mentions exist.
- Names are populated from the existing users cache; if a user is not cached, the entry still appears with only the `id` (graceful degradation).
- Raw output (`--raw`) and wrapped output both include the enriched messages.
- No breaking change to existing fields or response shape (additive only).

## Explicit Completion Conditions

- `cargo test` passes (existing tests + any new unit tests for the enrichment logic).
- Manual run of `slack-rs conv history Cxxxx --limit=3` shows `users` array on messages.
- Manual run of `slack-rs thread get Cxxxx Tsxxxx` shows `users` array on messages.
- `cargo clippy -- -D warnings` passes.
- `cargo fmt -- --check` passes.

## Out of Scope

- Automatically refreshing the users cache during history calls.
- Adding a `--with-users` flag (always on).
- Changing the cache storage format or `users_cache` module itself.
- Supporting other commands (e.g., search, react) in this change.
