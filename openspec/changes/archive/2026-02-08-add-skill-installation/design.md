# 設計メモ: install-skill

## 目的
`agent-skills-rs` を採用し、`@skills/slack-rs` の内容を `skills/slack-rs` に配置した埋め込み資産として導入可能にします。
引数なしの `install-skill` は `self` を既定解決し、実行時に外部ネットワークへ依存しない導入フローに統一します。

## 方針理由
- スキルはコマンドバイナリに埋め込み、コマンドのバージョンに追従して配布する
- 実行時の外部取り込みを排除し、導入対象とコマンド本体の整合性を保つ
- 配布済みバージョンに対する導入結果の再現性を維持する

## アーキテクチャ概要
- 依存関係: `agent-skills-rs` を利用し、Discovery/Installer/Lock を活用する
- 埋め込み資産: `skills/slack-rs` 配下を `self` ソースとして扱う
- CLI: `install-skill` をトップレベルコマンドとして提供する
- 配置先: `~/.config/slack-rs/.agents/skills/<skill-name>`
- ロック: `~/.config/slack-rs/.agents/.skill-lock.json` を更新する

## CLI サーフェス
- `slack-rs install-skill` (引数なし): `self` を既定解決してインストール
- `slack-rs install-skill <source>`: 明示ソースでインストール
  - `self` (埋め込み)
  - `local:<path>`

## 既定ソース解決
- 引数省略時は必ず `self` を解決する
- `self` は `skills/slack-rs` の埋め込み資産を参照する
- 実行時の既定経路で外部 GitHub を解決しない

## 出力方針
- JSON を既定出力とし、`ok`/`type`/`schemaVersion` を含める
- インストール結果として `skills[].name`/`skills[].path`/`skills[].source_type` を返す
- エラーは標準エラーへ出力し、非ゼロ終了する

## テスト方針
- ネットワーク不要の検証を優先し、`self`/`local` の導入経路をテスト対象にする
- ロック更新と配置先パスを一時ディレクトリで再現可能にする

## 互換性
- 既存コマンドや既存設定ディレクトリの挙動は変更しない
- `commands --json` と `schema` のイントロスペクションに `install-skill` を追加する
