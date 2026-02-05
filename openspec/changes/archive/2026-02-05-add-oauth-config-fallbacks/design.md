## 目的
keyring が使えない環境や非対話環境でも OAuth client secret を安全に設定できるようにする。

## 設計方針
- **優先順位を明確化**: 明示指定 > 環境変数 > ファイル > 対話入力の順で解決する。
- **安全性の確保**: `--client-secret` は危険性が高いため、明示同意が無い場合は拒否する。
- **保存先は維持**: `client_secret` の保存先は token store backend のままとし、`profiles.json` には保存しない。
- **非対話エラーを明示**: 入力経路が無い場合は失敗し、利用可能な手段を案内する。

## 入力ソースの優先順位
1. `--client-secret-env <VAR>` に指定された環境変数
2. `SLACKRS_CLIENT_SECRET`
3. `--client-secret-file <PATH>`
4. `--client-secret <SECRET>`（要 `--yes`）
5. 対話入力（上記が無い場合）

## 代替案
- keyring 不可時に自動で file backend に切り替える案は、既存の明示選択ルールに反するため採用しない。
