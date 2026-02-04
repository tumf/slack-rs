# oauth-login 仕様（差分）

## MODIFIED Requirements

### Requirement: Resolve redirect_uri (cloudflared is OPTIONAL)

`auth login` は `--cloudflared [path]` オプションで cloudflared 実行ファイルを受け付けなければならない (MUST)。

`--cloudflared` が存在し `path` が省略された場合、`auth login` は実行ファイル名 `cloudflared`（PATH から探索）を使用しなければならない (MUST)。

`--cloudflared <path>` が存在する場合、`auth login` は指定された `path` を使用しなければならない (MUST)。

`--cloudflared` が指定された場合、`auth login` は cloudflared tunnel プロセスを起動し、生成された公開 URL を抽出し、それを OAuth フローの `redirect_uri` として使用しなければならない (MUST)。tunnel は OAuth フロー完了後に停止しなければならない (MUST)。

`auth login` は `--ngrok [path]` を受け付けなければならない (MUST)。

`--ngrok` が指定され、`path` が省略された場合は実行ファイル名 `ngrok`（PATH 探索）を使用しなければならない (MUST)。

`--ngrok <path>` が指定された場合は、その `path` を使用しなければならない (MUST)。

`--ngrok` が指定された場合は `ngrok http 8765` を起動し、公開 URL を抽出して `{public_url}/callback` を redirect_uri として使用しなければならない (MUST)。OAuth 完了後に ngrok プロセスを停止しなければならない (MUST)。

`--ngrok` と `--cloudflared` が同時に指定された場合はエラーにしなければならない (MUST)。

`--cloudflared` が指定されない場合、`auth login` はユーザーに redirect_uri をプロンプトして取得し、その値を OAuth フローの `redirect_uri` として使用しなければならない (MUST)。この場合、cloudflared を起動してはならない (MUST NOT)。

#### ADDED Scenario: `--ngrok`（path 省略）で ngrok を起動し redirect_uri を解決する
- Given `auth login --ngrok` を実行する
- When OAuth フローを開始する
- Then 実行ファイル名 `ngrok` を使用して `ngrok http 8765` を起動する
- And ngrok の出力から `https://*.ngrok-free.app` の公開 URL を抽出する
- And redirect_uri を `{public_url}/callback` に設定する
- And OAuth 完了後に ngrok を停止する

#### ADDED Scenario: `--ngrok <path>` 指定で ngrok を起動し redirect_uri を解決する
- Given `auth login --ngrok <path>` を実行する
- When OAuth フローを開始する
- Then 指定された `path` の ngrok を起動する
- And 公開 URL を抽出して redirect_uri に設定する

#### ADDED Scenario: `--ngrok` と `--cloudflared` の同時指定はエラーになる
- Given `auth login --ngrok --cloudflared` を実行する
- When 引数を解決する
- Then 競合エラーが表示され OAuth フローを開始しない
