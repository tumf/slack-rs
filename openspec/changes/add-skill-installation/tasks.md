1. `agent-skills-rs` を依存追加し、スキル用モジュール骨格を作成する
   検証: `Cargo.toml` に依存が追加され、`src/lib.rs` に新モジュールが公開されている
2. 設定ディレクトリ配下にスキル用パスを定義する
   検証: `~/.config/slack-rs/.agents/skills` と `~/.config/slack-rs/.agents/.skill-lock.json` の解決ロジックが実装されている
3. `install-skill` コマンドを CLI に追加しエントリポイントへ配線する
   検証: `slack-rs --help` と `slack-rs commands --json` に `install-skill` が表示される
4. `agent-skills-rs` の Discovery/Installer/Lock を使ったインストール処理を実装する
   検証: 一時ディレクトリを用いたユニットテストで埋め込み/ローカルのインストールとロック更新が確認できる
5. GitHub 経路の discovery をモック化し、ネットワーク不要で検証できるようにする
   検証: モック利用のテストが追加され、CI 環境で `cargo test` が成功する
6. `schema --command install-skill --output json-schema` の出力を追加する
   検証: 生成スキーマに `ok`/`type`/`schemaVersion` とインストール結果フィールドが含まれる
