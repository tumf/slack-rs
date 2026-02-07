## MODIFIED Requirements

### Requirement: Unified argument parsing preserves existing behavior
`auth export` と `auth import` は共有引数パーサを使う場合でも既存フラグ互換を維持しなければならない。(MUST)

`-h` および `--help` は unknown option として失敗してはならず、サブコマンド固有ヘルプを表示して終了コード 0 で終了しなければならない。(MUST)

#### Scenario: `auth export` / `auth import` のヘルプフラグが成功する
- **WHEN** `slack-rs auth export -h` または `slack-rs auth export --help` を実行する
- **THEN** export サブコマンドの usage/options が表示される
- **AND** 終了コードは 0 になる
- **WHEN** `slack-rs auth import -h` または `slack-rs auth import --help` を実行する
- **THEN** import サブコマンドの usage/options が表示される
- **AND** 終了コードは 0 になる
