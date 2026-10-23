# Release notes: release-0.1.5

公開日: 2026-10-14  
タグ: `release-0.1.5`

## 概要

`release-0.1.5` は JadeMusic の Linux 向け配布リリースです。  
このリリースは実行バイナリと README を同梱しており、必要な外部ツールと Discord Bot の認証情報を用意することで、YouTube などの音声をボイスチャンネルで再生できます。

## ハイライト

- `/music play`、`skip`、`list`、`shuffle`、`delete`、`join`、`leave` の各スラッシュコマンドに対応
- 再生キューへの追加時にタイトル、URL、再生時間を表示
- 再生中の曲を維持したまま待機キューをシャッフル
- ボイスチャンネルから利用者がいなくなった場合の自動退出とキュー消去に対応
- yt-dlp の `bestaudio` を使用し、再生音質を改善

## 同梱物

- `JadeMusic`（実行ファイル）
- `README.MD`（セットアップ、設定、コマンド一覧）

## 実行前提

- OS: Linux x86_64
- 共有ライブラリ: glibc 系ランタイムが必要（動的リンク）
- 外部ツール: `ffmpeg`、`yt-dlp`、`deno`
- 推奨 yt-dlp バージョン: `2026.03.17` 以降
- 環境変数: `DISCORD_TOKEN`、`YTDLP_COOKIES`
- Discord Bot 設定: Server Members Intent を有効化

## 既知の注意点

- このリリースには Rust ソースコードと Cargo 関連ファイル（`src/`、`Cargo.toml`、`Cargo.lock`）は含まれていません。
- README の Cargo ビルド手順を利用する場合は `main` ブランチのソースコードが必要です。
- README には musl 向け静的ビルド手順がありますが、同梱バイナリは glibc を使用する動的リンク版です。
- `YTDLP_COOKIES` には yt-dlp 形式の Cookie ファイルへのパスを指定する必要があります。
- `main` ブランチの `Cargo.toml` に記載された package version は `0.1.0` のままです。

## 変更点（main ブランチ比）

- 追加: `JadeMusic`（実行バイナリ）
- 維持: `README.MD`、`.gitignore`
- 削除: `.cargo/config.toml`、`Cargo.toml`、`Cargo.lock`、`src/*`

## 検証情報

- `JadeMusic` SHA256: `88e0968fc778fb6fbf79c0a1dfa05d57815f929302b6ab95c910147553ea0658`
- ファイル形式: ELF 64-bit LSB PIE executable, x86-64
- ファイルサイズ: 16,201,808 bytes
- 動的リンク先: `libgcc_s.so.1`、`libm.so.6`、`libc.so.6`

---

ソースコードが必要な場合は `main` ブランチを参照してください。
