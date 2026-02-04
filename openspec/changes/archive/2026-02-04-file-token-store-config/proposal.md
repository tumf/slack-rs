## Why

macOS Keyring を使用したトークンストレージでは、トークンアクセス時に毎回パスワードプロンプトが表示され、ユーザー体験が著しく低下していた。また、プラットフォームごとに異なる設定ディレクトリ（macOS: `~/Library/Application Support/`、Linux: `~/.config/`）を使用していたため、クロスプラットフォームでの一貫性が欠けていた。

## What Changes

- **BREAKING**: KeyringTokenStore から FileTokenStore へのデフォルト変更
  - トークンを `~/.config/slack-rs/tokens.json` にファイルベースで保存
  - ファイルパーミッションを 0600 に設定してセキュリティを確保
  - パスワードプロンプトの完全排除

- 設定ディレクトリの統一
  - すべてのプラットフォームで `~/.config/slack-rs/` を使用
  - `~/Library/Application Support/slack-rs/` からの移行パスを提供

- KeyringTokenStore の保持
  - 既存実装は残し、ユーザーが選択可能に
  - デフォルトは FileTokenStore

## Capabilities

### New Capabilities
- `file-token-storage`: ファイルベースのトークンストレージ実装（`~/.config/slack-rs/tokens.json`、0600 パーミッション）
- `unified-config-directory`: すべてのプラットフォームで `~/.config/slack-rs/` を使用する統一設定ディレクトリ

### Modified Capabilities
- `profiles-and-token-store`: トークンストレージのデフォルトを Keyring からファイルベースに変更

## Impact

**Affected Code:**
- `src/profile/token_store.rs`: FileTokenStore 実装の追加
- `src/profile/storage.rs`: default_config_path() の変更
- `src/auth/commands.rs`: KeyringTokenStore から FileTokenStore への置き換え
- `src/cli/mod.rs`: 同上

**Breaking Changes:**
- 既存の Keyring に保存されたトークンは自動移行されない
- ユーザーは再ログインが必要

**Benefits:**
- パスワードプロンプトの排除
- クロスプラットフォーム一貫性の向上
- バックアップ・リストアの簡易化
- 設定ファイルの可視性向上
