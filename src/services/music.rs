use reqwest::Client;
use songbird::input::{Input, YoutubeDl};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

// TODO: Spotify, 音声ファイルに対応する
// TODO: Strategy
pub fn resolve(url: &str) -> Result<Input, BoxError> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("無効なURL: {url}").into());
    }
    Ok(YoutubeDl::new(Client::new(), url.to_string()).into())
}

// TODO: 音楽保存用サービスの作成(上限1GB)

// TODO: ローカル再生用サービスの作成
