# wrapper-commands 変更仕様（トークン種別選択）

## ADDED Requirements
### Requirement: ラッパーコマンドで `--token-type` を受け付ける
ラッパーコマンドは `--token-type user|bot` を受け付け、指定されたトークンを利用すること。 (MUST)
#### Scenario: conv list で bot を明示指定する
- Given `conv list --token-type bot` を実行する
- When API を呼び出す
- Then Bot Token が Authorization ヘッダに使われる

### Requirement: 既定トークン種別を利用する
`--token-type` 未指定の場合は、プロファイルの `default_token_type` に従うこと。 (MUST)
#### Scenario: 既定値が bot の場合
- Given `default_token_type` が `bot`
- When `msg post` を実行する
- Then Bot Token が使用される
