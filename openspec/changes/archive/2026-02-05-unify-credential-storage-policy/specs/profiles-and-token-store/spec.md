## MODIFIED Requirements

### Requirement: Tokens are saved in file-based storage and not in configuration file

トークン（bot/user）および OAuth `client_secret` は `profiles.json` ではなく、token store backend に保存されなければならない (MUST)。

デフォルトの token store backend は Keyring でなければならない (MUST)。

Keyring が利用不能な場合、`SLACKRS_TOKEN_STORE` による明示指定がない限り、関連コマンドは MUST で失敗し、対処方法を提示しなければならない。静かな file へのフォールバックはしてはならない (MUST NOT)。

ここで「Keyring が利用不能」には、初期化/アクセスの失敗に加えて、Keyring がロックされていてユーザー操作（対話的アンロック）を要求するケース（interaction required 等）も含む。slack-rs は Keyring backend のために独自のパスワード/パスフレーズ入力プロンプトを導入してはならない (MUST NOT)。そのようなエラーは Keyring 利用不能として扱い、OS の Keyring をアンロックして再実行するか、`SLACKRS_TOKEN_STORE=file` を設定して file backend を明示的に選択するよう案内しなければならない。

ファイルベースの token store は `SLACKRS_TOKEN_STORE=file` により明示的に選択された場合にのみ使用してよい (MAY)。

`SLACKRS_KEYRING_PASSWORD` は export/import の暗号化パスワードであり、OS の Keyring アンロック要求とは無関係である。

#### Scenario: 既定では Keyring が使用される
- Given `SLACKRS_TOKEN_STORE` が設定されていない
- And Keyring が利用可能である
- When トークンストレージを初期化する
- Then Keyring backend が選択される

#### Scenario: Keyring が利用不能な場合は失敗しガイダンスを出す
- Given `SLACKRS_TOKEN_STORE` が設定されていない
- And Keyring が利用不能である
- When 認証情報が必要なコマンドを実行する
- Then コマンドは失敗する
- And エラーに「OS の Keyring をアンロックして再実行」または `SLACKRS_TOKEN_STORE=file` を含む対処方法が含まれる
- And slack-rs は Keyring アンロックのための追加のパスワード入力を求めない

#### Scenario: file mode では FileTokenStore が使用される
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- When トークンストレージを初期化する
- Then `FileTokenStore` backend が選択される

## ADDED Requirements

### Requirement: FileTokenStore mode reuses tokens.json path and stable keys

file mode（`SLACKRS_TOKEN_STORE=file`）では、既存の `FileTokenStore` の保存パス `~/.config/slack-rs/tokens.json` を再利用しなければならない (MUST)。

file mode ではキー形式も既存仕様を維持しなければならない (MUST)。少なくとも以下のキーは同一形式であること:
- bot token: `{team_id}:{user_id}`
- OAuth `client_secret`: `oauth-client-secret:{profile_name}`

#### Scenario: file mode で tokens.json と既存キー形式を使用する
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- When `team_id=T123` と `user_id=U456` の bot token を保存する
- Then `~/.config/slack-rs/tokens.json` にキー `T123:U456` で保存される
- And `profile_name=default` の `client_secret` はキー `oauth-client-secret:default` で保存される
