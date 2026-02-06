## MODIFIED Requirements

### Requirement: マニフェスト保存後のクリップボードコピーは複数手段で試行する

`auth login` のマニフェスト生成後に行うコピー処理は、以下の順序で best effort として試行しなければならない (MUST)。いずれかが成功すれば完了とし、失敗しても処理は中断しない。

- SSH 環境（TTY かつ `SSH_CONNECTION` または `SSH_TTY` が存在）では OSC52 を優先して試行する。
- `arboard` を試行する。
- OS コマンド（macOS: `pbcopy` / Windows: `clip` / Linux: `wl-copy`, `xclip`, `xsel`）を順に試行する。

#### Scenario: SSH 環境では OSC52 を先に試行する
- Given `auth login` 実行時に SSH 環境が検出される
- When マニフェスト保存後のコピー処理が行われる
- Then OSC52 によるコピーを先に試行する

#### Scenario: `arboard` が失敗した場合は OS コマンドにフォールバックする
- Given `arboard` によるコピーが失敗する
- When マニフェスト保存後のコピー処理が継続される
- Then OS コマンドによるコピーを順に試行する
