# profiles-and-token-store 変更仕様（既定トークン種別）

## ADDED Requirements
### Requirement: 既定トークン種別をプロファイルに保存する
プロファイルは `default_token_type` を任意で保持し、永続化されること。 (MUST)
#### Scenario: 既定値の保存と再読み込み
- Given `default_token_type=user` を設定する
- When プロファイルを保存して再読み込みする
- Then `default_token_type` が保持されている
