# タスク

- [ ] OAuth 設定（client_id/secret、redirect_uri、scopes）を読み込む仕組みを実装する（検証: 未設定時にエラーになるユニットテスト）
- [ ] PKCE と state の生成を実装する（検証: 生成値が空でないこと、state が一致検証されることのテスト）
- [ ] localhost callback サーバを実装する（検証: callback に `code` と `state` を送ると受信できるテスト）
- [ ] `oauth.v2.access` 交換処理を実装し、base URL を差し替え可能にする（検証: モックサーバの応答で token を取得するテスト）
- [ ] `auth login` を CLI に接続する（検証: モック OAuth で profile と token が保存されることを確認）
- [ ] `auth status/list/rename/logout` を実装する（検証: profiles.json と token store が更新されるテスト）

## Future Work

- 実運用の Slack OAuth での疎通検証（外部アカウント/資格情報が必要）
