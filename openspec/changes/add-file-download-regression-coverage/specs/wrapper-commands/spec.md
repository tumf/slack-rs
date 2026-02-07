## MODIFIED Requirements

### Requirement: `file download` retrieves Slack files with authentication
`file download` は `file_id` 指定と `--url` 指定の両方で、認証付きダウンロードを実行できなければならない。(MUST)

`file_id` 指定時は `files.info` へ正しい形式の引数を送信し、取得した private URL へ遷移しなければならない。(MUST)

#### Scenario: `file_id` 指定で `files.info` に正しい引数を送る
- Given `file download F123` を実行する
- When Slack API へメタデータ取得を行う
- Then `files.info` には `file=F123` が送信される
- And `invalid_arguments` で失敗しない

#### Scenario: `--url` 指定で認証付き直接ダウンロードを行う
- Given `file download --url https://files.slack.com/...` を実行する
- When 対象 URL を取得する
- Then `Authorization: Bearer ...` を付与してダウンロードする
