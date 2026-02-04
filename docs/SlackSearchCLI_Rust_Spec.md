---
title: Slack Web API CLI（Rust）仕様書（案）
date: 2026-02-03
status: draft
tags:
  - slack
  - cli
  - rust
  - oauth
  - spec
---

# Slack Web API CLI（Rust）仕様書（案）

## 概要
複数の Slack ワークスペースに対して、**tumf本人権限（OAuth / User Token）**で Slack の Public Web API を操作できる **汎用CLI** を Rust で実装する。

gog のように「プロファイル切替 + サブコマンド + JSON出力 + パイプ前提」の使い心地を目指す。

> 重要な現実制約
> - Slack Web API は「網羅＝必要スコープが爆増」しやすい。
> - ワークスペース（特に取引先）では **スコープ承認が通らない**ことがある。
> - したがって本プロジェクトの方針は **“API網羅はCLI機能として”**（= どのメソッドも叩ける）を提供しつつ、
>   **デフォルト運用は最小スコープ（Read-only）**、必要に応じて段階的に拡張する。

---

## 1. ゴール / 非ゴール

### 1.1 ゴール
- **Slack Web API の任意メソッドを呼び出せる**（網羅性）
  - 例: `slackcli --profile acme api call conversations.history channel=C123 limit=200`
- 主要なよく使う操作は **人間向けの薄いラッパーコマンド**として提供
  - 例: `slackcli search ...`, `slackcli conv history ...`, `slackcli msg post ...`
- 出力は **JSON優先**（textはサマリ用途）
- **複数ワークスペース**を profile として扱う
- 認証は **OAuth（PKCE + localhost callback）**

### 1.2 非ゴール（初期）
- RTM / Socket Mode 等の常時接続によるイベント購読（別プロダクト）
- Enterprise Grid 管理系（監査ログ等）は要検討

---

## 2. アーキテクチャ方針（「網羅」を現実的にする）

### 2.1 2階建て
1) **汎用レイヤ**: `api call <method> [key=value...]`
   - Slack Web API の *どのメソッドでも* 叩ける
   - 未実装領域も即カバーできる
2) **高頻度コマンド**: `search`, `conv`, `msg`, `users`, `files` など
   - 使い勝手と事故防止（引数チェック、既定値、整形）

### 2.2 メソッド定義の扱い（任意）
- “網羅”を補助するため、可能なら **メソッドメタデータ（引数・説明）を取り込み**
  - 取り込み元は開発時に選定（Slack公式のメタ一覧/HTMLスクレイプ/手動スナップショット等）
- 最低限は **メタ無しでも動く**（key=value をそのままPOST）

---

## 3. 認証方式（OAuth）

- `slackcli auth login --profile <name>`
  - ブラウザを起動して OAuth
  - `oauth.v2.access` で token を取得
  - `team.id` / `team.name` を profile に保存

### 3.1 Token保存
- OS Keychain/SecretService を優先（平文禁止）
- debugログは token を必ずマスク

### 3.2 複数ワークスペース
- profile = 1 workspace
- `--profile` 省略時はエラー（混同事故防止）

---

## 4. スコープ戦略（段階的）

「Slack Public API を網羅」= すべてのスコープを最初から要求しない。

### 4.1 スコープセット（例）
- **Read-only セット（初期推奨）**
  - `search:read`
  - `channels:read`, `groups:read`, `im:read`, `mpim:read`
  - `channels:history`, `groups:history`, `im:history`, `mpim:history`
  - `users:read`

- **Write セット（今回: 必須）**
  - `chat:write`（投稿/返信）
  - `reactions:write`（リアクション）
  - （必要なら）`chat:write.public`（参加していないパブリックチャンネルへの投稿が必要な場合のみ）

- **Admin/Enterprise系**
  - 取引先WSでは通らない前提で別扱い

CLI側は
- `slackcli auth status` で現在のスコープを可視化
- 目的コマンド実行時に「不足スコープ」を表示

---

## 5. CLI コマンド仕様（提案）

### 5.1 グローバル
- `--profile <name>`（必須）
- `--format json|text`（デフォルト: json）
- `--no-color`
- `--debug`（HTTPログはマスク）

### 5.2 auth
- `slackcli auth login --profile <name>`
- `slackcli auth status [--profile <name>]`
- `slackcli auth logout --profile <name>`

### 5.3 api（網羅の核）
- `slackcli api call <method> [key=value ...] [--json '{...}']`
  - method例: `search.messages`, `conversations.history`, `users.info`
  - `key=value` は form-urlencoded を基本、`--json` で JSON body も可能に
  - Slackの仕様に合わせて GET/POST を切替（原則POSTでも可）

- `slackcli api methods`（任意：メタ取り込みができたら）
  - メソッド一覧表示、fuzzy検索

### 5.4 よく使うラッパー（最初に実装する候補）
- `slackcli search "<query>" [--limit N] [--sort timestamp|score] [--order asc|desc]`
- `slackcli conv list [--types public,private,im,mpim] [--limit N]`
- `slackcli conv history --channel <C...> [--oldest <ts>] [--latest <ts>] [--limit N]`
- `slackcli users info --user <U...>`

Write（今回入れる）
- `slackcli msg post --channel <C...> --text "..." [--thread-ts <ts>]`
- `slackcli msg update --channel <C...> --ts <ts> --text "..."`
- `slackcli msg delete --channel <C...> --ts <ts>`
- `slackcli react add --channel <C...> --ts <ts> --emoji ":white_check_mark:"`
- `slackcli react remove --channel <C...> --ts <ts> --emoji ":white_check_mark:"`

---

## 6. レート制限 / リトライ
- 429 + Retry-After を尊重
- exponential backoff + jitter
- 高負荷コマンド（history大量取得等）は並列度制御

---

## 7. セキュリティ / 事故防止
- token を出力・ログしない
- profile必須 + 出力に `team.name` を必ず含める
- write系は **事故防止の二重ガード**
  - デフォルトで write 無効
  - 実行には `SLACKRS_ENABLE_WRITE=1` が必要（環境変数ガード）
  - さらに破壊的操作（delete等）は `--yes` なしでは実行しない
  - `--lang` に応じて確認プロンプトも翻訳（i18n）

---

## 8. 実装（Rust）

### 8.1 利用可能な crate 候補（調査結果の追記）

#### Slack API クライアント
- **`slack-morphism`**
  - Slack Web API / Events API / Socket Mode / Block Kit まで含む “現役” のクライアントライブラリ
  - docs / examples がある（公式サイト: https://slack-rust.abdolence.dev）
  - OAuth ルートの例もある（`/auth/install`, `/auth/callback` 等）
  - 本プロジェクトでは **Web API（user token）呼び出し部分**で採用候補
  - 注意: 依存が大きくなりがちなので「軽量CLI」を優先するなら **reqwest直叩き**も選択肢

#### HTTP / OAuth / 秘密情報保管
- HTTP: `reqwest`
- OAuth/PKCE: `oauth2`
- Token/Secret保管: `keyring`（macOS Keychain / Windows / Linux Secret Service など）

#### CLI/出力
- CLI: `clap`
- JSON: `serde`, `serde_json`
- 端末表示（任意）: `colored` / `owo-colors`

#### ローカル callback server
- `axum`（推奨）/ `tiny_http`（軽量）

### 8.2 i18n（国際化）対応

#### 方針
- CLI のメッセージ（エラー、案内、確認文、ヘルプ補助など）を i18n 化する
- 初期対応言語: **英語（en）/ 日本語（ja）**
- 出力データ（JSON）は言語非依存（フィールド名は固定）。i18n は *人間向けテキスト*のみ。

#### 仕様
- 言語決定順（優先度順）
  1) `--lang <tag>`（例: `ja`, `en-US`）
  2) `SLACKRS_LANG` 環境変数
  3) OS ロケール（`LANG`, `LC_ALL` 等）
  4) default: `en`

#### 実装候補 crate
- **`i18n-embed`**（Fluent / gettext の両方式をサポート）
  - バイナリに翻訳資産を埋め込める
  - `cargo-i18n` と組み合わせ可能
- 併用候補:
  - `fluent-bundle`, `unic-langid`（Fluent採用時）

#### 翻訳資産
- `locales/en-US/...` `locales/ja-JP/...` のように配置
- 最低限、以下をキー化:
  - `auth.login.open_browser`
  - `auth.login.callback_wait`
  - `auth.status.ok`
  - `error.missing_scope`
  - `error.rate_limited`
  - `confirm.write_action`

### 8.3 内部構造
- `slack_api`：HTTP client、認証、レート制限（`slack-morphism`採用時は薄くラップ）
- `token_store`：Keychain/SecretService抽象
- `i18n`：LanguageLoader + メッセージ取得
- `commands`：auth/search/conv/api/msg...

---

## 9. 受け入れ条件（DoD）

- 2つ以上のワークスペースで profile を作って切り替えできる
- `api call search.messages` が動く（成功するWSで）
- `search` / `conv history` など最低限のラッパーが動く
- 429 を正しく扱う
- tokenが安全に保存され、マスクされる
- `--lang ja` / `--lang en` でCLIメッセージ言語が切り替わる（少なくとも主要メッセージ）
- write系コマンドが安全装置付きで動く（`SLACKRS_ENABLE_WRITE=1` + 必要に応じて `--yes`）

---

## 付録: 典型コマンド例

```bash
# OAuth（WSごと）
slackcli auth login --profile acme
slackcli auth login --profile partner

# 網羅：任意メソッド
slackcli --profile acme api call search.messages query="invoice in:#finance" sort=timestamp count=20
slackcli --profile acme api call conversations.history channel=C123 limit=50

# ラッパー
slackcli --profile acme search "invoice in:#finance" --limit 20
slackcli --profile partner conv history --channel C123 --limit 200
```
