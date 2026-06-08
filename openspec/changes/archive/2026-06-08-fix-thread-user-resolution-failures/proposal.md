---
change_type: implementation
priority: high
dependencies: []
references:
  - openspec/specs/thread-command/spec.md
  - src/cli/mod.rs
  - src/commands/conv/format.rs
  - src/api/client.rs
  - tests/thread_integration_tests.rs
---

# Fix Thread User Resolution Failures

**Change Type**: implementation

## Premise / Context

- The latest merge `fix-thread-resolved-users` moved `thread get` user metadata from per-message `users` arrays to response-level `response.data.resolved_users`.
- Existing canonical `thread-command` spec requires unresolved IDs to avoid fake hydrated profiles and be distinguishable, optionally through `unresolved_user_ids`.
- Code review found that `resolve_thread_users` currently propagates `users.info` errors, while `ApiClient::call_method` converts Slack `ok:false` into `Err(ApiError::SlackError)`.
- As a result, a cache miss plus `users.info` failure can make default `thread get` fail instead of returning thread messages with unresolved IDs recorded.
- Existing tests cover direct `thread_get` and helper behavior, but do not verify the actual CLI wrapper path that adds default envelope metadata and preserves raw output.

## Inferred Request

- Make `thread get` tolerate user-resolution failures and continue returning the thread response whenever `conversations.replies` itself succeeded.
- Ensure unresolved user IDs are reported separately without fabricating ID-only hydrated profiles.
- Add verification that exercises the real CLI output path for default and `--raw` behavior.

## Problem / Context

`thread get` should treat response-level user resolution as wrapper-owned enrichment. The primary Slack operation is `conversations.replies`; if that succeeds, thread retrieval should not fail merely because a referenced user cannot be resolved via cache or `users.info`.

The current implementation path can still fail after successful thread retrieval because `users.info` errors are returned from `resolve_thread_users`. This contradicts the intended graceful-degradation behavior and creates a regression risk for workspaces without `users:read`, deleted users, bot users with restricted visibility, or transient user lookup failures.

## Proposed Solution

- Update thread-scoped user resolution so cache misses that cannot be hydrated are appended to `unresolved_user_ids` instead of failing the entire command.
- Keep hard failures limited to the primary `conversations.replies` retrieval path and serialization/configuration errors that prevent valid output construction.
- Preserve the existing invariant that `response.data.messages` remains Slack-native and does not receive CLI-added `users` arrays.
- Preserve `--raw` behavior so raw output does not perform wrapper user resolution and does not include `resolved_users` or `unresolved_user_ids`.
- Add CLI-path tests or equivalent integration coverage that would fail if metadata is added in the wrong place, raw output is enriched, or user lookup failure aborts the command.

## Acceptance Criteria

- When `conversations.replies` succeeds but one or more referenced user IDs cannot be resolved from cache or `users.info`, default `thread get` still exits successfully and returns the thread messages.
- Default `thread get` output includes hydrated profiles only in `response.data.resolved_users`, keyed by user ID, for users that were actually resolved.
- Default `thread get` output includes unresolved IDs separately in `response.data.unresolved_user_ids` when user resolution fails or returns no usable user object.
- Default `thread get` output never places ID-only fake hydrated profiles in `response.data.resolved_users`.
- Default `thread get` output does not add CLI-owned `users` arrays to individual Slack message objects.
- `thread get --raw` returns the Slack-native aggregated response and does not include wrapper-owned `resolved_users` or `unresolved_user_ids`.
- Tests exercise the CLI output/envelope path, not only the lower-level `thread_get` helper.

## Explicit Completion Conditions

- `src/commands/conv/format.rs` or the equivalent resolution layer records user lookup failures as unresolved IDs without returning an error for Slack `users.info` failures.
- `src/cli/mod.rs` continues to add user resolution metadata only on the default non-raw output path.
- `tests/thread_integration_tests.rs` or a dedicated CLI integration test verifies default output with a mocked `users.info` failure and asserts successful output plus `unresolved_user_ids`.
- A CLI-path test verifies `--raw` output excludes wrapper-owned user resolution metadata.
- `cargo test --test thread_integration_tests` passes.
- `cargo test` or the repository-standard targeted replacement passes unless an unrelated heavy/default-suite constraint is documented.

## Out of Scope

- Changing the shape or fields of hydrated user profile entries beyond the existing `resolved_users` contract.
- Adding persistent cache updates as a side effect of `thread get`.
- Changing `ApiClient::call_method` error semantics globally for all wrapper commands.
- Introducing batching, concurrency, or rate-limit backoff for user lookups beyond what is needed for graceful degradation.
