## MODIFIED Requirements

### Requirement: Prompt manifest installation before OAuth authentication

`auth login --cloudflared` および `auth login --ngrok` は、OAuth 認証を開始する前に redirect URI を確定して Slack App Manifest を生成・保存し、Slack App 管理ページで app 作成を行うための案内を表示し、ユーザーの確認入力を待たなければならない (MUST)。

これらの manifest-first 経路では、Manifest 生成前に `Client ID` または `Client Secret` の入力を要求してはならない (MUST NOT)。

#### Scenario: cloudflared 経路では manifest 案内が credentials 入力より先に行われる
- Given `auth login --cloudflared` を実行する
- When cloudflared tunnel が起動して redirect URI が確定する
- Then Slack App Manifest が生成・保存される
- And Slack App 作成の案内と確認入力が表示される
- And その時点では `Client ID` と `Client Secret` の入力は要求されない
- And ユーザー確認後に credentials 解決へ進む

#### Scenario: ngrok 経路では manifest 案内が credentials 入力より先に行われる
- Given `auth login --ngrok` を実行する
- When ngrok tunnel が起動して redirect URI が確定する
- Then Slack App Manifest が生成・保存される
- And Slack App 作成の案内と確認入力が表示される
- And その時点では `Client ID` と `Client Secret` の入力は要求されない
- And ユーザー確認後に credentials 解決へ進む
