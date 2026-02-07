## MODIFIED Requirements

### Requirement: `file download` retrieves Slack files with authentication
`file download <file_id>` MUST call `files.info` to resolve the download URL and retrieve the file via authenticated GET. (MUST)
`file download <file_id>` MUST prefer `url_private_download` and fall back to `url_private` when unavailable. (MUST)
`file download --url <url_private_or_download>` MUST skip `files.info` and directly retrieve from the specified URL. (MUST)
`file download` の HTTP 取得は、Slack の標準的な配布経路に対応するため 3xx リダイレクトを追従しなければならない。(MUST)

#### Scenario: 3xx リダイレクト先を追従してダウンロードする
- Given 有効なトークンが存在し、初回 URL が 302 を返す
- When `file download F1234567890` を実行する
- Then クライアントは `Location` を追従し、最終到達先で認証付き GET を完了する
- And 最終応答が 2xx の場合はダウンロード成功として扱う

### Requirement: `file download` explicitly errors on HTML responses and HTTP failures
`file download` MUST exit with non-zero status and return a concise error message when the download response is non-2xx. (MUST)
`file download` MUST return an error indicating a possible URL mismatch or authentication issue when the download response has `Content-Type: text/html`. (MUST)
`Content-Type: text/html` の失敗時は、診断を容易にするためレスポンス本文の短い先頭断片（安全に切り詰めたスニペット）をエラーメッセージ文脈に含めなければならない。(MUST)

#### Scenario: HTML 応答時にヒントと本文スニペットを表示して失敗する
- Given ダウンロード先が `Content-Type: text/html` と短い HTML 本文を返す
- When `file download F123` を実行する
- Then コマンドは失敗終了する
- And stderr には URL 種別不一致または認証問題のヒントが含まれる
- And stderr には本文先頭の短いスニペットが含まれる
