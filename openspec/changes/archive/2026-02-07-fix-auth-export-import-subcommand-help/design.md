# 設計: auth export/import のヘルプフラグ処理

## 設計方針
- 変更範囲は `auth export` / `auth import` の引数解釈に限定する
- 既存の引数互換性を維持しつつ、`-h` / `--help` を早期ハンドリングする

## 想定動作
- `slack-rs auth export -h` / `--help`: export 用の usage/options を表示して終了コード 0
- `slack-rs auth import -h` / `--help`: import 用の usage/options を表示して終了コード 0
- ヘルプ経路では永続化処理や復号処理を実行しない

## テスト方針（mock-first）
- 外部認証情報不要の CLI テストで検証する
- 少なくとも以下を固定する
  - `auth export -h` と `auth export --help` が成功
  - `auth import -h` と `auth import --help` が成功
  - 出力にサブコマンド固有 usage が含まれる

## トレードオフ
- 引数分岐が増える
- ただし標準 CLI 操作性と利用者期待の整合性が向上する
