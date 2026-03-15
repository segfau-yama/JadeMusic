use crate::{Error, Context};

#[poise::command(
    slash_command, 
    subcommands("markov", "llm")
)]
pub async fn chat(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn markov(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn llm(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}