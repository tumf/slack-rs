## MODIFIED Requirements

### Requirement: Tokens are saved in file-based storage and not in configuration file

トークン（bot/user）および OAuth `client_secret` は `profiles.json` ではなく、FileTokenStore に保存されなければならない (MUST)。

FileTokenStore のデフォルト保存先は `~/.local/share/slack-rs/tokens.json` でなければならない (MUST)。

`profiles.json` と認証情報ファイル `tokens.json` は同一ファイルに保存してはならない (MUST NOT)。

#### Scenario: 認証情報は設定ファイルと分離して保存される
- Given `profiles.json` が設定ディレクトリに存在する
- When bot token と OAuth `client_secret` を保存する
- Then 認証情報は `~/.local/share/slack-rs/tokens.json` に保存される
- And `profiles.json` には認証情報が保存されない
