# 設計: 汎用 API 呼び出し

## HTTP クライアント
- `reqwest` を使用
- base URL はテスト用に差し替え可能にする

## 入力パラメータ
- `key=value` は form-urlencoded で送信
- `--json` 指定時は JSON body を送信
- `--get` 指定時は GET を使用（デフォルトは POST）

## 出力形式
- Slack API の raw response を `response` に格納
- `meta` に profile/team/user/method を付与

## リトライ
- 429 は Retry-After を優先
- それ以外は指数バックオフ + jitter
- 上限回数を設定する

## テスト方針
- モック HTTP サーバで成功/失敗/429 を検証
- 実 Slack 呼び出しは Future Work
