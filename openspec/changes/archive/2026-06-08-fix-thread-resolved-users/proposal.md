---
change_type: implementation
priority: high
dependencies: []
references:
  - openspec/specs/thread-command/spec.md
  - openspec/specs/wrapper-commands/spec.md
  - openspec/specs/mention-resolution/spec.md
  - src/cli/mod.rs
  - src/commands/conv/format.rs
  - tests/thread_integration_tests.rs
---

# Fix misleading user hydration in `thread get`

**Change Type**: implementation

## Problem / Context

`slack-rs thread get` currently mutates each returned Slack message by injecting a `users` array assembled from locally collected user IDs. The implementation lives in `src/cli/mod.rs` and calls `enrich_message_with_users`, which currently produces cache-backed profile fields when available and `{ "id": "U..." }` entries when the workspace cache is absent or stale.

This output is misleading for CLI consumers because the injected `users` field looks like hydrated profile data even when it only contains IDs. It also omits user IDs referenced by `reactions[].users[]`, which means consumers cannot reliably build human-readable summaries from the wrapper output.

The current behavior is not described in the canonical `thread-command` spec, so the proposal must define a stable response contract for wrapper consumers without changing raw Slack payload semantics.

## Proposed Solution

Stop injecting CLI-owned `users` arrays into message objects returned by `thread get`. Instead, keep `response.data.messages` structurally aligned with the Slack payload and add a response-level `resolved_users` map in the default wrapper output path.

The `resolved_users` map will be built from user IDs collected across all thread messages, including:

- message authors from `message.user`
- mentions embedded in `text`
- mentions embedded anywhere within `blocks`
- reaction actors from `reactions[].users[]`

Resolution will prefer the existing workspace cache. If an ID is not present in cache, the wrapper should attempt to hydrate it via `users.info`. Only successfully resolved users should appear in `resolved_users`; unresolved IDs should be surfaced separately so consumers do not confuse them with hydrated profiles.

`--raw` output must remain the unmodified Slack API response and must not include `resolved_users` or other wrapper-owned enrichment fields.

## Acceptance Criteria

1. Default `slack-rs thread get` output includes `response.data.resolved_users` as a map keyed by Slack user ID with resolved profile fields sufficient for human-readable summaries.
2. `response.data.messages` does not gain a CLI-injected `users` field in default output.
3. User IDs referenced by message authors, text mentions, block mentions, and `reactions[].users[]` are deduplicated before resolution.
4. Cache hits populate `resolved_users` without requiring `users.info`; cache misses are eligible for `users.info` fallback.
5. IDs that cannot be resolved are not represented as misleading ID-only profile objects inside `resolved_users`; if surfaced, they appear in a distinct field such as `unresolved_user_ids`.
6. `slack-rs thread get --raw` continues to return the raw Slack response shape without wrapper-owned `resolved_users` metadata.
7. Help/introspection and OpenSpec canonical requirements describe the wrapper-owned resolution contract clearly enough that consumers do not mistake raw Slack fields for hydrated profiles.

## Explicit Completion Conditions

- `src/cli/mod.rs` no longer mutates each message with injected `users` arrays in the default `thread get` path.
- The repository contains an implementation path that assembles a response-level `resolved_users` map for `thread get`, including reaction user collection.
- Repository tests cover at least one case where a thread message contains mentions and reactions and the resulting wrapper output contains resolved users at response scope rather than message scope.
- Repository tests cover raw-output preservation so a stubbed or no-op implementation would fail verification.
- Canonical OpenSpec deltas define the new wrapper contract for `thread get` and the normalized wrapper response.

## Out of Scope

- Changing raw Slack API payloads returned by `--raw`
- Resolving non-user actors such as `bot_id` into bot profile records
- Backfilling identical response-level hydration for every wrapper command in this same change
