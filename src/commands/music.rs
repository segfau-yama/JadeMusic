use crate::{Error, Context};

#[poise::command(
    slash_command,
    subcommands("play", "skip", "join", "leave"),
    guild_only
)]
pub async fn music(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    ctx.say("利用可能なサブコマンド: play, skip, join, leave").await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().ok_or("ギルド外では使えません")?;

    let channel_id = ctx
        .guild()
        .ok_or("ギルド情報を取得できませんでした")?
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .ok_or("先にボイスチャンネルに参加してください")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird が初期化されていません")?;

    manager.join(guild_id, channel_id).await?;
    ctx.say("ボイスチャンネルに参加しました").await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().ok_or("ギルド外では使えません")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird が初期化されていません")?;

    if manager.get(guild_id).is_none() {
        ctx.say("ボイスチャンネルに参加していません").await?;
        return Ok(());
    }

    manager.remove(guild_id).await?;
    ctx.say("ボイスチャンネルから退出しました").await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "再生するURL (YouTube / ニコニコ動画 / Spotify)"]
    url: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().ok_or("ギルド外では使えません")?;

    let channel_id = ctx
        .guild()
        .ok_or("ギルド情報を取得できませんでした")?
        .voice_states
        .get(&ctx.author().id)
        .and_then(|vs| vs.channel_id)
        .ok_or("先にボイスチャンネルに参加してください")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird が初期化されていません")?;

    let handler_lock = match manager.get(guild_id) {
        Some(h) => h,
        None => manager.join(guild_id, channel_id).await?,
    };

    let track = match crate::services::music::resolve(&url).await {
        Ok(track) => track,
        Err(e) => {
            ctx.say(format!("再生できませんでした: {e}")).await?;
            return Ok(());
        }
    };
    handler_lock.lock().await.play_input(track.input);

    ctx.say(format!("再生中: {}", track.title)).await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().ok_or("ギルド外では使えません")?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or("Songbird が初期化されていません")?;

    let handler_lock = manager
        .get(guild_id)
        .ok_or("ボイスチャンネルに参加していません")?;

    handler_lock.lock().await.queue().skip()?;
    ctx.say("スキップしました").await?;
    Ok(())
}

// TODO: Tier1 shuffleコマンドの実装
