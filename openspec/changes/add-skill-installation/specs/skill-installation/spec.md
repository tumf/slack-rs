# skill-installation Specification

## Purpose
`agent-skills-rs` を利用して `install-skill` コマンドでスキルを導入できるようにする。

## ADDED Requirements

### Requirement: install-skill は埋め込みスキルを既定でインストールする
`slack-rs install-skill` が引数なしで実行された場合、埋め込みスキルを検出してインストールしなければならない。(MUST)
インストール先は `~/.config/slack-rs/.agents/skills/<skill-name>` の正規パスでなければならない。(MUST)
`~/.config/slack-rs/.agents/.skill-lock.json` にロック情報を更新しなければならない。(MUST)

#### Scenario: 引数なしで埋め込みスキルを導入する
- Given `install-skill` を引数なしで実行する
- When 埋め込みスキルが検出される
- Then `~/.config/slack-rs/.agents/skills/<skill-name>` にスキルが配置される
- And ロックファイルが更新される

### Requirement: install-skill は source 文字列を受け付ける
`install-skill <source>` は以下の source 形式を受け付けなければならない。(MUST)
- `self`
- `local:<path>`
- `github:<owner>/<repo>[#ref][:subpath]`
未知のスキームはエラーにしなければならない。(MUST)

#### Scenario: local ソースでスキルを導入する
- Given `local:/path/to/skills` に `SKILL.md` が存在する
- When `slack-rs install-skill local:/path/to/skills` を実行する
- Then 指定パスからスキルを検出してインストールする

#### Scenario: 未知スキームはエラーになる
- Given `slack-rs install-skill foo:bar` を実行する
- When ソース解釈に失敗する
- Then 許容されるスキーム一覧を含むエラーが表示される

### Requirement: インストールは symlink を優先し失敗時に copy へフォールバックする
`install-skill` は symlink を優先し、作成に失敗した場合は copy にフォールバックしなければならない。(MUST)

#### Scenario: symlink 失敗時に copy へ切り替える
- Given symlink を作成できない環境である
- When `install-skill` を実行する
- Then copy モードでスキルが配置される

### Requirement: install-skill は JSON 出力を返す
`install-skill` の出力は JSON で、`ok`/`type`/`schemaVersion` を含まなければならない。(MUST)
インストール結果として `skills` 配列に `name`/`path`/`source_type` を含めなければならない。(MUST)

#### Scenario: JSON 出力でインストール結果を返す
- Given `install-skill` を実行する
- When インストールが成功する
- Then JSON で結果が出力される
- And `skills` 配列にインストール情報が含まれる

### Requirement: イントロスペクションに install-skill が含まれる
`commands --json` は `install-skill` を含むコマンド一覧を返さなければならない。(MUST)
`schema --command install-skill --output json-schema` は出力スキーマを返さなければならない。(MUST)

#### Scenario: commands に install-skill が含まれる
- Given `slack-rs commands --json` を実行する
- Then `install-skill` がコマンド一覧に含まれる

#### Scenario: install-skill の schema が取得できる
- Given `slack-rs schema --command install-skill --output json-schema` を実行する
- Then JSON Schema が出力される
