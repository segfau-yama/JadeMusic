
// use poise::serenity_prelude::CreateEmbed;
// use songbird::tracks::TrackQueue;

// pub async fn show_playlist(tracks: TrackQueue) -> CreateEmbed {
//     let max_show = 15usize;
//     let mut lines = Vec::new();

//     for (i, handle) in tracks.iter().take(max_show).enumerate() {
//         let status = if i == 0 { "再生中" } else { "待機" };
//         let data = handle.data::<TrackData>();
//         let url = data
//             .source_url
//             .clone()
//             .unwrap_or_else(|| "URL不明".to_string());
//         let duration = format_duration(data.duration);
//         let line = if let Some(d) = duration {
//             format!("{}. [{}] {} ({})", i + 1, status, url, d)
//         } else {
//             format!("{}. [{}] {}", i + 1, status, url)
//         };
//         lines.push(line);
//     }

//     if tracks.len() > max_show {
//         lines.push(format!("...他 {} 件", tracks.len() - max_show));
//     }

//     CreateEmbed::default()
//         .title(format!("再生キュー: {} 件", tracks.len()))
//         .description(lines.join("\n"))
// }
