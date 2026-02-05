## MODIFIED Requirements

### Requirement: client_secret is stored in Keyring and not displayed

`client_secret` は設定ファイルに保存してはならず (MUST NOT)、token store backend に保存されなければならない (MUST)。

token store backend のデフォルトは Keyring でなければならない (MUST)。

`config oauth show` は backend に関わらず `client_secret` を出力してはならない (MUST NOT)。

#### Scenario: show は client_secret を表示しない
- Given `config oauth set` で `client_secret` を入力する
- When `config oauth show` を実行する
- Then 出力に `client_secret` の値が含まれない
- And `profiles.json` に `client_secret` が含まれない

#### Scenario: file mode でも client_secret は表示されない
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- And `client_secret` が file backend に保存されている
- When `config oauth show` を実行する
- Then 出力に `client_secret` の値が含まれない

## ADDED Requirements

### Requirement: Keyring unavailable must fail without silent fallback

`SLACKRS_TOKEN_STORE` による明示指定がない場合、Keyring が利用不能な環境で `config oauth set/show/delete` を実行してはならない (MUST NOT)。その場合は MUST で失敗し、対処方法（例: OS の Keyring をアンロックして再実行する、または `SLACKRS_TOKEN_STORE=file` を設定する）を提示しなければならない。

ここで「Keyring が利用不能」には、Keyring がロックされていてユーザー操作（対話的アンロック）を要求するケース（interaction required 等）も含む。slack-rs は Keyring backend のアンロックのために独自のパスワード/パスフレーズ入力プロンプトを実装してはならない (MUST NOT)。

#### Scenario: Keyring 利用不能時に set が失敗しガイダンスを出す
- Given `SLACKRS_TOKEN_STORE` が設定されていない
- And Keyring が利用不能である
- When `config oauth set` を実行する
- Then コマンドは失敗する
- And エラーに「OS の Keyring をアンロックして再実行」または `SLACKRS_TOKEN_STORE=file` を含む対処方法が含まれる
- And slack-rs は Keyring アンロックのための追加のパスワード入力を求めない
