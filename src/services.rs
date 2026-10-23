use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::GuildId;
use songbird::Songbird;
use songbird::tracks::TrackHandle;
use std::{collections::HashSet, sync::Arc, time::Duration};

use crate::{Context, Error};

pub fn check_msg<T, E: std::fmt::Debug>(result: Result<T, E>) {
    if let Err(why) = result {
        println!("Error sending message: {why:?}");
    }
}

pub async fn get_handler_lock(
    ctx: &Context<'_>,
) -> Result<Arc<tokio::sync::Mutex<songbird::Call>>, Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap();
    manager
        .get(guild_id)
        .ok_or_else(|| "ボイスチャンネルに参加していません".into())
}

#[derive(Clone, Debug, Default)]
pub struct TrackData {
    pub title: Option<String>,
    pub source_url: Option<String>,
    pub duration: Option<std::time::Duration>,
}

pub fn is_youtube_mix(url: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(url) else {
        return false;
    };
    let host = url.host_str().unwrap_or_default();
    matches!(url.scheme(), "http" | "https")
        && (host == "youtube.com" || host.ends_with(".youtube.com") || host == "youtu.be")
        && url
            .query_pairs()
            .any(|(key, value)| key == "list" && value.starts_with("RD"))
}

pub async fn youtube_mix_tracks(
    url: &str,
    cookies: &str,
    limit: usize,
) -> Result<Vec<TrackData>, Error> {
    let output = tokio::time::timeout(
        Duration::from_secs(60),
        tokio::process::Command::new("yt-dlp")
            .args([
                "--ignore-config",
                "--flat-playlist",
                "--dump-single-json",
                "--yes-playlist",
                "--playlist-items",
                &format!("1:{limit}"),
                "--cookies",
                cookies,
                "--",
                url,
            ])
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| "ミックスリストの取得がタイムアウトしました")??;

    if !output.status.success() {
        eprintln!(
            "YouTube mix extraction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(
            "ミックスリストの取得に失敗しました。URL・cookies・yt-dlpを確認してください".into(),
        );
    }
    parse_mix_tracks(&output.stdout)
}

fn parse_mix_tracks(json: &[u8]) -> Result<Vec<TrackData>, Error> {
    #[derive(serde::Deserialize)]
    struct Entry {
        id: Option<String>,
        title: Option<String>,
        duration: Option<f64>,
        availability: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct Playlist {
        entries: Vec<Option<Entry>>,
    }

    let playlist: Playlist = serde_json::from_slice(json)?;
    let mut seen = HashSet::new();
    let tracks: Vec<_> = playlist
        .entries
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let id = entry.id?;
            if id.len() != 11
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
                || entry.availability.as_deref() == Some("private")
                || !seen.insert(id.clone())
            {
                return None;
            }
            Some(TrackData {
                title: entry.title,
                source_url: Some(format!("https://www.youtube.com/watch?v={id}")),
                duration: entry
                    .duration
                    .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok()),
            })
        })
        .collect();
    if tracks.is_empty() {
        return Err("ミックスリストに追加できる曲がありません".into());
    }
    Ok(tracks)
}

pub fn format_duration(duration: Option<std::time::Duration>) -> Option<String> {
    let total = duration?.as_secs();
    let minutes = total / 60;
    let seconds = total % 60;
    Some(format!("{minutes}:{seconds:02}"))
}

pub fn build_queue_embed(tracks: &[TrackHandle]) -> CreateEmbed {
    let max_show = 15usize;
    let mut lines = Vec::new();

    for (i, handle) in tracks.iter().take(max_show).enumerate() {
        let status = if i == 0 { "再生中" } else { "待機" };
        let data = handle.data::<TrackData>();
        let title = data
            .title
            .clone()
            .or_else(|| data.source_url.clone())
            .unwrap_or_else(|| "不明".to_string());
        let duration = format_duration(data.duration);
        let line = if let Some(d) = duration {
            format!("{}. [{}] {} ({})", i + 1, status, title, d)
        } else {
            format!("{}. [{}] {}", i + 1, status, title)
        };
        lines.push(line);
    }

    if tracks.len() > max_show {
        lines.push(format!("...他 {} 件", tracks.len() - max_show));
    }

    CreateEmbed::default()
        .title(format!("再生キュー: {} 件", tracks.len()))
        .description(lines.join("\n"))
}

/// ボイスチャンネルから退出し、再生キューをすべて空にする
pub async fn clear_queue(manager: &Songbird, guild_id: GuildId) {
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        handler.queue().stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_only_youtube_mix_urls() {
        for url in [
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=RDdQw4w9WgXcQ&start_radio=1",
            "https://youtu.be/dQw4w9WgXcQ?list=RDdQw4w9WgXcQ",
            "https://music.youtube.com/watch?v=dQw4w9WgXcQ&list=RDAMVMdQw4w9WgXcQ",
            "https://m.youtube.com/playlist?list=%52%44example",
        ] {
            assert!(is_youtube_mix(url), "{url}");
        }
        for url in [
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLexample",
            "https://www.youtube.com/playlist?list=PLexample",
            "https://youtube.com.evil.example/watch?list=RDexample",
            "https://notyoutube.com/watch?list=RDexample",
            "https://example.com/?url=https://youtube.com&list=RDexample",
            "ftp://www.youtube.com/playlist?list=RDexample",
            "RDexample music search",
        ] {
            assert!(!is_youtube_mix(url), "{url}");
        }
    }

    #[test]
    fn parses_ordered_mix_without_duplicate_or_unusable_entries() {
        let tracks = parse_mix_tracks(
            br#"{"entries":[
            {"id":"aaaaaaaaaaa","title":"First","duration":123.5,"url":"https://untrusted.example"},
            null,
            {"id":"bbbbbbbbbbb","title":"Second","duration":-1},
            {"id":"aaaaaaaaaaa","title":"Duplicate"},
            {"id":"ccccccccccc","availability":"private"},
            {"id":"bad/id"},
            {"title":"Missing ID"}
        ]}"#,
        )
        .unwrap();
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].title.as_deref(), Some("First"));
        assert_eq!(
            tracks[0].source_url.as_deref(),
            Some("https://www.youtube.com/watch?v=aaaaaaaaaaa")
        );
        assert_eq!(tracks[0].duration, Some(Duration::from_millis(123_500)));
        assert_eq!(tracks[1].title.as_deref(), Some("Second"));
        assert_eq!(
            tracks[1].source_url.as_deref(),
            Some("https://www.youtube.com/watch?v=bbbbbbbbbbb")
        );
        assert_eq!(tracks[1].duration, None);
    }

    #[test]
    fn permits_missing_title_and_duration() {
        let tracks = parse_mix_tracks(br#"{"entries":[{"id":"aaaaaaaaaaa"}]}"#).unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, None);
        assert_eq!(tracks[0].duration, None);
    }

    #[test]
    fn rejects_empty_or_invalid_playlist_responses() {
        for json in [
            "not json",
            "null",
            "{}",
            r#"{"entries":[]}"#,
            r#"{"entries":[null,{"id":"bad"},{"id":"aaaaaaaaaaa","availability":"private"}]}"#,
        ] {
            assert!(parse_mix_tracks(json.as_bytes()).is_err(), "{json}");
        }
    }
}
