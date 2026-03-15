use crate::{Error, Context};

#[poise::command(slash_command)]
pub async fn notify(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("通知機能は現在開発中です").await?;
    Ok(())
}
