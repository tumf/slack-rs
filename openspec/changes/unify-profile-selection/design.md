# Design: Unify profile selection

## 方針
- すべてのコマンドで同一の profile 解決関数を使う。
- 引数中の `--profile` をグローバルフラグとして扱い、サブコマンドより前でも後でも解決する。
- `--profile` が未指定の場合のみ `SLACK_PROFILE` を参照し、それも無ければ `default` とする。

## 実装アプローチ
1. **引数正規化**
   - ルーティング前に `--profile` を検出し、`--profile <name>` / `--profile=<name>` の両方を解釈する。
   - `--profile` はどの位置でも有効であることを前提に、検索は全引数を対象にする。

2. **共通ヘルパー**
   - `resolve_profile_name(args)` のような共通関数を用意し、
     `--profile` > `SLACK_PROFILE` > `default` の優先順位を返す。

3. **適用箇所**
   - `api call` の profile 解決を共通ヘルパーに移行する。
   - wrapper コマンド群の `--profile` 解釈にも共通ヘルパーを適用する。

## 互換性
- 既存の `--profile` 指定は維持される。
- `SLACK_PROFILE` 依存のフローは、`--profile` 指定が無い場合に限り有効となる。
