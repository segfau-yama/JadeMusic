use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use serde::Deserialize;
use songbird::input::{ChildContainer, HttpRequest, Input, RawAdapter};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use symphonia::core::io::ReadOnlySource;
use tokio::process::Command as TokioCommand;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub struct ResolvedTrack {
    pub input: Input,
    pub title: String,
}

#[derive(Debug, Deserialize)]
struct YtdlpOutput {
    url: String,
    protocol: Option<String>,
    http_headers: Option<HashMap<String, String>>,
    filesize: Option<u64>,
    title: Option<String>,
    track: Option<String>,
}

pub async fn resolve(url: &str) -> Result<ResolvedTrack, BoxError> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("無効なURL: {url}").into());
    }

    let mut args: Vec<String> = Vec::new();
    if let Ok(extra) = std::env::var("YTDL_ARGS") {
        args.extend(extra.split_whitespace().map(|s| s.to_string()));
    }
    if let Ok(cookies) = std::env::var("YTDL_COOKIES") {
        args.push("--cookies".to_string());
        args.push(cookies);
    }

    let output = TokioCommand::new("yt-dlp")
        .args(args)
        .arg("-j")
        .arg(url)
        .arg("-f")
        .arg("ba[abr>0][vcodec=none]/best")
        .arg("--no-playlist")
        .output()
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("os error 2") || msg.contains("No such file or directory") {
                BoxError::from("yt-dlp が見つかりません。音楽再生には `yt-dlp` のインストールが必要です。")
            } else {
                BoxError::from(format!("yt-dlp の実行に失敗しました: {msg}"))
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp が失敗しました: {stderr}").into());
    }

    let first = output
        .stdout
        .split(|&b| b == b'\n')
        .find(|l| !l.is_empty())
        .ok_or_else(|| BoxError::from("yt-dlp の結果が空です"))?;

    let result: YtdlpOutput = serde_json::from_slice(first)
        .map_err(|e| format!("yt-dlp のJSON解析に失敗しました: {e}"))?;

    let title = result
        .track
        .or(result.title)
        .unwrap_or_else(|| url.to_string());

    let mut headers = HeaderMap::default();
    if let Some(map) = result.http_headers {
        headers.extend(map.iter().filter_map(|(k, v)| {
            Some((
                HeaderName::from_bytes(k.as_bytes()).ok()?,
                HeaderValue::from_str(v).ok()?,
            ))
        }));
    }

    let is_hls = result
        .protocol
        .as_deref()
        .map(|p| p.contains("m3u8"))
        .unwrap_or(false)
        || result.url.contains(".m3u8");

    let input = if is_hls {
        let mut header_str = String::new();
        for (k, v) in headers.iter() {
            if let Ok(val) = v.to_str() {
                header_str.push_str(k.as_str());
                header_str.push_str(": ");
                header_str.push_str(val);
                header_str.push_str("\r\n");
            }
        }

        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-loglevel")
            .arg("warning")
            .arg("-nostdin");
        if !header_str.is_empty() {
            cmd.arg("-headers").arg(header_str);
        }
        cmd.arg("-i")
            .arg(&result.url)
            .arg("-vn")
            .arg("-acodec")
            .arg("pcm_f32le")
            .arg("-f")
            .arg("f32le")
            .arg("-ar")
            .arg("48000")
            .arg("-ac")
            .arg("2")
            .arg("pipe:1")
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let child = cmd.spawn().map_err(|e| {
            let msg = e.to_string();
            if msg.contains("No such file or directory") {
                BoxError::from("ffmpeg が見つかりません。音楽再生には `ffmpeg` のインストールが必要です。")
            } else {
                BoxError::from(format!("ffmpeg の起動に失敗しました: {msg}"))
            }
        })?;

        let child = ChildContainer::from(child);
        let source = ReadOnlySource::new(child);
        let raw = RawAdapter::new(source, 48_000, 2);
        Input::from(raw)
    } else {
        let req = HttpRequest {
            client: Client::new(),
            request: result.url,
            headers,
            content_length: result.filesize,
        };
        Input::from(req)
    };

    Ok(ResolvedTrack { input, title })
}
