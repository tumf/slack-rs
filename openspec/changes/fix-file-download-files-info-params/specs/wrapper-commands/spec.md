## MODIFIED Requirements

### Requirement: File download retrieves file metadata before download
`file download` は `files.info` 呼び出し時に `file_id` を Slack API が解釈可能なパラメータ形式（フォームまたはクエリ）で送信しなければならない。(MUST)
`file_id` を JSON ボディとして送信してはならない。(MUST NOT)

#### Scenario: `file_id` が正しい形式で送信される
- Given `file download --file-id F123` を実行する
- When `file download` が `files.info` を呼び出す
- Then `file_id=F123` はフォームまたはクエリのパラメータとして送信される
- And Slack API から `invalid_arguments`（パラメータ形式不正）を受けない

#### Scenario: JSON ボディ送信を防止する
- Given `files.info` への送信形式を検証するテストダブル（モック/スタブ）がある
- When `file_id` を JSON ボディとして送る実装を適用する
- Then テストは不正な送信形式として失敗する
