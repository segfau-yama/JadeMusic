use reqwest::Client;
use songbird::input::{Compose, Input, YoutubeDl};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub struct ResolvedTrack {
    pub input: Input,
    pub title: String,
}

// TODO: Spotify, 音声ファイルに対応する
// TODO: Strategy
pub async fn resolve(url: &str) -> Result<ResolvedTrack, BoxError> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("無効なURL: {url}").into());
    }

    let mut source = YoutubeDl::new(Client::new(), url.to_string());
    let metadata = source.aux_metadata().await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("os error 2") || msg.contains("No such file or directory") {
            BoxError::from("yt-dlp が見つかりません。音楽再生には `yt-dlp` のインストールが必要です。")
        } else {
            BoxError::from(format!("URL を解決できませんでした: {msg}"))
        }
    })?;

    let title = metadata
        .track
        .or(metadata.title)
        .unwrap_or_else(|| url.to_string());

    Ok(ResolvedTrack {
        input: source.into(),
        title,
    })
}

// TODO: 音楽保存用サービスの作成(上限1GB)

// TODO: ローカル再生用サービスの作成
