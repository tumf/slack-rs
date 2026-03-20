## Implementation Tasks

- [ ] Update `src/commands/file.rs` so `files.getUploadURLExternal` uses Slack-compatible request encoding instead of the current JSON body (verification: inspect `src/commands/file.rs` and run `cargo test test_file_upload_external_flow`).
- [ ] Keep the external upload flow intact for raw byte upload and completion, adjusting `files.completeUploadExternal` encoding only if required to match Slack behavior (verification: run `cargo test test_file_upload_external_flow`).
- [ ] Strengthen `tests/commands_integration.rs` to assert the request shape for `files.getUploadURLExternal`, including encoded `filename` and `length` values (verification: inspect `tests/commands_integration.rs` and run `cargo test test_file_upload_external_flow`).
- [ ] Run targeted regression coverage for file upload behavior and request formatting (verification: run `cargo test test_file_upload_external_flow test_file_upload_nonexistent_file`).

## Future Work

- Verify against a real Slack workspace with `SLACKCLI_ALLOW_WRITE=1` after the code change is merged, because live API credentials are not available during proposal drafting.
