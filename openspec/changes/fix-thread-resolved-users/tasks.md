## Implementation Tasks

- [ ] Remove message-scoped `users` injection from the `thread get` default wrapper path and introduce response-scoped hydration assembly for thread results. (verification: integration - add/update `tests/thread_integration_tests.rs` with assertions that `response.data.messages[*].users` is absent and `response.data.resolved_users` is present in default output)
- [ ] Extend thread user ID collection to include `message.user`, mention references in `text`/`blocks`, and `reactions[].users[]`, with deterministic deduplication across the whole thread. (verification: unit - add focused collector tests in `src/commands/conv/format.rs` covering reaction actors and duplicate IDs from author/mention/reaction sources)
- [ ] Resolve collected IDs using workspace cache first and `users.info` fallback for cache misses, returning only hydrated profiles in `resolved_users` and surfacing misses separately when needed. (verification: integration - add mock-backed tests in `tests/thread_integration_tests.rs` or a new adjacent integration test file that exercises a cache miss followed by mocked `users.info` fallback and fails if the resolver returns only ID placeholders)
- [ ] Keep `--raw` behavior unchanged and document the default wrapper contract in command help/introspection or adjacent output schema surfaces. (verification: integration - extend `tests/thread_integration_tests.rs` or adjacent CLI output tests to assert raw output remains Slack-native without `resolved_users`; verification: manual - inspect help/introspection text that documents response-level `resolved_users` semantics)
- [ ] Update OpenSpec deltas and repository tests together so the canonical contract and runnable verification agree on response-level user resolution. (verification: manual - run `cflx openspec validate fix-thread-resolved-users --strict --evidence warn` and verify `openspec/changes/fix-thread-resolved-users/specs/thread-command/spec.md` still matches the implemented response contract exercised by `tests/thread_integration_tests.rs`)

## Future Work

- Consider applying the same response-level hydration contract to other wrapper commands that currently enrich messages locally.
- If performance or rate limits become a concern, add batching/caching strategy refinements for repeated `users.info` lookups across commands.

## Final Validation

Expected proposal validation commands:
`cflx openspec validate fix-thread-resolved-users --strict`
`cflx openspec validate fix-thread-resolved-users --strict --evidence warn`
