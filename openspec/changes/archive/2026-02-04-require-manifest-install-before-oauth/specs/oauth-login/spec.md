# oauth-login Specification

## ADDED Requirements
### Requirement: Prompt manifest installation before OAuth authentication

`auth login` は OAuth 認証を開始する前に、生成済みの Slack App Manifest をユーザーにインストールさせるための案内を表示し、Slack App 管理ページをブラウザで開いた上で、ユーザーの確認入力を待たなければならない (MUST)。

#### 解釈/対象
- 本要件の対象は CLI の `auth login` サブコマンドであり、実装上はメインの実行経路（例: `src/main.rs` から呼ばれるログイン処理）に適用される (MUST)。
- 「OAuth 認証フローを開始する」とは、認可 URL を表示/ブラウザ起動する前の段階を指す (MUST)。

#### エラーハンドリング
- Manifest の生成または一時ファイルへの保存に失敗した場合、OAuth 認証フローを開始してはならない (MUST)。
- Slack App 管理ページのブラウザ起動に失敗した場合は、URL を表示して手動で開けるようにしなければならない (MUST)。
- ユーザー確認待ちの入力読み取りに失敗した場合、OAuth 認証フローを開始してはならない (MUST)。

#### Scenario: OAuth 認証前に案内と確認待ちが行われる
- Given `auth login` を実行する
- When マニフェストが生成され保存される
- When マニフェストが生成され一時ファイルへ保存される
- Then `https://api.slack.com/apps` を開く試行が行われる
- And マニフェストの保存先が表示される
- And ユーザーの確認入力を待ってから OAuth 認証フローを開始する
