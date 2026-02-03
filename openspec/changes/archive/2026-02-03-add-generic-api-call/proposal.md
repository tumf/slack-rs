# 変更提案: 汎用 API 呼び出しコマンド

## 背景
Slack Web API を網羅的に扱うためには、任意メソッドを呼べる `api call` が中核となる。

## 目的
- `api call <method>` による汎用 API 呼び出し
- form と JSON の両方に対応
- レート制限の取り扱い（429 + Retry-After）
- 返却 JSON に実行コンテキストを付与

## 対象範囲
- `api call` コマンドの CLI ルーティング
- HTTP クライアント（reqwest）
- リトライ/バックオフ

## 対象外
- ラッパーコマンド
- OAuth 認証

## 依存関係
- プロファイル/トークン保存基盤

## 成功条件
- 任意 Slack メソッドを呼び出せる
- 429 時に Retry-After を尊重する
- 出力に profile/team/user の meta を含める
