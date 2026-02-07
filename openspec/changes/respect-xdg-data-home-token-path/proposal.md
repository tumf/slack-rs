# 提案: FileTokenStore の保存先解決で XDG_DATA_HOME を尊重する

## 概要
`SLACKRS_TOKEN_STORE=file` 利用時、現在の `FileTokenStore::default_path()` は `XDG_DATA_HOME` を参照せず、常に `~/.local/share/slack-rs/tokens.json` を返している。これにより、隔離テストや CI で `XDG_DATA_HOME` を使った実行時に期待外れのパスが使われる。本提案では、トークン保存先のデフォルト解決に `XDG_DATA_HOME` を導入し、`auth status` 表示と実際の解決結果を一致させる。

## 背景
- Issue #20 で、`XDG_DATA_HOME` 設定時も `~/.local/share/slack-rs/tokens.json` が表示される事象が報告された
- 既存の優先ルールは `SLACK_RS_TOKENS_PATH` の明示オーバーライドのみで、XDG 準拠の期待が仕様に明記されていない
- テスト隔離やコンテナ実行で、データ領域を環境変数で切り替える運用と相性が悪い

## 目的
- `FileTokenStore` のデフォルトパス解決で `XDG_DATA_HOME` を尊重する
- `auth status` の `Token Store: file (...)` 表示を、実際の解決ロジックと一致させる
- 既存の `SLACK_RS_TOKENS_PATH` 最優先ルールを維持し、破壊的変更を避ける

## 非目的
- トークンの保存形式（JSON）やキー形式（`{team_id}:{user_id}`）の変更
- `profiles.json` の保存先解決ロジックの変更
- `SLACKRS_TOKEN_STORE` 自体の挙動変更

## 提案内容
- トークン保存先解決の優先順位を以下に明文化する
  - 1) `SLACK_RS_TOKENS_PATH`（明示上書き）
  - 2) `XDG_DATA_HOME/slack-rs/tokens.json`（`XDG_DATA_HOME` が有効値の場合）
  - 3) `~/.local/share/slack-rs/tokens.json`（フォールバック）
- `auth status` が表示する file backend の保存先は、上記解決結果をそのまま表示する
- 回帰テストで `XDG_DATA_HOME` の有無と `SLACK_RS_TOKENS_PATH` 優先を固定する

## 期待効果
- XDG 準拠のユーザー期待と CLI 挙動が一致する
- `mktemp` ベースの隔離テストで実ホームを汚さず検証しやすくなる
- CI/コンテナ環境でのパス解決の予測可能性が上がる

## 将来検討
- Issue #20 コメントで提案された追加改善（`auth import --dry-run`、`doctor` コマンド等）は本変更と独立したため、別 change として分離して扱う
