# 設計メモ: install-skill

## 目的
`agent-skills-rs` を採用して `install-skill` のインストール経路を最小実装で提供する。
外部ネットワークへの依存はモックで代替し、CI で検証可能な範囲に限定する。

## アーキテクチャ概要
- 依存関係: `agent-skills-rs` を追加し、Discovery/Installer/Lock を活用する
- CLI: `install-skill` をトップレベルコマンドとして追加
- 配置先: `~/.config/slack-rs/.agents/skills/<skill-name>` を正規配置先とする
- ロック: `~/.config/slack-rs/.agents/.skill-lock.json` を更新する

## CLI サーフェス
- `slack-rs install-skill` (引数なし): 埋め込みスキルをインストール
- `slack-rs install-skill <source>`: ソース指定でインストール
  - `self` (埋め込み)
  - `local:<path>`
  - `github:<owner>/<repo>[#ref][:subpath]`

## 出力方針
- JSON をデフォルト出力とし、`ok`/`type`/`schemaVersion` を含む
- インストールされたスキル名・パス・ソース種別を返す
- エラーは標準エラーへ簡潔に出力し、非ゼロ終了

## モック方針 (外部依存対策)
- GitHub 取得は `agent-skills-rs` の抽象化を使ってモック/スタブを注入
- ネットワーク不要のテストとして、埋め込み/ローカル経路を優先して検証
- GitHub 経路は疑似レスポンスで discovery/installer の流れを検証

## 互換性
- 既存コマンドの挙動や設定ディレクトリは変更しない
- `commands --json` と `schema` の出力に `install-skill` を追加

## 代替案
- `skills install` のサブコマンド化
  - 既存 CLI の構造に合わせやすいが、ユーザー要望の `install-skill` から外れる
