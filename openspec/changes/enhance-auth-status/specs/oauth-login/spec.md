# oauth-login 変更仕様（auth status 拡張）

## MODIFIED Requirements
### Requirement: auth status はトークン情報を表示する
`auth status` は利用可能なトークン種別、各スコープ、既定トークン種別を表示すること。 (MUST)
#### Scenario: User/Bot 両方が存在する場合
- Given User Token と Bot Token が保存されている
- When `auth status` を実行する
- Then 両方のトークン種別とスコープが表示される

### Requirement: Bot Token が存在する場合は Bot ID を表示する
Bot Token が保存されている場合、`auth status` は Bot ID を表示すること。 (MUST)
#### Scenario: Bot Token の Bot ID が表示される
- Given Bot Token と Bot ID が保存されている
- When `auth status` を実行する
- Then Bot ID が表示される
