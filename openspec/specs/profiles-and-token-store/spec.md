# profiles-and-token-store Specification

## Purpose
TBD - created by archiving change establish-profile-storage. Update Purpose after archive.
## Requirements
### Requirement: プロファイル設定を永続化できる
プロファイルの非秘密情報は `profiles.json` に保存され、再起動後も同一の内容を取得できなければならない。(MUST)
#### Scenario: 新しいプロファイルを保存して再読み込みする
- Given 空の設定ファイルが存在する
- When プロファイル情報（profile_name, team_id, user_id, scopes）を保存する
- Then 再読み込み時に同一の値が取得できる

### Requirement: 設定ファイルにバージョンを持つ
`profiles.json` は `version` フィールドを持たなければならない。(MUST)
#### Scenario: 保存時に version が含まれる
- Given 新規作成の設定ファイルが存在する
- When 設定を保存する
- Then `version` が含まれる

### Requirement: profile_name は一意である
同一の `profile_name` は複数登録してはならない。(MUST NOT)
#### Scenario: 同名プロファイルを追加しようとする
- Given `profile_name` が既に存在する
- When 同じ `profile_name` を保存する
- Then 重複エラーになる

### Requirement: `(team_id, user_id)` は安定キーとして一意である
同じ `(team_id, user_id)` を持つプロファイルは重複してはならない。(MUST NOT)
#### Scenario: 同じ `(team_id, user_id)` を再登録する
- Given `(team_id, user_id)` が既に存在する
- When 同じ `(team_id, user_id)` を持つプロファイルを保存する
- Then 既存エントリが更新され、新規追加されない

### Requirement: トークンは keyring に保存し、設定ファイルに保存しない
トークンは OS keyring に保存し、`profiles.json` に保存してはならない。(MUST NOT)
#### Scenario: token を保存して設定ファイルを確認する
- Given token を保存した
- When `profiles.json` を読み込む
- Then token が含まれていない

### Requirement: keyring のキー形式は安定である
keyring の保存キーは `service=slackcli`, `username={team_id}:{user_id}` でなければならない。(MUST)
#### Scenario: keyring キーを生成する
- Given `team_id=T123` と `user_id=U456` がある
- When keyring の保存キーを生成する
- Then `slackcli` と `T123:U456` が使われる

### Requirement: profile_name から安定キーを解決できる
`profile_name` から `(team_id, user_id)` を一意に解決できなければならない。(MUST)
#### Scenario: profile_name から `(team_id, user_id)` を解決する
- Given 設定に profile_name が存在する
- When profile_name を指定する
- Then `(team_id, user_id)` が返る

