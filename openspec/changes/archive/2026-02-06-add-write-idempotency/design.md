## Goal
エージェント暴走による二重書き込みを、CLI 単体で予防できる冪等機構として提供する。再実行時は同じ結果を返し、実際の Slack 書き込みを発生させない。

## Non-Goals
- Slack API 側の冪等キー機能追加
- `api call` のリトライ方針変更
- 既存の確認フロー（`--yes`/`SLACKCLI_ALLOW_WRITE`）の削除

## Design Overview
- `--idempotency-key` 指定時に、ローカルストアを参照する。
  - 既存エントリがあり、同一リクエストなら replay して終了（Slack へ送信しない）。
  - 既存エントリがあり、リクエスト内容が異なる場合は即時エラー（fail fast）。
  - 既存エントリが無い場合は実行し、成功レスポンスを保存して返す。
- 期限管理は TTL 7日、件数上限 10,000件、GC（期限切れ削除 + 上限超過削除）を行う。

## Keying Strategy
- キーは意図に紐づく安定値が前提。
- CLI 内では以下でスコープ化し衝突を防止する。
  - `team_id + user_id + method + idempotency_key`
- Slack 上の「最終発言 ID」などの状態依存値は冪等キーとして不適切。

## Request Fingerprint
- 同一キーで内容が変わる事故を防ぐため、リクエストの正規化ハッシュを保存する。
- ハッシュ対象は最低限以下を含む。
  - コマンド名、method、主要パラメータ（channel, ts, text, thread_ts, emoji 等）

## Storage
- 保存先: OS の config dir（`~/.config/slack-rs/idempotency_store.json`）
- Unix では 0600 権限に設定する。
- 形式: JSON（`key -> entry`）

## Response Meta
- `--idempotency-key` 指定時のみ `meta.idempotency_key` と `meta.idempotency_status` を付与する。
  - `idempotency_status`: `executed` | `replayed`

## GC Policy
- 期限切れ削除: `expires_at <= now` を削除
- 件数上限超過時: `created_at` の古い順に削除
- GC 実行タイミング: ストア読み込み時と書き込み前
