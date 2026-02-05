## 目的
`auth status` の表示から、参照している token store backend と実際の保存先を理解できるようにする。

## 設計方針
- **backend を明示する**: `keyring`/`file` と保存先（file の場合はパス）を必ず出力する。
- **フォールバックしない**: `auth status` 自体は backend を切り替えない。あくまで状態表示に留める。
- **誤解を減らす案内**: keyring を参照していて token が見つからないが file backend に token がある場合は案内を出す。
- **秘密情報を出さない**: `SLACK_TOKEN` の値は出力せず、設定有無のみ表示する。

## 具体的な挙動
- `auth status` は token store backend と保存先を出力する。
- backend が keyring のときのみ、file backend の tokens.json を軽量チェックし、該当キーが存在する場合に案内を出す。
- `SLACK_TOKEN` が設定されている場合は `SLACK_TOKEN: set` を表示する。

## 代替案
- token store を自動的に file へフォールバックする案は、明示的な backend 選択という既存仕様に反するため採用しない。
