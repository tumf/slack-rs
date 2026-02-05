## 変更概要
Issue #15 に対応し、`auth status` が参照しているトークンストアの backend と保存先を明示し、file backend にトークンが存在するのに keyring を見ている場合の案内を追加する。

## 背景
- `tokens.json` にトークンが存在しても、backend が keyring の場合 `auth status` が `Tokens Available: None` を出すケースがある。
- 現状の出力だけでは「トークンが無い」のか「別 backend を参照している」のかが判断できない。

## スコープ
- `auth status` 出力に token store backend と保存先を表示する。
- backend が keyring のときに file backend に該当トークンが存在する場合、`SLACKRS_TOKEN_STORE=file` の案内を表示する。
- `SLACK_TOKEN` が設定されている場合、値を出さずに設定済みであることを表示する。

## スコープ外
- backend の自動フォールバック（keyring から file への自動切替）。
- `auth status` 以外のコマンドのトークン解決ルール変更。

## 既知のリスク
- 出力項目が増えるため、既存のスクリプトで固定行数を前提にしている場合に影響が出る可能性がある。
- file backend の存在確認は追加の I/O を伴う。

## 受け入れ基準
- `auth status` が token store backend と保存先を表示する。
- keyring backend で token が見つからず、file backend に token が存在する場合、`SLACKRS_TOKEN_STORE=file` の案内が表示される。
- `SLACK_TOKEN` が設定されている場合、値を出さずに設定済みであることが表示される。
