## Why
- 現状は `scope=` のみで `user_scope` を扱えず、user 用の権限が混在すると `invalid_scope` で失敗しやすい。
- `auth login` の redirect_uri は実行環境やネットワーク構成に依存し、毎回同じ値を固定できないことがある。
- cloudflared tunnel を使える環境では動的に公開 URL を生成して楽に OAuth 認証を完了できるが、導入できない環境もある。
- 保存した設定値をそのまま Slack App の Manifest に反映できると運用が単純化する。

## What Changes
Slack OAuth の `user_scope` を設定できるようにし、`slack-rs auth login` で redirect URL（redirect_uri）の解決方法を選べるようにする（cloudflared tunnel は任意）。保存した OAuth 設定（redirect_uri と bot/user scopes）から Slack App Manifest を生成できるようにする。

## 概要
Slack OAuth の `user_scope` を設定できるようにし、`slack-rs auth login` で redirect URL（redirect_uri）の解決方法を選べるようにする（cloudflared tunnel は任意）。保存した OAuth 設定（redirect_uri と bot/user scopes）から Slack App Manifest を生成できるようにする。

## 背景
- 現状は `scope=` のみで `user_scope` を扱えず、user 用の権限が混在すると `invalid_scope` で失敗しやすい。
- `auth login` の redirect_uri は実行環境やネットワーク構成に依存し、毎回同じ値を固定できないことがある。
- cloudflared tunnel を使える環境では動的に公開 URL を生成して楽に OAuth 認証を完了できるが、導入できない環境もある。
- 保存した設定値をそのまま Slack App の Manifest に反映できると運用が単純化する。

## ゴール
- `user_scope` を OAuth 認可 URL に反映できる。
- `auth login` で bot/user スコープを対話的に入力できる（デフォルト入力値は両方とも `all`）。
- 生成する Slack App Manifest は、`auth login` で選択されたスコープ（bot/user）に基づく。
- cloudflared は OPTIONAL とし、`auth login` に `--cloudflared [path]` オプションで cloudflared 実行ファイルを指定できる（`path` は省略可能）。
  - `--cloudflared` が存在し `path` が省略された場合、CLI は `cloudflared`（PATH から探索）を実行ファイルとして使用する。
  - `--cloudflared <path>` が存在する場合、CLI はその `path` を実行ファイルとして使用する。
- `--cloudflared` が指定されない場合、`auth login` は redirect_uri をユーザーにプロンプトして決定する。
- `--cloudflared` が指定される場合、OAuth は cloudflared tunnel の公開 URL を用いた `{public_url}/callback` を redirect_uri として使用する。
- `auth login` 実行時に入力情報から自動的に Slack App Manifest を生成する。

## 非ゴール
- Slack App への自動反映（API 経由更新）
- OAuth フローの UI/ブラウザ起動の変更
- cloudflared のインストール自動化（ユーザーが事前にインストール済みであることを前提とする）

## 影響範囲
- OAuth 設定の保存フォーマット（`scopes` を `bot_scopes`/`user_scopes` に分割）
- 既存プロファイルの後方互換（旧 `scopes` は bot 側に移譲）
- `auth login` の redirect_uri 処理（cloudflared を使う場合の動的生成、使わない場合のプロンプト入力）

## 成果物
- `user_scope` 対応
- cloudflared tunnel（任意）統合
- `auth login` 実行時の Manifest 自動生成
- 新/更新仕様（specs）
