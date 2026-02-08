# 実装タスク

- [ ] `auth::login` と拡張ログイン経路で重複する OAuth 実行手順を単一コア関数へ統合する（検証: 両経路が同じコア呼び出しを利用するコード経路を確認）。
- [ ] コア関数の入出力を既存保存処理（profile/token/client secret）に接続し、呼び出し側は前後処理に限定する（検証: `cargo test --test auth_integration` が成功）。
- [ ] OAuth 通信部分は mock/stub で検証可能な単位に分け、外部クレデンシャル不要でテスト可能にする（検証: fixture ベーステストが `cargo test --lib` で成功）。
- [ ] 標準ログイン・拡張ログインの回帰を確認する（検証: `cargo test --test oauth_integration` と `cargo test --test manifest_generation_integration` が成功）。
