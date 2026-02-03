# 設計: OAuth 認証と auth コマンド

## OAuth フロー
- PKCE を使用する
- callback は `127.0.0.1` の一時サーバで受ける
- state を検証して CSRF を防止する

## 依存注入
- OAuth エンドポイントはテスト用に差し替え可能にする（base URL 設定）
- ブラウザ起動は失敗時に URL を表示する

## auth コマンド
- `auth login`: OAuth 実行 → token 保存 → profile 更新
- `auth status`: profile 情報の表示（token の存在確認）
- `auth list`: profile 一覧
- `auth rename`: profile 名の変更
- `auth logout`: token 削除 + profile 削除

## テスト方針
- OAuth 交換はモックサーバで検証する
- 実際の Slack OAuth は Future Work とする
