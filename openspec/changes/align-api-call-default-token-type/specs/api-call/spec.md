# api-call Delta Specification (default token type inference)

## MODIFIED Requirements
### Requirement: Resolve with default token type
`--token-type` が未指定の場合、プロフィールの `default_token_type` が設定されていればその値を使用しなければならない。(MUST)
`default_token_type` が未設定の場合は、トークンストアに User token が存在するなら User を既定として選択し、存在しない場合は Bot を既定として選択しなければならない。(MUST)

#### Scenario: default_token_type 未設定で User token が存在する
- Given `default_token_type` が未設定である
- And トークンストアに User token が保存されている
- When `api call` を実行する
- Then User Token が使用される
