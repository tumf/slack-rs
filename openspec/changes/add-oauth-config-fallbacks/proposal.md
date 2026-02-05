## 変更概要
Issue #3 に対応し、`config oauth set` が keyring 非対応環境でも OAuth client secret を設定できるように、環境変数/ファイル/明示フラグの入力経路を追加する。

## 背景
- `config oauth set` は対話入力に依存しており、keyring が使えない環境や非対話環境では設定が進められない。
- file backend が存在しても、client secret の入力経路が無いため事実上ブロックされる。

## スコープ
- `config oauth set` に client secret の入力ソースを追加する。
- 非対話環境での挙動を明確化し、ガイダンスを出す。
- `client_secret` は引き続き token store backend に保存し、`profiles.json` には保存しない。

## スコープ外
- OAuth フローそのものの変更。
- keyring の自動フォールバック。

## 既知のリスク
- `--client-secret` はシェル履歴/プロセス一覧に残るため安全性が低い。
- 入力経路が増えることでヘルプが長くなる。

## 受け入れ基準
- `config oauth set` で `--client-secret-env`/`--client-secret-file`/`--client-secret` が利用できる。
- `--client-secret` は明示的な同意（`--yes` など）無しには受け付けない。
- `client_secret` は token store backend に保存され、`profiles.json` には保存されない。
