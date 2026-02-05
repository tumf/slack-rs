# api-call 変更仕様（private_channel 優先）

## MODIFIED Requirements
### Requirement: private_channel 指定時は User Token を優先する
`api call conversations.list` で `types` に `private_channel` が含まれる場合、`--token-type` 未指定時は User Token を優先すること。 (MUST)
#### Scenario: private_channel 指定で user を優先する
- Given `types=private_channel` を指定して `api call conversations.list` を実行する
- And `--token-type` は未指定
- And User Token が存在する
- When API を呼び出す
- Then User Token が使用される
