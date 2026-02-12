- [ ] `ExportProfile` 構造体に `user_token` フィールドを追加する
  - ファイル: `src/auth/format.rs`
  - 変更箇所: `ExportProfile` struct (39-52行目付近)
  - 実装内容:
    ```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_token: Option<String>,
    ```
  - 検証: `cargo build` が成功し、`ExportProfile` に `user_token` フィールドが存在することを確認

- [ ] `export_profiles` 関数で user token も取得するよう修正する
  - ファイル: `src/auth/export_import.rs`
  - 変更箇所: `export_profiles` 関数の168-200行目付近（token取得ロジック）
  - 実装内容:
    - bot token key: `make_token_key(&profile.team_id, &profile.user_id)`
    - user token key: `format!("{}:{}:user", &profile.team_id, &profile.user_id)`
    - どちらかのトークンが存在すれば `ExportProfile` を作成
    - 両方存在しない場合のみスキップ/エラー
  - 検証: `cargo build` が成功すること

- [ ] `ExportProfile` 作成時に user_token を設定する
  - ファイル: `src/auth/export_import.rs`
  - 変更箇所: `export_profiles` 関数内の `ExportProfile` 生成部分（177-188行目付近）
  - 実装内容: `ExportProfile { ..., user_token, ... }` に user token を渡す
  - 検証: `cargo build` が成功すること

- [ ] `import_profiles` 関数で user_token を復元するよう修正する
  - ファイル: `src/auth/export_import.rs`
  - 変更箇所: `import_profiles` 関数内の token 保存ロジック（369-393行目付近）
  - 実装内容:
    ```rust
    // Store bot token
    let bot_token_key = make_token_key(&export_profile.team_id, &export_profile.user_id);
    token_store.set(&bot_token_key, &export_profile.token)?;

    // Store user token if present
    if let Some(user_token) = &export_profile.user_token {
        let user_token_key = format!("{}:{}:user", &export_profile.team_id, &export_profile.user_id);
        token_store.set(&user_token_key, user_token)?;
    }
    ```
  - 検証: `cargo build` が成功すること

- [ ] export/import の往復テストを実装する
  - ファイル: `src/auth/export_import.rs`
  - 追加箇所: テストモジュール末尾（920行目以降）
  - テスト名: `test_export_import_with_user_token`
  - 実装内容:
    - bot token と user token を両方保存したプロファイルを作成
    - export を実行
    - import を実行
    - bot token と user token が正しいキーで復元されていることを確認
  - 検証: `cargo test test_export_import_with_user_token` が成功すること

- [ ] user token のみ存在する場合のテストを実装する
  - ファイル: `src/auth/export_import.rs`
  - テスト名: `test_export_user_token_only`
  - 実装内容:
    - user token のみ保存したプロファイルを作成
    - export が成功すること
    - import 後に user token が復元されていることを確認
  - 検証: `cargo test test_export_user_token_only` が成功すること

- [ ] 既存テストがすべて通ることを確認する
  - コマンド: `cargo test export_import`
  - 検証: すべてのテストが PASS すること

- [ ] フォーマットとlintを実行する
  - コマンド: `cargo fmt && cargo clippy`
  - 検証: エラーや警告が出ないこと
