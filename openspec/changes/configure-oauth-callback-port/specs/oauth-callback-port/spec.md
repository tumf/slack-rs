# oauth-callback-port Specification (Delta)

## Purpose
OAuthコールバックサーバーの待受ポートを安全に設定できるようにし、一般的な開発ポートとの衝突を減らす。

## ADDED Requirements

### Requirement: 既定のコールバックポートは8765である

`SLACK_OAUTH_PORT` が未設定の場合、OAuthコールバックサーバーはポート8765で待受しなければならない (MUST)。

#### Scenario: 環境変数が未設定のときに既定ポートを使用する
- Given `SLACK_OAUTH_PORT` が未設定である
- When OAuthログインのコールバック待受を開始する
- Then 8765番ポートで待受する

### Requirement: 環境変数で待受ポートを上書きできる

`SLACK_OAUTH_PORT` が1〜65535の有効な数値の場合、OAuthコールバックサーバーはそのポートで待受しなければならない (MUST)。

#### Scenario: 有効な環境変数で待受ポートが上書きされる
- Given `SLACK_OAUTH_PORT=13000` が設定されている
- When OAuthログインのコールバック待受を開始する
- Then 13000番ポートで待受する

### Requirement: 無効な環境変数は起動前に拒否する

`SLACK_OAUTH_PORT` が数値ではない、または範囲外の場合、起動前に設定エラーとして扱わなければならない (MUST)。

#### Scenario: 無効な環境変数でエラーになる
- Given `SLACK_OAUTH_PORT=abc` が設定されている
- When OAuthログインのコールバック待受を開始する
- Then 設定エラーが返され、待受は開始されない
