# doctor-diagnostics Specification

## Purpose
TBD - created by archiving change add-doctor-diagnostics-command. Update Purpose after archive.
## Requirements
### Requirement: `doctor` は認証・環境診断情報を一括表示する
CLI は `doctor` コマンドを提供し、認証と環境解決の切り分けに必要な情報を一括で表示しなければならない。(MUST)

`doctor` は少なくとも解決済み `profiles.json` パス、token store backend/path、bot/user token の存在有無を表示しなければならない。(MUST)

トークン値そのものは表示してはならない。(MUST NOT)

#### Scenario: `doctor --json` で機械可読な診断情報を取得できる
- Given 設定ファイルと token store が解決可能な環境がある
- When `slack-rs doctor --json` を実行する
- Then `configPath` と `tokenStore` と `tokens` を含む JSON が返る
- And token の生値は出力に含まれない

