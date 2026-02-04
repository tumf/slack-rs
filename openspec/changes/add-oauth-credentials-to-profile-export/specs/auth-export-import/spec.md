## MODIFIED Requirements

### Requirement: Import writes back to keyring

インポート時に復元対象の機密情報はKeyringへ保存されることがMUST。

#### Scenario: Import decrypts and stores to keyring
- `slackcli auth import --in <path>` 実行時、OAuthクライアントシークレットが含まれていればKeyringへ保存される

## ADDED Requirements

### Requirement: Export/importにOAuthクレデンシャルを含める

エクスポート対象のプロファイルにOAuthクレデンシャルが存在する場合、暗号化ペイロードに含めることがMUST。

#### Scenario: Export includes OAuth credentials when available
- `client_id` が存在する場合、エクスポートペイロードに含まれる
- `client_secret` がKeyringに存在する場合、エクスポートペイロードに含まれる

### Requirement: OAuthクレデンシャルは設定ファイルに平文で保存しない

OAuthクライアントシークレットは設定ファイルに保存されないことがMUST。

#### Scenario: Import stores client_secret only in keyring
- import時に `client_secret` はKeyringへ保存され、設定ファイルには書き込まれない
