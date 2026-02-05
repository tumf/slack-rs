## 変更概要
Issue #13 と #14 に対応し、ラッパーコマンドで `SLACK_TOKEN` を確実に使用できるようにし、`auth status` がプロフィールの `default_token_type` を正しく表示するように整合させる。

## 背景
- `conv` などのラッパーコマンドが `SLACK_TOKEN` を参照せず、`api call` との挙動が不一致になっている。
- `config set --token-type` の設定が `auth status` 表示に反映されず、既定トークン種別が誤解される。

## スコープ
- ラッパーコマンドのトークン解決に `SLACK_TOKEN` を組み込む。
- `auth status` における `default_token_type` 表示の決定ロジックを明確化する。
- 既存の `--token-type` 指定と `default_token_type` の優先順位は維持する。

## スコープ外
- OAuth 設定の keyring フォールバック（Issue #3）。
- 新規コマンドや出力フォーマットの追加。

## 既知のリスク
- `SLACK_TOKEN` が設定されている環境では、トークンストアを使わない実行になるため、意図しないトークンを使う可能性がある。
- 既定トークン種別の表示ルール変更により、これまでの表示と異なる結果になる。

## 受け入れ基準
- `SLACK_TOKEN` が設定されている場合、`conv list`/`conv history`/`users info`/`msg`/`react`/`file` などのラッパーコマンドがそのトークンで実行される。
- `auth status` で `default_token_type` がプロフィール設定に従って表示される。
