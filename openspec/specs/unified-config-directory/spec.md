# unified-config-directory Specification

## Purpose
すべてのプラットフォームで `~/.config/slack-rs/` を統一的に使用する設定ディレクトリを定義する。

## ADDED Requirements

### Requirement: すべてのプラットフォームで ~/.config/slack-rs/ を使用する

設定ディレクトリは、すべてのプラットフォーム（macOS、Linux、Windows）で `~/.config/slack-rs/` を使用しなければならない (MUST)。

プラットフォーム固有のディレクトリ（macOS の `~/Library/Application Support/` など）は使用してはならない (MUST NOT)。

#### Scenario: macOS で ~/.config/slack-rs/ が使用される
- **WHEN** macOS で設定ディレクトリパスを取得する
- **THEN** `~/.config/slack-rs/` が返される

#### Scenario: Linux で ~/.config/slack-rs/ が使用される
- **WHEN** Linux で設定ディレクトリパスを取得する
- **THEN** `~/.config/slack-rs/` が返される

#### Scenario: Windows で ~/.config/slack-rs/ が使用される
- **WHEN** Windows で設定ディレクトリパスを取得する
- **THEN** `%USERPROFILE%/.config/slack-rs/` が返される

### Requirement: 設定ディレクトリが存在しない場合は自動作成される

設定ディレクトリ `~/.config/slack-rs/` が存在しない場合、自動的に作成されなければならない (MUST)。

ディレクトリ作成に失敗した場合は、エラーを返さなければならない (MUST)。

#### Scenario: 設定ディレクトリが自動作成される
- **WHEN** `~/.config/slack-rs/` が存在しない状態で設定ファイルパスを取得する
- **THEN** `~/.config/slack-rs/` ディレクトリが自動的に作成される

#### Scenario: ディレクトリ作成失敗時はエラーを返す
- **WHEN** ディレクトリの作成に失敗する（権限不足など）
- **THEN** エラーが返される

### Requirement: profiles.json は ~/.config/slack-rs/ に保存される

プロファイル設定ファイル `profiles.json` は `~/.config/slack-rs/profiles.json` に保存されなければならない (MUST)。

#### Scenario: profiles.json のパスが正しい
- **WHEN** デフォルトの設定ファイルパスを取得する
- **THEN** `~/.config/slack-rs/profiles.json` が返される

### Requirement: tokens.json は ~/.config/slack-rs/ に保存される

トークンファイル `tokens.json` は `~/.config/slack-rs/tokens.json` に保存されなければならない (MUST)。

#### Scenario: tokens.json のパスが正しい
- **WHEN** デフォルトのトークンファイルパスを取得する
- **THEN** `~/.config/slack-rs/tokens.json` が返される

### Requirement: HOME 環境変数が設定されていない場合はエラーを返す

`HOME` 環境変数が設定されていない場合、設定ディレクトリパスの取得はエラーを返さなければならない (MUST)。

#### Scenario: HOME 環境変数が未設定の場合はエラーを返す
- **WHEN** `HOME` 環境変数が設定されていない状態で設定ディレクトリパスを取得する
- **THEN** `ConfigDirNotFound` エラーが返される
