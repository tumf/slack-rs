# 設計: 認証情報ストレージバックエンドの解決

## 目的
トークンおよび OAuth `client_secret` の保存先（token store backend）を統一し、デフォルトを Keyring にする。Keyring が利用不能な場合は静かに別バックエンドへ切り替えず、コマンドを失敗させて対処方法を示す。

## 用語
- 認証情報（credentials）: アクセストークン（bot/user）および OAuth `client_secret`
- token store backend: 認証情報を永続化するバックエンド（Keyring / file）
- file mode: `SLACKRS_TOKEN_STORE=file` によりファイルベース保存を明示的に選択するモード
- `SLACKRS_KEYRING_PASSWORD`: export/import の暗号化パスワード。OS の Keyring アンロック要求とは無関係

## バックエンド解決ルール
### 1. 環境変数による明示指定
- 環境変数 `SLACKRS_TOKEN_STORE` を参照する
- 許可する値:
  - `keyring`
  - `file`
- 未設定の場合は `keyring` をデフォルトとする
- 未知の値の場合は MUST で失敗し、許可値を提示する

### 2. Keyring の初期化と失敗時挙動
- `keyring` 選択時:
  - Keyring が利用できる場合は Keyring を使用する
  - slack-rs は Keyring backend のために独自のパスワード/パスフレーズ入力プロンプトを実装してはならない (MUST NOT)
  - Keyring が利用不能な場合（初期化/アクセスに失敗、または対話的アンロックが必要な状態）は MUST でコマンドを失敗させる
    - 例: interaction required / user action required / keyring is locked 等の「ユーザー操作が必要」「ロックされている」ことを示すエラー
  - リトライや繰り返しプロンプトで回避しようとしてはならない (MUST NOT)
  - その際のガイダンスは以下を含むこと:
    - 何が失敗したか（例: Keyring backend unavailable / locked / interaction required）
    - 対処方法（例: OS のキーチェーン/Secret Service をアンロックして再実行、または `SLACKRS_TOKEN_STORE=file` を設定）
  - 静かな file へのフォールバックは MUST NOT

### 3. file mode の挙動
- `file` 選択時は `FileTokenStore` を使用する
- file mode では既存の仕様を維持する:
  - パス: `~/.config/slack-rs/tokens.json`（既存の上書き可能なパス指定も維持）
  - キー: 既存のキー形式をそのまま利用する（例: `{team_id}:{user_id}`、`oauth-client-secret:{profile_name}`）

## 保存対象と保存先
- `profiles.json`:
  - 非機密情報のみ（例: `client_id`, `redirect_uri`, scopes 等）
  - トークンおよび `client_secret` は MUST NOT
- token store backend（Keyring または file）:
  - アクセストークン（bot/user）
  - OAuth `client_secret`

## `config oauth show` の出力ポリシー
- `config oauth show` は `client_secret` を MUST NOT で出力しない（バックエンド非依存）
- 代替として、存在確認のために以下のような非秘匿情報の出力は許可する（MAY）:
  - `client_secret_set: true|false`

## セキュリティノート
- デフォルトを Keyring にすることで、平文ファイルへの保存を避ける
- file mode は互換性/移行のための明示的オプトインとし、セキュリティ上の妥協であることを前提にする
- いかなる出力（stdout/stderr/log）でも `client_secret` を表示しない
- エラーガイダンスは秘匿値を含めず、次のアクション（環境変数設定/依存の有効化/OS Keyring のアンロック）を示す
- `SLACKRS_KEYRING_PASSWORD` は export/import の暗号化のみに使用され、OS の Keyring アンロック要求とは無関係である
