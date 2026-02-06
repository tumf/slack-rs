# cli-profile-selection Specification

## Purpose
CLI 全体で profile 解決を一貫させ、`--profile` と `SLACK_PROFILE` の優先順位を明確にする。

## ADDED Requirements

### Requirement: Global profile resolution precedence is consistent
全コマンドの profile 解決は `--profile` > `SLACK_PROFILE` > `default` の優先順位でなければならない。(MUST)

#### Scenario: `--profile` が指定されている場合
- Given `SLACK_PROFILE=work` が設定されている
- When `slack-rs --profile personal api call auth.test` を実行する
- Then 解決される profile は `personal` である

#### Scenario: `SLACK_PROFILE` が指定されている場合
- Given `SLACK_PROFILE=work` が設定されている
- And `--profile` が指定されていない
- When `slack-rs api call auth.test` を実行する
- Then 解決される profile は `work` である

#### Scenario: どちらも指定されていない場合
- Given `SLACK_PROFILE` が未設定である
- And `--profile` が指定されていない
- When `slack-rs api call auth.test` を実行する
- Then 解決される profile は `default` である

### Requirement: `--profile` is accepted in both formats and positions
`--profile <name>` と `--profile=<name>` の両形式を、サブコマンドの前後どちらに置いても受け付けなければならない。(MUST)

#### Scenario: 前置きの `--profile` を使用する
- When `slack-rs --profile work conv list` を実行する
- Then profile は `work` として解決される

#### Scenario: 後置きの `--profile` を使用する
- When `slack-rs conv list --profile work` を実行する
- Then profile は `work` として解決される
