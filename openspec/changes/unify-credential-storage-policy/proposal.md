# 提案: 認証情報ストレージポリシーの統一

## 概要
既存の仕様で、トークンおよび OAuth `client_secret` の保存先が「ファイルがデフォルト」(profiles-and-token-store) と「Keyring がデフォルト」(oauth-config-management / profile-oauth-credentials) で分かれており、セキュリティ/運用の前提が一致していない。本変更では、認証情報（トークンと `client_secret`）の保存ポリシーを統一し、デフォルトは Keyring、ファイル保存は明示オプトインにする。

## 背景
- OS の Keyring は一般に最も安全な保存先であり、デフォルトとして妥当
- 一方で、Keyring が利用できない環境（最小コンテナ、CI、一部の Linux 環境など）もあり得る
- 現状は仕様間で前提が異なるため、実装・テスト・ユーザー体験が一貫しない

## 目的
- トークンと `client_secret` の保存先ポリシーを仕様レベルで統一する
- Keyring が利用できない場合に「静かなフォールバック」を禁止し、問題を早期に発見できるようにする
- 互換性のために、既存の `tokens.json` を利用するファイルベースの保存を明示的に選択できるようにする
- `config oauth show` で `client_secret` が漏洩しないことをバックエンドに依らず保証する

## 非目的
- 新しい暗号化方式や独自キーストア形式の導入
- 既存の `tokens.json` のキー/パスの変更（ファイルモードでは現状を踏襲する）
- 既存のログ/出力フォーマット全般の刷新（本提案は秘匿値の非表示に限定）

## 提案ポリシー
- デフォルトの token store backend は Keyring とする（最も安全）
- Keyring が利用不能な場合、関連コマンドは MUST で失敗し、対処方法を提示する（無言のファイルフォールバックは禁止）
- ファイルベースの token store は、環境変数 `SLACKRS_TOKEN_STORE=file`（または同等の明示設定）でのみ有効化できる
- ファイルモードでは既存の `~/.config/slack-rs/tokens.json` のパスおよび既存キー形式を再利用する
- `config oauth show` はバックエンドに関わらず `client_secret` を MUST NOT で出力しない

## 互換性と移行
- 既存のファイル保存トークンは、`SLACKRS_TOKEN_STORE=file` を設定することで引き続き利用できる
- デフォルトが Keyring になるため、Keyring 非対応環境では明示設定が必要になる（失敗時にガイダンスを出す）

## リスク
- Keyring 非対応環境での初期導入時にコマンド失敗が増える可能性
- ファイルモードの利用が増えると、運用上の秘匿値漏洩リスクが高まる（ただし明示オプトインに限定する）

## 受け入れ条件
- 仕様間で、トークンおよび `client_secret` の保存先が同一ポリシーに整合している
- Keyring 利用不能時、デフォルト設定では必ず失敗し、`SLACKRS_TOKEN_STORE=file` 等の次アクションが提示される
- ファイルモードでは既存 `tokens.json` のパスとキー形式が維持される
- `config oauth show` はどのバックエンドでも `client_secret` を表示しない
