# oauth-login 仕様（差分）

## MODIFIED Requirements

### Requirement: redirect_uri の解決に ngrok を選択できる

`auth login` は `--ngrok [path]` を受け付けなければならない (MUST)。

`--ngrok` が指定され、`path` が省略された場合は実行ファイル名 `ngrok`（PATH 探索）を使用しなければならない (MUST)。

`--ngrok <path>` が指定された場合は、その `path` を使用しなければならない (MUST)。

`--ngrok` が指定された場合は `ngrok http 8765` を起動し、公開 URL を抽出して `{public_url}/callback` を redirect_uri として使用しなければならない (MUST)。OAuth 完了後に ngrok プロセスを停止しなければならない (MUST)。

`--ngrok` と `--cloudflared` が同時に指定された場合はエラーにしなければならない (MUST)。

#### Scenario: `--ngrok`（path 省略）で ngrok を起動し redirect_uri を解決する
- Given `auth login --ngrok` を実行する
- When OAuth フローを開始する
- Then 実行ファイル名 `ngrok` を使用して `ngrok http 8765` を起動する
- And ngrok の出力から `https://*.ngrok-free.app` の公開 URL を抽出する
- And redirect_uri を `{public_url}/callback` に設定する
- And OAuth 完了後に ngrok を停止する

#### Scenario: `--ngrok <path>` 指定で ngrok を起動し redirect_uri を解決する
- Given `auth login --ngrok <path>` を実行する
- When OAuth フローを開始する
- Then 指定された `path` の ngrok を起動する
- And 公開 URL を抽出して redirect_uri に設定する

#### Scenario: `--ngrok` と `--cloudflared` の同時指定はエラーになる
- Given `auth login --ngrok --cloudflared` を実行する
- When 引数を解決する
- Then 競合エラーが表示され OAuth フローを開始しない
