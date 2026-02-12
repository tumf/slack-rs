# auth-export-import-user-token Specification (Delta)

## MODIFIED Requirements

### Requirement: Export/import は bot/user トークンを両方扱う
`auth export` は bot token だけでなく user token も暗号化ペイロードに含めなければならない。(MUST)
`auth import` は user token が含まれている場合、専用キーへ復元しなければならない。(MUST)

#### Scenario: Export で user token が含まれる
- **GIVEN** プロファイルに bot token と user token が保存されている
- **WHEN** `slack-rs auth export --profile <name> --out <path>` を実行する
- **THEN** 暗号化ペイロードに user token が含まれる

#### Scenario: Import で user token が復元される
- **GIVEN** user token を含む export ファイルがある
- **WHEN** `slack-rs auth import --in <path> --yes` を実行する
- **THEN** user token は `team_id:user_id:user` のキーへ保存される
