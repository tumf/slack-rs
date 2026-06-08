## Implementation Tasks

- [ ] Update thread user resolution to treat `users.info` Slack lookup failures as unresolved user IDs rather than command-fatal errors. (verification: unit - add/adjust tests in `src/commands/conv/format.rs` or `tests/thread_integration_tests.rs` for `resolve_thread_users` using a mocked `users.info` `ok:false` response and assert the ID appears only in `unresolved_user_ids`; completion: `src/commands/conv/format.rs` no longer returns `Err` for user lookup failures after successful thread retrieval)
- [ ] Preserve command-fatal behavior for primary thread retrieval and non-recoverable output construction errors. (verification: integration - `cargo test --test thread_integration_tests` covers `thread_get`/`conversations.replies` behavior in `src/commands/thread.rs` and `tests/thread_integration_tests.rs`; completion: only enrichment lookup failures from `src/commands/conv/format.rs` are downgraded)
- [ ] Verify default CLI output uses response-level metadata without mutating Slack messages. (verification: integration - add a CLI-path test in `tests/thread_integration_tests.rs` or a dedicated CLI integration test that invokes `src/cli/mod.rs::run_thread_get`/equivalent command wiring and asserts default JSON envelope has `response.data.resolved_users` and no per-message `users` field; completion: test fails if metadata is placed on messages)
- [ ] Verify unresolved IDs are represented separately in default CLI output. (verification: integration - add a CLI-path test in `tests/thread_integration_tests.rs` with one resolved user and one mocked unresolved lookup, asserting hydrated profiles exclude ID-only fake objects and `response.data.unresolved_user_ids` contains the missing ID; completion: test fails if the command exits with an enrichment error or fabricates a profile)
- [ ] Verify raw output remains Slack-native. (verification: integration - add a CLI-path `thread get --raw` test in `tests/thread_integration_tests.rs` or a dedicated CLI integration test that asserts no `resolved_users` or `unresolved_user_ids` fields and no user lookup calls are required; completion: test fails if raw output is enriched)
- [ ] Run targeted thread verification. (verification: integration - `cargo test --test thread_integration_tests`; completion: command passes locally)

## Final Validation

Archive validation itself is the authoritative final OpenSpec validation gate.
Expected archive gate: `cflx openspec validate fix-thread-user-resolution-failures --archive-gate`

## Future Work

- Consider lookup concurrency, batching, or a configurable enrichment toggle only if real workspace usage shows latency or rate-limit pressure.
