// use reqwest::Client;
// use songbird::input::{Compose, Input, YoutubeDl};
// type BoxError = Box<dyn std::error::Error + Send + Sync>;

// pub struct ResolvedTrack {
//     pub input: Input,
//     pub title: String,
// }

// pub async fn resolve(url: &str, client: Client) -> Result<ResolvedTrack, BoxError> {
//     let query = url.trim();
//     if query.is_empty() {
//         return Err("URLまたは検索クエリを入力してください".into());
//     }

//     let mut args: Vec<String> = Vec::new();

//     // 安定構成: --cookies cookies.txt --js-runtime node
//     let cookies_file = std::env::var("YTDL_COOKIES")
//         .ok()
//         .filter(|s| !s.trim().is_empty())
//         .unwrap_or_else(|| "cookies.txt".to_string());
//     args.extend(["--cookies".to_string(), cookies_file]);
//     args.extend(["--js-runtime".to_string(), "node".to_string()]);

//     let do_search = !query.starts_with("http://") && !query.starts_with("https://");

//     let mut source: YoutubeDl<'static> = if do_search {
//         YoutubeDl::new_search(client, query.to_string())
//     } else {
//         YoutubeDl::new(client, query.to_string())
//     }
//     .user_args(args);

//     let metadata = source
//         .aux_metadata()
//         .await
//         .map_err(|e| {
//             let msg = e.to_string();
//             if msg.contains("could not find executable 'yt-dlp' on path")
//                 || msg.contains("No such file or directory")
//             {
//                 BoxError::from("yt-dlp が見つかりません。音楽再生には `yt-dlp` のインストールが必要です。")
//             } else if msg.contains("Sign in to confirm you're not a bot") {
//                 BoxError::from(
//                     "YouTube の bot 確認でブロックされました。cookies.txt を再エクスポートするか、`YTDL_COOKIES=<パス>` を設定してください。",
//                 )
//             } else {
//                 BoxError::from(format!("yt-dlp から音源情報を取得できませんでした: {msg}"))
//             }
//         })?;

//     let title = metadata
//         .track
//         .or(metadata.title)
//         .unwrap_or_else(|| query.to_string());

//     let input = Input::from(source);

//     Ok(ResolvedTrack { input, title })
// }

