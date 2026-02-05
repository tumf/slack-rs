# api-call 変更仕様（トークン種別選択）

## MODIFIED Requirements
### Requirement: 出力メタ情報にトークン種別を含める
`api call` の出力 `meta` は `token_type` を含めること。 (MUST)
#### Scenario: token_type が出力される
- Given `api call` を実行する
- When 有効なトークンで API を呼び出す
- Then `meta.token_type` に `user` または `bot` が含まれる

## ADDED Requirements
### Requirement: `--token-type` でトークン種別を明示できる
`api call` は `--token-type user|bot` を受け付け、指定された種別のトークンを利用すること。 (MUST)
#### Scenario: user トークンを明示指定する
- Given `--token-type user` が指定されている
- When `api call conversations.list` を実行する
- Then User Token が Authorization ヘッダに使われる

### Requirement: 既定トークン種別での解決
`--token-type` 未指定の場合、プロファイルの `default_token_type` を利用すること。 (MUST)
#### Scenario: 既定値が user の場合
- Given プロファイルの `default_token_type` が `user`
- When `api call` を実行する
- Then User Token が使用される

### Requirement: トークンが存在しない場合のエラー
指定された種別のトークンが存在しない場合は、実行を失敗させて明確なエラーを返すこと。 (MUST)
#### Scenario: user トークンが保存されていない
- Given `--token-type user` が指定されている
- And User Token が保存されていない
- When `api call` を実行する
- Then トークン不足を示すエラーが返る
