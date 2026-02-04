## 概要
`auth login` に `--ngrok [path]` を追加し、ngrok を使った動的 redirect_uri を選択できるようにする。`--cloudflared` と同時指定は不可とし、ngrok 使用時は Manifest の redirect_urls に `https://*.ngrok-free.app/callback` を含める。

## 背景
- cloudflared が使えない環境でもトンネル経由の OAuth を行いたい。
- ngrok を使う場合、動的に生成される URL に対応した Manifest が必要になる。

## ゴール
- `auth login --ngrok [path]` で ngrok を利用できる。
- `--ngrok` が指定された場合、ngrok の公開 URL を redirect_uri に使用できる。
- ngrok 使用時は Manifest の redirect_urls に `https://*.ngrok-free.app/callback` が含まれる。
- `--ngrok` と `--cloudflared` の同時指定はエラーになる。

## 非ゴール
- ngrok のインストール自動化
- ngrok のカスタムドメイン対応

## 影響範囲
- `auth login` の引数と redirect_uri 解決ロジック
- Manifest の redirect_urls 生成ルール

## 成果物
- `auth login --ngrok [path]` 対応
- Manifest 生成の ngrok 分岐
- 新/更新仕様（specs）
