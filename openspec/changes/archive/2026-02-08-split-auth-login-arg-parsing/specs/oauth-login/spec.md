# oauth-login Change Proposal

## ADDED Requirements

### Requirement: auth login の引数解釈規約は内部解析分離後も維持される
`auth login` の引数解析を内部的に分離した後も、既存のオプション解釈（排他制約、既定値適用、非対話制約、エラーメッセージ種別）は互換性を維持しなければならない。(MUST)

#### Scenario: 排他オプション制約が維持される
- Given `auth login --cloudflared --ngrok` を実行する
- When 引数解析が分離された実装で評価される
- Then 排他違反エラーが返る
- And OAuth 実行フェーズには進まない

### Requirement: OAuth ログインの実行コアは重複なく一貫して適用される
標準ログイン経路と拡張ログイン経路は、認証 URL 生成からトークン交換までのコア OAuth 実行処理を内部的に共有しなければならない。(MUST)
この内部統合後も、既存の成功条件・失敗条件・保存結果は互換でなければならない。(MUST)

#### Scenario: 標準/拡張ログインで同一 OAuth 実行コアを使用する
- Given 標準ログインまたは拡張ログインを実行する
- When OAuth フローが開始される
- Then 認証コード取得からトークン交換までの処理は同一の内部コアを通る
- And 既存と同じ条件で profile と token が保存される
