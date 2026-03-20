# Fix file upload request encoding for external uploads

## Problem/Context

- Issue #29 reports that `slack-rs file upload <path> --yes` consistently fails with `files.getUploadURLExternal failed: invalid_arguments`.
- The current `file upload` wrapper in `src/commands/file.rs` sends JSON bodies to Slack API endpoints, while the generic `api call` command defaults to form-encoded POST bodies.
- Direct `api call files.getUploadURLExternal filename=test.txt length=4` succeeds, which indicates the wrapper command is not matching Slack's accepted request encoding for this method.
- Existing integration coverage verifies the high-level upload flow but does not assert the request encoding or body shape sent to `files.getUploadURLExternal`.

## Proposed Solution

- Update `file upload` so that Slack API endpoints in the external upload flow use Slack-compatible request encoding, starting with `files.getUploadURLExternal`.
- Preserve the current three-step external upload flow: request upload URL, send raw file bytes to the returned `upload_url`, then finalize with `files.completeUploadExternal`.
- Add regression coverage that verifies the wrapper sends the expected encoded parameters for `filename` and `length`, preventing accidental regression back to JSON payloads.

## Acceptance Criteria

- `file upload <path> --yes` no longer fails with `invalid_arguments` when `files.getUploadURLExternal` is called with a valid file.
- `files.getUploadURLExternal` receives `filename` and `length` using the Slack-compatible encoding expected by the API.
- The external upload still sends raw file bytes to the returned `upload_url` and completes with `files.completeUploadExternal`.
- Regression tests fail if the wrapper reverts to sending the `files.getUploadURLExternal` request in the wrong format.

## Out of Scope

- Changing `file upload` flags or user-facing UX beyond fixing the broken upload path.
- Reworking idempotency, profile resolution, or OAuth/token storage behavior.
- Adding multipart uploads, progress reporting, or multi-file upload support.
