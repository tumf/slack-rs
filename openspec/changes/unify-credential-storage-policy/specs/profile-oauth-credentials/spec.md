## MODIFIED Requirements

### Requirement: OAuth credentials retrieval uses token store backend and prompts only for missing values

login 開始時のクライアント情報は、プロファイルに保存された設定（非機密）および token store backend（機密）を優先して利用し、不足している項目のみ対話入力で補完しなければならない (MUST)。

ここで token store backend のデフォルトは Keyring でなければならない (MUST)。

対話入力は OAuth の不足値（例: `client_secret` が未保存）を補うためのものに限定される。Keyring backend のアンロックのために独自のパスワード/パスフレーズ入力プロンプトを導入してはならない (MUST NOT)。Keyring がロックされていてユーザー操作（interaction required 等）が必要な場合は Keyring を利用不能として扱い、OS の Keyring をアンロックするか `SLACKRS_TOKEN_STORE=file` を設定するよう案内して失敗しなければならない。

#### Scenario: client_secret が token store backend にない場合のみプロンプトされる
- Given `client_id` はプロファイルに保存されている
- And `client_secret` が token store backend に存在しない
- When `auth login` を実行する
- Then `client_secret` の入力が求められる
- And `client_id` の入力は求められない

### Requirement: Store client_id in profile and client_secret in token store backend

各プロファイルは `client_id` を `profiles.json` に保存しなければならない (MUST)。

`client_secret` は token store backend に保存されなければならず (MUST)、設定ファイルに残してはならない (MUST NOT)。

#### Scenario: ログイン成功後に client_id は profiles.json に、client_secret は token store backend に保存される
- When `auth login` が成功する
- Then `client_id` が `profiles.json` に保存される
- And `client_secret` が token store backend に保存される
- And `profiles.json` に `client_secret` が含まれない

#### Scenario: file mode では client_secret は既存の file backend キー形式で保存される
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- When `auth login` が成功する
- Then `client_secret` は file backend のキー `oauth-client-secret:{profile_name}` で保存される
