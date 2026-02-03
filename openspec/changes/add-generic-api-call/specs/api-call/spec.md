# 汎用 API 呼び出し

## ADDED Requirements

### Requirement: 任意の Slack API メソッドを呼び出せる
Slack Web API の任意メソッドに対して `https://slack.com/api/{method}` でリクエストできなければならない。(MUST)
#### Scenario: `api call search.messages` を実行する
- Given 有効な profile と token が存在する
- When `api call search.messages` を実行する
- Then `https://slack.com/api/search.messages` にリクエストが送られる

### Requirement: 認証ヘッダを付与する
リクエストには `Authorization: Bearer <token>` を付与しなければならない。(MUST)
#### Scenario: Bearer トークンを付与する
- Given token が取得できる
- When `api call` を実行する
- Then Authorization ヘッダが含まれる

### Requirement: form と JSON を切り替えて送信できる
`key=value` は form-urlencoded、`--json` は JSON body で送信しなければならない。(MUST)
#### Scenario: `--json` 指定時に JSON body を送る
- Given `--json` が指定されている
- When `api call chat.postMessage` を実行する
- Then Content-Type が `application/json` で送信される

### Requirement: HTTP メソッドを切り替えられる
デフォルトは POST とし、`--get` 指定時は GET を使用しなければならない。(MUST)
#### Scenario: `--get` を指定する
- Given `--get` が指定されている
- When `api call users.info` を実行する
- Then GET リクエストが送信される

### Requirement: 429 を Retry-After に従って再試行する
429 応答時は `Retry-After` に従って待機し、上限回数まで再試行しなければならない。(MUST)
#### Scenario: 429 応答後に待機して再試行する
- Given 429 と Retry-After を返す
- When `api call` を実行する
- Then 指定秒数待機して再試行する

### Requirement: 出力に meta を含める
出力 JSON には `meta.profile_name`, `meta.team_id`, `meta.user_id`, `meta.method` を含めなければならない。(MUST)
#### Scenario: 実行コンテキストが JSON に含まれる
- Given profile が選択されている
- When `api call` を実行する
- Then meta に必須フィールドが含まれる

### Requirement: Slack API 応答をそのまま返す
Slack API のレスポンスは `response` に保持し、`ok=false` でも返さなければならない。(MUST)
#### Scenario: `ok=false` の応答を返す
- Given Slack API が `ok=false` を返す
- When `api call` を実行する
- Then `response.ok` が `false` のまま返る
