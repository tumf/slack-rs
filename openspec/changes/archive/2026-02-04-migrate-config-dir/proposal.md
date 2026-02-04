## Why

現状のプロファイル設定ファイルは `~/.config/slack-cli/` 配下にあり、バイナリ名 `slack-rs` と一致していません。利用者の混乱を避け、設定の所在を明確にするため、設定ディレクトリ名を `slack-rs` に変更します。

## What Changes

- プロファイル設定ファイルの既定パスを `~/.config/slack-rs/profiles.json`（Linux/macOS）に変更する
- 旧パス `~/.config/slack-cli/profiles.json` が存在し、新パスが未作成の場合は自動移行する
- ドキュメントの設定パス記述を更新する

## Capabilities

### New Capabilities
- (なし)

### Modified Capabilities
- `profiles-and-token-store`: プロファイル設定ファイルの既定パスと旧パスからの移行挙動を更新する

## Impact

- `src/profile/storage.rs`（既定パス算出、移行処理）
- 既定パス利用箇所（CLI/auth/profile の各呼び出し）
- `README.md` の設定パス記述
