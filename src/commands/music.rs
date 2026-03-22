use crate::{Error, Context};
use rand::seq::SliceRandom;
use songbird::input::YoutubeDl;
use songbird::tracks::Track;
use std::sync::Arc;

use dotenv::dotenv;

#[derive(Clone, Debug, Default)]
struct TrackData {
    source_url: Option<String>,
    duration: Option<std::time::Duration>,
}

fn format_duration(duration: Option<std::time::Duration>) -> Option<String> {
    let total = duration?.as_secs();
    let minutes = total / 60;
    let seconds = total % 60;
    Some(format!("{minutes}:{seconds:02}"))
}

fn check_msg<T, E: std::fmt::Debug>(result: Result<T, E>) {
    if let Err(why) = result {
        println!("Error sending message: {why:?}");
    }
}

#[poise::command(
    slash_command,
    subcommands("play", "skip", "join", "leave", "shuffle", "list", "delete"),
    guild_only
)]
pub async fn music(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let channel_id = ctx
        .guild()
        .unwrap()
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .ok_or("先にボイスチャンネルに参加してください")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .unwrap();

    manager.join(guild_id, channel_id).await?;
    check_msg(ctx.say("ボイスチャンネルに参加しました").await);
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .unwrap();

    if manager.get(guild_id).is_none() {
        check_msg(ctx.say("ボイスチャンネルに参加していません").await);
        return Ok(());
    }

    manager.remove(guild_id).await?;
    check_msg(ctx.say("ボイスチャンネルから退出しました").await);
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "再生するURL (YouTube / ニコニコ動画 / etc…)"]
    url: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let do_search = !url.starts_with("http");

    let guild_id = ctx.guild_id().unwrap();

    // poise では ctx.data() が &Data を直接返す
    let http_client = ctx.data().http_client.clone();

    // songbird::get には &serenity::Context が必要
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = if let Some(existing) = manager.get(guild_id) {
        existing
    } else {
        let channel_id = ctx
            .guild()
            .unwrap()
            .voice_states
            .get(&ctx.author().id)
            .and_then(|vs| vs.channel_id)
            .ok_or("先にボイスチャンネルに参加してください")?;
        manager.join(guild_id, channel_id).await?;
        manager
            .get(guild_id)
            .ok_or("ボイスチャンネルに参加できませんでした")?
    };

    let mut handler = handler_lock.lock().await;

    dotenv().ok();
    let browser = std::env::var("YTDLP_BROWSER").expect("YTDLP_BROWSER environment variable not set");
    let cookies = std::env::var("YTDLP_COOKIES").expect("YTDLP_COOKIES environment variable not set");

    let extra_args = vec![
        "--cookies".to_string(), cookies,
        "--cookies-from-browser".to_string(), browser,
        "--js-runtime".to_string(), "deno".to_string(),
    ];

    let src = if do_search {
        YoutubeDl::new_search(http_client, url)
    } else {
        YoutubeDl::new(http_client, url)
    }
    .user_args(extra_args);
    let mut input: songbird::input::Input = src.into();
    let aux = input.aux_metadata().await.ok();
    let track = Track::new_with_data(
        input,
        Arc::new(TrackData {
            source_url: aux.as_ref().and_then(|m| m.source_url.clone()),
            duration: aux.as_ref().and_then(|m| m.duration),
        }),
    );
    let _ = handler.enqueue(track).await;

    check_msg(ctx.say("Playing song").await);
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .unwrap();

    let handler_lock = manager
        .get(guild_id)
        .ok_or("ボイスチャンネルに参加していません")?;

    handler_lock.lock().await.queue().skip()?;
    check_msg(ctx.say("スキップしました").await);
    Ok(())
}

// TODO: Tier2 shuffleコマンドの実装
// 再生キューをシャッフルするコマンド
#[poise::command(slash_command, guild_only)]
pub async fn shuffle(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .unwrap();

    let handler_lock = manager
        .get(guild_id)
        .ok_or("ボイスチャンネルに参加していません")?;

    let handler = handler_lock.lock().await;
    let queue = handler.queue();
    let len = queue.len();

    if len <= 1 {
        check_msg(ctx.say("シャッフルできる曲がありません").await);
        return Ok(());
    }

    queue.modify_queue(|vq| {
        let mut rng = rand::thread_rng();
        let slice = vq.make_contiguous();

        if slice.len() > 1 {
            slice[1..].shuffle(&mut rng);
        }
    });

    check_msg(ctx.say(format!("待機キューをシャッフルしました ({} 件)", len - 1)).await);
    Ok(())
}

// TODO: Tier1 listコマンドの実装
// 再生キューの内容を表示するコマンド
#[poise::command(slash_command, guild_only)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {

    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .unwrap();

    let handler_lock = manager
        .get(guild_id)
        .ok_or("ボイスチャンネルに参加していません")?;

    let handler = handler_lock.lock().await;
    let queue = handler.queue();
    let tracks = queue.current_queue();

    if tracks.is_empty() {
        check_msg(ctx.say("再生キューは空です").await);
        return Ok(());
    }

    let max_show = 15usize;
    let mut lines = Vec::new();

    for (i, handle) in tracks.iter().take(max_show).enumerate() {
        let status = if i == 0 { "再生中" } else { "待機" };
        let data = handle.data::<TrackData>();
        let url = data
            .source_url
            .clone()
            .unwrap_or_else(|| "URL不明".to_string());
        let duration = format_duration(data.duration);
        let line = if let Some(d) = duration {
            format!("{}. [{}] {} ({})", i + 1, status, url, d)
        } else {
            format!("{}. [{}] {}", i + 1, status, url)
        };
        lines.push(line);
    }

    if tracks.len() > max_show {
        lines.push(format!("...他 {} 件", tracks.len() - max_show));
    }

    let embed = poise::serenity_prelude::CreateEmbed::default()
        .title(format!("再生キュー: {} 件", tracks.len()))
        .description(lines.join("\n"));
    check_msg(ctx.send(poise::CreateReply::default().embed(embed)).await);
    Ok(())
}

// TODO: Tier1 deleteコマンドの実装
// 再生キューの特定の曲を削除するコマンド
#[poise::command(slash_command, guild_only)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "削除する曲番号 (list の 1 始まり)"] index: usize,
) -> Result<(), Error> {

    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .unwrap();

    let handler_lock = manager
        .get(guild_id)
        .ok_or("ボイスチャンネルに参加していません")?;

    let handler = handler_lock.lock().await;
    let queue = handler.queue();
    let len = queue.len();

    if len == 0 {
        check_msg(ctx.say("再生キューは空です").await);
        return Ok(());
    }

    if index == 0 || index > len {
        check_msg(ctx.say(format!("無効な番号です。1 から {len} の範囲で指定してください")).await);
        return Ok(());
    }

    if index == 1 {
        check_msg(ctx.say("再生中の曲は削除できません。/music skip を使ってください").await);
        return Ok(());
    }

    let removed = queue.dequeue(index - 1);
    if let Some(track) = removed {
        let _ = track.stop();
        check_msg(ctx.say(format!("{} 番目の曲を削除しました", index)).await);
    } else {
        check_msg(ctx.say("削除対象を見つけられませんでした").await);
    }

    Ok(())
}
