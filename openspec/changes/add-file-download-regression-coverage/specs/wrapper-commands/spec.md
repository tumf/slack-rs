## MODIFIED Requirements

### Requirement: `file download` retrieves Slack files with authentication
`file download` は `file_id` 指定と `--url` 指定の両方で、認証付きダウンロードを実行できなければならない。(MUST)

`file_id` 指定時は `files.info` へ正しい形式の引数（form-encoded）を送信し、取得した private URL へ遷移しなければならない。(MUST)

`--url` 指定時は `files.info` を呼ばず、指定された URL へ直接認証ヘッダ付きでアクセスしなければならない。(MUST)

#### Scenario: `file_id` 指定で `files.info` に正しい引数を送る（回帰防止）
- Given `file download F123` を実行する
- When Slack API へメタデータ取得を行う
- Then `files.info` には `file=F123` が form パラメータとして送信される
- And Content-Type は `application/x-www-form-urlencoded` である
- And `invalid_arguments` で失敗しない

#### Scenario: `--url` 指定で認証付き直接ダウンロードを行う（回帰防止）
- Given `file download --url https://files.slack.com/...` を実行する
- When 対象 URL を取得する
- Then `files.info` は呼ばれない
- And ダウンロードリクエストに `Authorization: Bearer ...` ヘッダが含まれる

#### Scenario: 画像ファイルを `file_id` 経路でダウンロードする
- Given PNG 形式のファイル ID が存在する
- When `file download <image_file_id>` を実行する
- Then `files.info` で取得した URL から画像をダウンロードする
- And ダウンロードされたファイルは有効な PNG 形式である

#### Scenario: 動画ファイルを `--url` 経路でダウンロードする
- Given MP4 形式のプライベート URL が存在する
- When `file download --url <video_url>` を実行する
- Then 認証ヘッダ付きで URL から直接ダウンロードする
- And ダウンロードされたファイルは有効な MP4 形式である
