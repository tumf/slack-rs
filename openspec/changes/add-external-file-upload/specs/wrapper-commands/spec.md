# wrapper-commands

## ADDED Requirements

### Requirement: file upload で外部アップロード方式を実行できる
`file upload` は `files.getUploadURLExternal` を呼び出し、取得した `upload_url` へファイルの生バイトを送信し、`files.completeUploadExternal` を呼び出して共有を完了しなければならない。(MUST)
`files.completeUploadExternal` には `files`（`id` と任意 `title`）を含め、`--channel`/`--channels`/`--comment` が指定されている場合は対応するパラメータを送信しなければならない。(MUST)
旧方式の `files.upload` を呼び出してはならない。(MUST NOT)

#### Scenario: channel を指定して file upload を実行する
- Given 有効な profile と token が存在する
- When `file upload ./report.pdf --allow-write --channel=C123 --comment="Weekly report"` を実行する
- Then `files.getUploadURLExternal` が `filename` と `length` 付きで呼ばれる
- And 返却された `upload_url` にファイルの生バイトが送信される
- And `files.completeUploadExternal` に `files` と `channel_id` と `initial_comment` が送信される
