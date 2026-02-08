## MODIFIED Requirements

### Requirement: auth login の引数解釈規約は内部解析分離後も維持される
`auth login` の引数解析を内部的に分離した後も、既存のオプション解釈（排他制約、既定値適用、非対話制約、エラーメッセージ種別）は互換性を維持しなければならない。(MUST)

#### Scenario: 排他オプション制約が維持される
- Given `auth login --cloudflared --ngrok` を実行する
- When 引数解析が分離された実装で評価される
- Then 排他違反エラーが返る
- And OAuth 実行フェーズには進まない
