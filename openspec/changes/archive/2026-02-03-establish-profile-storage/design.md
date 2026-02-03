# 設計: プロファイル/トークン保存

## 方針
- プロファイルは非秘密情報のみを `profiles.json` に保存する
- トークンは OS keyring に保存し、平文ファイルには保存しない
- 安定キーは `(team_id, user_id)` の組み合わせとする

## 設定ファイル
- 位置は `directories` crate の `ProjectDirs` に従う
- スキーマは versioned で将来の移行を想定する

## keyring のキー設計
- service: `slackcli`
- username: `{team_id}:{user_id}`
- profile 名のリネームは keyring に影響しない

## テスト方針
- keyring への依存を避けるため、インメモリの TokenStore 実装を用意する
- `profiles.json` は temp dir を使って読み書き検証する
