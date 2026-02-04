# oauth-login Specification

## ADDED Requirements
### Requirement: Prompt manifest installation before OAuth authentication

`auth login` は OAuth 認証を開始する前に、生成済みの Slack App Manifest をユーザーにインストールさせるための案内を表示し、Slack App 管理ページをブラウザで開いた上で、ユーザーの確認入力を待たなければならない (MUST)。

#### Scenario: OAuth 認証前に案内と確認待ちが行われる
- Given `auth login` を実行する
- When マニフェストが生成され保存される
- Then `https://api.slack.com/apps` を開く試行が行われる
- And マニフェストの保存先が表示される
- And ユーザーの確認入力を待ってから OAuth 認証フローを開始する
