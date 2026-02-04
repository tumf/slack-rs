---
title: Slack Web API CLI（Rust）- 認証情報 Export/Import（Keyring互換）仕様
date: 2026-02-03
status: draft
tags:
  - slack
  - cli
  - rust
  - oauth
  - security
  - keyring
---

# Slack Web API CLI（Rust）- 認証情報 Export/Import（Keyring互換）仕様

## 目的
- `slackcli` の **profile（=ワークスペースごとのOAuth認証情報）** を、端末間で移行/バックアップできるようにする。
- 通常運用は OS の secure store（Keychain/SecretService等）に保存しつつ、
  **gog の keyring export/import に相当する“暗号化エクスポート”**を提供する。

## 前提・脅威モデル
- exportファイルは **漏洩すると即アウト**（Slackアカウント/WSへのアクセス権）
- 従って exportは
  - デフォルトで危険操作として扱う
  - 暗号化必須
  - ファイル権限/保管ルールを仕様として強制/警告

---

## 要件

### 機能要件
- profile単位の export/import
- 全profile一括 export/import
- exportは必ず **暗号化**（平文禁止）
- i18n（ja/en）対応：確認文・警告文・パスフレーズ入力

### 非機能要件
- exportファイルの互換性（将来拡張）
  - `format_version` を必ず持つ
  - unknown field は無視できる
- ログ/標準出力に token を出さない（debug含む）

---

## CLI コマンド仕様

### 1) Export

#### 単一profile
- `slackcli auth export --profile <name> --out <path>`

オプション:
- `--passphrase-env SLACKRS_KEYRING_PASSWORD`（既定）
- `--passphrase-prompt`（環境変数が無い時の対話入力）
- `--force`（既存ファイル上書き）
- `--yes`（危険操作の同意）

挙動:
- `--yes` が無い場合は中止（安全装置）
- `<path>` は `0600` を強制（作成時）
- 既存が 0600 以外なら警告/失敗（要検討）

#### 全profile
- `slackcli auth export --all --out <path>`

### 2) Import

#### 単一profile
- `slackcli auth import --profile <name> --in <path>`

オプション:
- `--passphrase-env SLACKRS_KEYRING_PASSWORD`（既定）
- `--passphrase-prompt`
- `--yes`（上書き等の同意）

挙動:
- importは OS keyring（`keyring` crate）へ書き戻す
- `--profile` は「ローカルの表示名」。中身の `team_id` と紐づく
- 既に同 `team_id` のprofileが存在する場合:
  - 既定: 失敗して、選択肢を表示（上書き/別名で追加）
  - `--yes` + `--force` で上書き可能（要検討）

#### 全profile
- `slackcli auth import --all --in <path>`

---

## データフォーマット

### 形式（推奨）
- `slackauth`（単一バイナリファイル）
- 内部は以下の構造:
  1) ヘッダ（マジック + バージョン）
  2) KDFパラメータ（salt, iterations/memory）
  3) 暗号化ペイロード（AES-256-GCM）

### 平文（暗号化前）ペイロードJSON（例）
```json
{
  "format_version": 1,
  "exported_at": "2026-02-03T08:58:00Z",
  "profiles": [
    {
      "profile_name": "acme",
      "team": {"id": "T123", "name": "Acme"},
      "token": {
        "access_token": "xoxp-...",
        "refresh_token": "...",
        "expires_at": "2026-03-03T...Z",
        "scopes": ["search:read", "chat:write"],
        "token_type": "user"
      }
    }
  ]
}
```

### 暗号化
- KDF: Argon2id 推奨（実装難なら PBKDF2 でも可だが優先度はArgon2id）
- 暗号: AES-256-GCM
- 失敗時エラー: 「パスフレーズが違う / ファイル破損」の区別は漏洩リスクとUXのバランスで決める（既定は曖昧で良い）

---

## i18n（ja/en）対象メッセージ例

- `warn.export_sensitive`
  - ja: "このファイルにはSlackの認証情報が含まれます。漏洩すると第三者があなたとしてSlackを操作できます。続行しますか？"
  - en: "This file contains Slack credentials. If leaked, someone can act as you on Slack. Continue?"

- `prompt.passphrase`
  - ja: "エクスポート/インポート用パスフレーズを入力してください"
  - en: "Enter passphrase for export/import"

- `error.bad_permissions`
  - ja: "出力ファイルの権限が安全ではありません（推奨: 0600）"
  - en: "Output file permissions are not secure (recommended: 0600)"

---

## 受け入れ条件（DoD）
- export/import が単一profile・全profileで動く
- exportが暗号化必須で、平文を吐かない
- パスフレーズ入力が env/prompt の両方に対応
- tokenがログに出ない
- `--lang` で警告/プロンプトが切り替わる
