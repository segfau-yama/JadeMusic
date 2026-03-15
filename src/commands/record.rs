
// TODO: Tier2 録音機能の追加
use crate::{Error, Context};

#[poise::command(
    slash_command,
    subcommands("record", "stop", "save", "skip", "join", "leave")
)]
pub async fn record(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("録音機能は現在開発中です").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn start(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("録音機能は現在開発中です").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("録音機能は現在開発中です").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn save(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("録音機能は現在開発中です").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("録音機能は現在開発中です").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("録音機能は現在開発中です").await?;
    Ok(())
}