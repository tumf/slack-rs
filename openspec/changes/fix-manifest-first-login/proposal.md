---
change_type: implementation
priority: high
dependencies: []
references:
  - src/cli/handlers.rs
  - src/auth/commands.rs
  - src/auth/manifest.rs
  - openspec/specs/oauth-login/spec.md
  - openspec/specs/oauth-manifest-generation/spec.md
---

# 変更提案: cloudflared/ngrok ログインを manifest-first 順序へ修正する

**Change Type**: implementation

## Problem / Context
- 現在の `auth login --cloudflared` / `--ngrok` 経路は、Slack App Manifest を生成してアプリ作成を案内する前に `Client ID` と `Client Secret` の入力を要求している。
- 実装上、`run_auth_login` が先に `Enter Slack Client ID:` と `Enter OAuth client secret:` を実行し、その後で `login_with_credentials_extended` 側で manifest を生成している。
- 一方で manifest 生成関数は `src/auth/manifest.rs` で `client_id` を実際には使用しておらず、redirect URI と scopes と profile 名だけで生成できる。
- このため、ユーザーが「manifest をもとに Slack App を作成し、その後に Client ID / Secret を取得する」運用と CLI の対話順序が不整合になっている。

## Proposed Solution
- `--cloudflared` / `--ngrok` の拡張ログイン経路を manifest-first に再構成する。
- 先に tunnel により redirect URI を確定し、bot/user scopes と profile 情報だけで manifest を生成・保存・案内する。
- ユーザーが Slack 側で app を作成したあとに、`Basic Information -> App Credentials` から Client ID / Secret を入力させる。
- 既存の標準ログイン経路や OAuth 実行コアの責務は維持しつつ、拡張経路だけ順序を修正する。
- 可能なら保存済み OAuth 設定を再利用し、未保存時のみ対話入力する方針を許容するが、少なくとも manifest 生成前に credentials を要求しないことを必須にする。

## Acceptance Criteria
- `auth login --cloudflared` / `--ngrok` では、manifest 生成前に Client ID / Secret の入力プロンプトを表示しない。
- tunnel 起動後に redirect URI を用いて manifest が生成・保存され、Slack App 作成の案内が表示される。
- ユーザーが app 作成後に Enter したあと、OAuth 開始直前のタイミングで Client ID / Secret を解決する。
- manifest 生成は Client ID 未入力でも成立し、外部 API に依存しない。
- README と CLI 案内は manifest-first の順序に一致する。

## Out of Scope
- Slack Web UI 側の自動操作。
- OAuth PKCE や token exchange のプロトコル変更。
- `auth login` 標準経路の大規模な責務再設計。
