## ADDED Requirements

### Requirement: `file download` は認証付きで Slack ファイルを取得できる
`file download <file_id>` は `files.info` を呼び出してダウンロード URL を解決し、認証付き GET でファイルを取得しなければならない。(MUST)
`file download <file_id>` は `url_private_download` を優先し、未提供時は `url_private` にフォールバックしなければならない。(MUST)
`file download --url <url_private_or_download>` は `files.info` を省略して指定 URL を直接取得できなければならない。(MUST)

#### Scenario: `<file_id>` 指定で URL 解決してダウンロードする
- Given 有効な profile と token が存在する
- When `file download F1234567890` を実行する
- Then `files.info` が `file=F1234567890` で呼び出される
- And `url_private_download`（未提供時は `url_private`）に対して認証付き GET が実行される

#### Scenario: `--url` 指定で直接ダウンロードする
- Given 有効な token が存在する
- When `file download --url https://files.slack.com/files-pri/...` を実行する
- Then `files.info` を呼ばずに指定 URL へ認証付き GET が実行される

### Requirement: `file download` は保存先と出力方法を制御できる
`file download` は `--out <path>` を受け付け、指定先へダウンロード内容を書き込まなければならない。(MUST)
`--out` 未指定時は安全なデフォルトファイル名でカレントディレクトリに保存しなければならない。(MUST)
`--out -` 指定時は標準出力へバイナリをストリームし、データ本体以外を標準出力へ出力してはならない。(MUST NOT)

#### Scenario: `--out -` で標準出力へストリームする
- Given `file download F123 --out -` を実行する
- When ダウンロードが成功する
- Then ファイルバイト列が標準出力へ出力される
- And 進捗や診断メッセージは標準出力に混在しない

### Requirement: `file download` は HTML 応答と HTTP 失敗を明確にエラー化する
`file download` はダウンロード応答が非 2xx の場合、非ゼロ終了コードで失敗し、簡潔なエラーメッセージを返さなければならない。(MUST)
`file download` はダウンロード応答の `Content-Type` が `text/html` の場合、誤 URL または認証不備の可能性を示すエラーを返さなければならない。(MUST)

#### Scenario: HTML 応答を検出してエラーにする
- Given ダウンロード先が `Content-Type: text/html` を返す
- When `file download F123` を実行する
- Then コマンドは失敗終了する
- And エラーメッセージに「URL 種別ミスまたは認証問題」のヒントが含まれる

### Requirement: `file download` は write ガードの対象外である
`file download` は read 操作として扱い、`SLACKCLI_ALLOW_WRITE` が `false`/`0` の場合でも実行可能でなければならない。(MUST)

#### Scenario: `SLACKCLI_ALLOW_WRITE=false` でもダウンロードできる
- Given `SLACKCLI_ALLOW_WRITE=false` が設定されている
- When `file download F123` を実行する
- Then write ガード起因の拒否は発生しない
