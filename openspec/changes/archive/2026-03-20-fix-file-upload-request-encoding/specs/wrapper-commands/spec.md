## MODIFIED Requirements

### Requirement: file upload で外部アップロード方式を実行できる

`file upload` MUST call `files.getUploadURLExternal` using Slack-compatible parameter encoding for `filename` and `length`.
`file upload` MUST POST the file's raw bytes to the returned `upload_url`.
`file upload` MUST call `files.completeUploadExternal` to finish the upload and include `files` plus any requested sharing parameters.
`file upload` MUST NOT call the deprecated `files.upload` method.

#### Scenario: file upload sends Slack-compatible parameters before uploading bytes

- Given a valid profile and token exist
- And a local file path resolves to readable bytes
- When `file upload ./report.pdf --yes --channel=C123 --comment="Weekly report"` is executed
- Then `files.getUploadURLExternal` is called with `filename=report.pdf` and the file byte length using Slack-compatible request encoding
- And the returned `upload_url` receives the raw file bytes
- And `files.completeUploadExternal` receives `files`, `channel_id`, and `initial_comment`

#### Scenario: wrapper regression test prevents JSON request bodies for upload URL creation

- Given the integration test suite runs against the file upload command
- When the request to `files.getUploadURLExternal` is inspected
- Then the test asserts the encoded `filename` and `length` parameters are present
- And the test fails if the wrapper sends the upload URL request as the previously broken JSON body
