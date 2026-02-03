# 設計方針

## 全体構成
`slackcli auth export/import` を追加し、profile を暗号化ファイルとして保存・復元する。通常時は OS Keyring を使用し、export/import は明示的な危険操作とする。

## 主要コンポーネント
- **CLI 層**: `auth export` / `auth import` の引数・確認・エラーメッセージ
- **Storage 層**: keyring への保存・取得（profile name / team_id の対応）
- **Crypto 層**: Argon2id + AES-256-GCM による暗号化と復号
- **Format 層**: magic + version + KDF params + encrypted payload のバイナリ構造
- **i18n**: ja/en の警告・プロンプト・エラー

## データフォーマット
### ファイル構造
1. magic（固定バイト列）
2. format_version（u32）
3. KDF params（salt、memory/time/parallelism）
4. nonce + ciphertext（AES-256-GCM）

### 平文ペイロード
JSON 形式で `format_version` と `profiles` を持つ。unknown field は無視できるようにする。

## セキュリティ設計
- export は必ず暗号化し、平文出力を禁止
- passphrase は env または prompt から取得（空文字は禁止）
- 0600 でファイル作成し、既存ファイルの権限が不正ならエラー
- token などの機密情報はログに出さない

## 競合処理
- import 時に同じ `team_id` が存在する場合はデフォルトで失敗
- `--yes` + `--force` で上書き可能
- `--profile` はローカル表示名であり、実体の紐づきは `team_id`

## i18n 設計
- `warn.export_sensitive`, `prompt.passphrase`, `error.bad_permissions` を追加
- `--lang` で ja/en を切り替え

## テスト設計
- Crypto: 既知の passphrase + salt で round-trip の検証
- Format: version, magic, unknown field の読み込み
- CLI: `--yes`/`--force` の分岐、env/prompt の優先順位
- Storage: keyring の mock 実装で export/import を検証
